use super::{Segment, RankingSegment};
use crate::config::InputData;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};

// 全局缓存，避免重复 API 调用
lazy_static::lazy_static! {
    static ref API_CACHE: Arc<Mutex<Option<(UserApiResponse, SystemTime)>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiQuota {
    remaining: f64,
    total: f64,
    used: f64,
    timestamp: SystemTime,
}

// API 响应结构 - 根据 packycode-cost 项目定义
#[derive(Debug, Deserialize, Clone)]
struct UserApiResponse {
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    daily_budget_usd: f64,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    daily_spent_usd: f64,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    monthly_budget_usd: f64,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    monthly_spent_usd: f64,
    opus_enabled: Option<bool>,
}

// 自定义反序列化函数，将字符串转换为 f64
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrFloat {
        String(String),
        Float(f64),
    }
    
    match StringOrFloat::deserialize(deserializer)? {
        StringOrFloat::String(s) => s.parse::<f64>()
            .map_err(|_| de::Error::custom("Failed to parse string as f64")),
        StringOrFloat::Float(f) => Ok(f),
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicUsageResponse {
    #[serde(rename = "remaining_credit_in_usd")]
    remaining: f64,
    #[serde(rename = "credit_limit_in_usd")]
    limit: f64,
}

#[derive(Debug, Deserialize)]
struct ClaudeCodeSettings {
    env: Option<ClaudeCodeEnv>,
    info_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeCodeEnv {
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    base_url: Option<String>,
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    auth_token: Option<String>,
    #[serde(rename = "ANTHROPIC_API_KEY")]
    api_key: Option<String>,
}

pub struct QuotaSegment {
    enabled: bool,
    api_key: Option<String>,
    base_url: String,
    info_url: Option<String>,
    jwt_token: Option<String>,
}

impl QuotaSegment {
    pub fn new(enabled: bool) -> Self {
        let (api_key, base_url, info_url) = Self::load_api_config();
        Self {
            enabled,
            api_key,
            base_url,
            info_url,
            jwt_token: None,
        }
    }

    pub fn new_with_config(enabled: bool, jwt_token: Option<String>) -> Self {
        let (api_key, base_url, info_url) = Self::load_api_config();
        Self {
            enabled,
            api_key,
            base_url,
            info_url,
            jwt_token,
        }
    }

    fn load_api_config() -> (Option<String>, String, Option<String>) {
        // Try multiple sources for API configuration

        // 1. Claude Code settings.json
        if let Some(config_dir) = Self::get_claude_config_dir() {
            let settings_path = config_dir.join("settings.json");
            if let Ok(content) = fs::read_to_string(&settings_path) {
                if let Ok(settings) = serde_json::from_str::<ClaudeCodeSettings>(&content) {
                    let info_url = settings.info_url.clone();
                    if let Some(env) = settings.env {
                        let api_key = env.auth_token.or(env.api_key);
                        let base_url = env
                            .base_url
                            .unwrap_or_else(|| "https://api.anthropic.com".to_string());
                        if api_key.is_some() {
                            return (api_key, base_url, info_url);
                        }
                    }
                }
            }
        }

        // 2. Environment variable
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .ok()
            .or_else(|| std::env::var("ANTHROPIC_AUTH_TOKEN").ok());

        let base_url = std::env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());

        let info_url = std::env::var("INFO_URL").ok();

        // 3. Claude Code api_key file
        if api_key.is_none() {
            if let Some(home) = dirs::home_dir() {
                let config_path = home.join(".claude").join("api_key");
                if let Ok(key) = fs::read_to_string(config_path) {
                    return (Some(key.trim().to_string()), base_url, info_url);
                }
            }
        }

        (api_key, base_url, info_url)
    }

    fn get_claude_config_dir() -> Option<PathBuf> {
        // Claude Code config directory is ~/.claude
        dirs::home_dir().map(|home| home.join(".claude"))
    }

    // 获取用户信息 API 数据
    fn fetch_user_info_api(api_key: &str, base_url: &str) -> Option<UserApiResponse> {
        // 优先使用配置的 info_url
        let (_, _, info_url) = Self::load_api_config();
        
        let url = if let Some(info_url) = info_url {
            // 如果配置了 info_url，直接使用
            info_url
        } else if base_url.starts_with("https://share-api") {
            // share-api 的情况，使用正确的端点
            "https://share.packycode.com/api/backend/users/info".to_string()
        } else if base_url.contains("packycode.com") {
            // 公交车模式
            format!("{}/api/backend/users/info", base_url)
        } else {
            // 默认处理
            format!("{}/api/backend/users/info", base_url)
        };
        
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("accept", "application/json")
            .timeout(Duration::from_secs(2))
            .send()
            .ok()?;

        if response.status().is_success() {
            response.json().ok()
        } else {
            None
        }
    }

    // 共享的 API 获取函数，带缓存
    fn fetch_user_info_cached() -> Option<UserApiResponse> {
        const CACHE_DURATION: Duration = Duration::from_secs(30); // 30秒缓存

        // 检查缓存
        if let Ok(cache) = API_CACHE.lock() {
            if let Some((cached_info, cached_time)) = cache.as_ref() {
                if cached_time.elapsed().unwrap_or(CACHE_DURATION) < CACHE_DURATION {
                    return Some(cached_info.clone());
                }
            }
        }

        // 缓存过期或不存在，重新获取
        let (api_key, base_url, _) = Self::load_api_config();
        let api_key = api_key?;
        
        // 使用统一的 backend/users/info API
        let user_info = Self::fetch_user_info_api(&api_key, &base_url)?;
        
        // 更新缓存
        if let Ok(mut cache) = API_CACHE.lock() {
            *cache = Some((user_info.clone(), SystemTime::now()));
        }
        
        Some(user_info)
    }

    fn fetch_quota(&self) -> Option<ApiQuota> {
        // 使用统一的 backend/users/info API
        if let Some(ref api_key) = self.api_key {
            if let Some(user_info) = Self::fetch_user_info_api(api_key, &self.base_url) {
                // 直接使用 API 返回的数据
                let quota = ApiQuota {
                    remaining: user_info.daily_budget_usd - user_info.daily_spent_usd,
                    total: user_info.daily_budget_usd,
                    used: user_info.daily_spent_usd,
                    timestamp: SystemTime::now(),
                };
                return Some(quota);
            }
        }
        
        // 使用缓存的 API 调用（回退方案）
        if let Some(user_info) = Self::fetch_user_info_cached() {
            let quota = ApiQuota {
                remaining: user_info.daily_budget_usd - user_info.daily_spent_usd,
                total: user_info.daily_budget_usd,
                used: user_info.daily_spent_usd,
                timestamp: SystemTime::now(),
            };
            return Some(quota);
        }

        // Fallback to standard Anthropic API
        let api_key = self.api_key.as_ref()?;
        let url = if self.base_url.contains("api.anthropic.com") {
            format!("{}/v1/dashboard/usage", self.base_url)
        } else {
            // For proxy/custom endpoints, try common patterns
            format!("{}/v1/dashboard/usage", self.base_url)
        };

        let client = reqwest::blocking::Client::new();
        let mut request = client.get(&url).timeout(Duration::from_secs(2));

        // Handle different auth header formats based on the endpoint
        if self.base_url.contains("api.anthropic.com") {
            request = request
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01");
        } else {
            // For custom endpoints, try both header formats
            request = request
                .header("Authorization", format!("Bearer {}", api_key))
                .header("x-api-key", api_key);
        }

        let response = request.send().ok()?;

        if response.status().is_success() {
            let usage: AnthropicUsageResponse = response.json().ok()?;

            let quota = ApiQuota {
                remaining: usage.remaining,
                total: usage.limit,
                used: usage.limit - usage.remaining,
                timestamp: SystemTime::now(),
            };

            Some(quota)
        } else {
            None
        }
    }

    fn format_quota(&self, quota: &ApiQuota) -> String {
        // 显示今日花费金额
        let daily_spent = quota.used;

        // Choose emoji based on spending amount
        let emoji = if daily_spent < 5.0 {
            "💚" // Green - very low spending
        } else if daily_spent < 15.0 {
            "💛" // Yellow - moderate spending
        } else if daily_spent < 30.0 {
            "🧡" // Orange - high spending
        } else {
            "❤️" // Red - very high spending
        };

        // 尝试获取排名信息和垃圾话
        let ranking_info = self.get_ranking_info();

        // 格式化显示：emoji Today: $花费 排名图标 排名数字 | 垃圾话
        if let Some((rank_display, talk, _gap_info)) = ranking_info {
            format!("{} Today: ${:.2} {} | {}", emoji, daily_spent, rank_display, talk)
        } else {
            format!("{} Today: ${:.2}", emoji, daily_spent)
        }
    }

    fn get_ranking_info(&self) -> Option<(String, String, Option<String>)> {
        // 创建一个临时的RankingSegment来获取排名信息，传入JWT token
        let ranking_segment = RankingSegment::new_with_token(true, self.jwt_token.clone());
        if let Some((rank, total)) = ranking_segment.get_current_ranking() {
            // 根据排名选择图标和颜色
            let (icon, color) = match rank {
                1 => ("🥇", "\x1b[33m"), // 金色
                2 => ("🥈", "\x1b[37m"), // 银色
                3 => ("🥉", "\x1b[31m"), // 铜色
                _ => ("📊", "\x1b[36m"), // 青色
            };

            let rank_display = format!("{} {}{}\x1b[0m", icon, color, rank);
            let trash_talk = RankingSegment::get_trash_talk_by_rank(rank, total).to_string();

            // 获取与上一名的差距
            let gap_info = if rank > 1 {
                ranking_segment.get_gap_to_previous()
            } else {
                None
            };

            Some((rank_display, trash_talk, gap_info))
        } else {
            None
        }
    }
}

impl Segment for QuotaSegment {
    fn render(&self, _input: &InputData) -> String {
        if !self.enabled || self.api_key.is_none() {
            return String::new();
        }

        // Try to fetch quota (from cache or API)
        if let Some(quota) = self.fetch_quota() {
            self.format_quota(&quota)
        } else {
            // If we can't get quota, show unknown
            "◔ Quota: N/A".to_string()
        }
    }

    fn enabled(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }
}