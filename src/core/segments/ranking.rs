use super::Segment;
use crate::config::InputData;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RankingSegment {
    enabled: bool,
    jwt_token: String,
}

impl RankingSegment {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            // 使用默认的JWT token，实际应该从配置中传入
            jwt_token: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3OGM2NTM3OC0xN2RhLTRkNzAtOTcyMC05ZjVhNGNkMmZhOGMiLCJpc3MiOiJma2NvZGUtYXBpIiwiYXVkIjoiZmtjb2RlLXVzZXJzIiwiZXhwIjoxNzU2NzgyMzIwLCJpYXQiOjE3NTYxNzc1MjAsIm5iZiI6MTc1NjE3NzUyMCwianRpIjoiZTAwZTVkY2EtYTgyYi00OTgwLTlkMmUtNDMwMWZkODViMzQ0IiwidXNlcl9pZCI6Ijc4YzY1Mzc4LTE3ZGEtNGQ3MC05NzIwLTlmNWE0Y2QyZmE4YyIsImVtYWlsIjoiMTA1MjYwNzQyM0BxcS5jb20iLCJ1c2VybmFtZSI6IjEwNTI2MDc0MjMiLCJ1c2VyX3R5cGUiOiJ1c2VyIiwicGVybWlzc2lvbnMiOlsiYXBpOnVzZSIsInByb2ZpbGU6cmVhZCIsInByb2ZpbGU6dXBkYXRlIiwidXNhZ2U6cmVhZCJdLCJzZXNzaW9uX2lkIjoiMjA2ZGNhNzgtYWU4MS00ODRlLWIyMTItNWYyODdmZmU2OWQ3In0.7FueQXA_3qlOJgzWQY6-gjKzvPlHw-V9f4jC7TN_U6w".to_string(),
        }
    }

    pub fn new_with_token(enabled: bool, jwt_token: Option<String>) -> Self {
        Self {
            enabled,
            // 优先使用配置中的token，然后是环境变量，最后是默认token
            jwt_token: jwt_token
                .or_else(|| std::env::var("PACKYCODE_JWT_TOKEN").ok())
                .unwrap_or_else(|| {
                    "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiI3OGM2NTM3OC0xN2RhLTRkNzAtOTcyMC05ZjVhNGNkMmZhOGMiLCJpc3MiOiJma2NvZGUtYXBpIiwiYXVkIjoiZmtjb2RlLXVzZXJzIiwiZXhwIjoxNzU2NzgyMzIwLCJpYXQiOjE3NTYxNzc1MjAsIm5iZiI6MTc1NjE3NzUyMCwianRpIjoiZTAwZTVkY2EtYTgyYi00OTgwLTlkMmUtNDMwMWZkODViMzQ0IiwidXNlcl9pZCI6Ijc4YzY1Mzc4LTE3ZGEtNGQ3MC05NzIwLTlmNWE0Y2QyZmE4YyIsImVtYWlsIjoiMTA1MjYwNzQyM0BxcS5jb20iLCJ1c2VybmFtZSI6IjEwNTI2MDc0MjMiLCJ1c2VyX3R5cGUiOiJ1c2VyIiwicGVybWlzc2lvbnMiOlsiYXBpOnVzZSIsInByb2ZpbGU6cmVhZCIsInByb2ZpbGU6dXBkYXRlIiwidXNhZ2U6cmVhZCJdLCJzZXNzaW9uX2lkIjoiMjA2ZGNhNzgtYWU4MS00ODRlLWIyMTItNWYyODdmZmU2OWQ3In0.7FueQXA_3qlOJgzWQY6-gjKzvPlHw-V9f4jC7TN_U6w".to_string()
                }),
        }
    }

    // 静态方法：根据排名获取垃圾话，供其他segment使用
    pub fn get_trash_talk_by_rank(rank: usize, total: usize) -> &'static str {
        match rank {
            1 => Self::get_first_place_talk_static(),
            2 => Self::get_second_place_talk_static(),
            3 => Self::get_third_place_talk_static(),
            _ if rank == total => Self::get_last_place_talk_static(),
            _ => Self::get_middle_place_talk_static(),
        }
    }

    // 获取当前用户的排名信息，供其他segment使用
    pub fn get_current_ranking(&self) -> Option<(usize, usize)> {
        let ranking_info = self.get_ranking_info();
        match ranking_info.status {
            RankingStatus::Success => {
                if let (Some(rank), Some(total)) = (ranking_info.current_rank, ranking_info.total_participants) {
                    Some((rank, total))
                } else {
                    None
                }
            }
            RankingStatus::Error => None,
        }
    }

    // 获取与上一名的差距
    pub fn get_gap_to_previous(&self) -> Option<String> {
        // 获取同行消费数据
        let peer_data = self.fetch_peer_spending_data()?;
        let current_user_spending = self.get_current_user_spending().unwrap_or(0.0);

        if peer_data.is_empty() {
            return None;
        }

        // 解析所有消费数据并排序
        let mut all_spending: Vec<f64> = peer_data.iter()
            .filter_map(|peer| peer.spent_usd_today.parse::<f64>().ok())
            .collect();

        // 添加当前用户消费
        all_spending.push(current_user_spending);

        // 按消费金额降序排序
        all_spending.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // 找到当前用户的位置
        let current_position = all_spending.iter()
            .position(|&spending| (spending - current_user_spending).abs() < 0.01)?;

        // 如果是第一名，不显示差距
        if current_position == 0 {
            return None;
        }

        // 获取上一名的消费金额
        let previous_spending = all_spending[current_position - 1];
        let gap = previous_spending - current_user_spending;

        // 根据差距大小选择颜色
        let color = if gap < 5.0 {
            "\x1b[32m" // 绿色 - 差距很小
        } else if gap < 15.0 {
            "\x1b[33m" // 黄色 - 差距中等
        } else if gap < 30.0 {
            "\x1b[31m" // 红色 - 差距较大
        } else {
            "\x1b[35m" // 紫色 - 差距巨大
        };

        Some(format!("(距上一名{}${:.2}\x1b[0m)", color, gap))
    }

    // 获取同行消费原始数据
    fn fetch_peer_spending_data(&self) -> Option<Vec<PeerRecord>> {
        let output = Command::new("curl")
            .args([
                "-s",
                "-H", &format!("Authorization: Bearer {}", self.jwt_token),
                "-H", "Content-Type: application/json",
                "https://share.packycode.com/api/backend/accounts/peer-spending/today"
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let response_text = String::from_utf8_lossy(&output.stdout);
        match serde_json::from_str::<PeerSpendingResponse>(&response_text) {
            Ok(response) => Some(response.peers),
            Err(_) => None,
        }
    }

    fn get_ranking_info(&self) -> RankingInfo {
        match self.fetch_ranking_data() {
            Some(data) => data,
            None => RankingInfo {
                current_rank: None,
                total_participants: None,
                status: RankingStatus::Error,
            },
        }
    }

    fn get_current_user_spending(&self) -> Option<f64> {
        // 使用curl命令请求用户信息API
        let output = Command::new("curl")
            .args([
                "-s", // 静默模式
                "-H", &format!("Authorization: Bearer {}", self.jwt_token),
                "-H", "Content-Type: application/json",
                "https://share.packycode.com/api/backend/users/info"
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let response_text = String::from_utf8_lossy(&output.stdout);

        // 解析JSON响应
        match serde_json::from_str::<UserInfoResponse>(&response_text) {
            Ok(response) => {
                response.daily_spent_usd.parse::<f64>().ok()
            }
            Err(_) => None,
        }
    }

    fn fetch_ranking_data(&self) -> Option<RankingInfo> {
        // 使用curl命令请求排名API
        let output = Command::new("curl")
            .args([
                "-s", // 静默模式
                "-H", &format!("Authorization: Bearer {}", self.jwt_token),
                "-H", "Content-Type: application/json",
                "https://share.packycode.com/api/backend/accounts/peer-spending/today"
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let response_text = String::from_utf8_lossy(&output.stdout);

        // 解析JSON响应
        match serde_json::from_str::<PeerSpendingResponse>(&response_text) {
            Ok(response) => {
                // 计算排名信息
                let total_participants = response.peers.len();

                if total_participants == 0 {
                    return Some(RankingInfo {
                        current_rank: None,
                        total_participants: Some(0),
                        status: RankingStatus::Success,
                    });
                }

                // 解析同行消费数据并排序
                let mut peer_spending: Vec<(String, f64)> = response.peers
                    .iter()
                    .filter_map(|peer| {
                        peer.spent_usd_today.parse::<f64>().ok()
                            .map(|spent| (peer.display_name.clone(), spent))
                    })
                    .collect();

                // 按消费金额降序排序
                peer_spending.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

                // 获取当前用户消费数据
                let current_user_spending = self.get_current_user_spending().unwrap_or(0.0);

                // 计算当前用户排名
                let mut current_rank = 1;
                for (_, spent) in &peer_spending {
                    if current_user_spending >= *spent {
                        break;
                    }
                    current_rank += 1;
                }

                // 总参与者数量包括当前用户
                let total_with_user = total_participants + 1;

                Some(RankingInfo {
                    current_rank: Some(current_rank),
                    total_participants: Some(total_with_user),
                    status: RankingStatus::Success,
                })
            }
            Err(_) => None,
        }
    }

    fn get_first_place_talk(&self) -> &'static str {
        Self::get_first_place_talk_static()
    }

    fn get_second_place_talk(&self) -> &'static str {
        Self::get_second_place_talk_static()
    }

    fn get_third_place_talk(&self) -> &'static str {
        Self::get_third_place_talk_static()
    }

    fn get_middle_place_talk(&self) -> &'static str {
        Self::get_middle_place_talk_static()
    }

    fn get_last_place_talk(&self) -> &'static str {
        Self::get_last_place_talk_static()
    }

    // 静态版本的垃圾话方法
    fn get_first_place_talk_static() -> &'static str {
        let talks = [
            "遥遥领先！",
            "还有谁？嗯？",
            "卷王本王了属于是",
            "这就是你们的极限？",
            "额度收割机申请出战",
            "快喊老板糊涂！",
            "绝对王者降临",
            "你们都是弟弟",
        ];
        let index = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize % talks.len();
        talks[index]
    }

    fn get_second_place_talk_static() -> &'static str {
        let talks = [
            "万年老二难受啊马飞",
            "差点意思下次一定",
            "逼死强迫症就差一步",
            "一人之下玩得挺溜啊",
            "第一你晚上睁眼睡觉",
            "顶尖高手就是你",
            "距离王座一步之遥",
            "再肝一点就登顶了",
        ];
        let index = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize % talks.len();
        talks[index]
    }

    fn get_third_place_talk_static() -> &'static str {
        let talks = [
            "中流砥柱主打陪伴",
            "比上不足比下有余",
            "佛系玩家但没完全佛",
            "完美避开所有竞争",
            "你们争你们的我吃瓜",
            "中坚力量就是你",
            "不卷不躺刚刚好",
            "稳定发挥选手",
        ];
        let index = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize % talks.len();
        talks[index]
    }

    fn get_middle_place_talk_static() -> &'static str {
        let talks = [
            "哥们你搁这养生呢？",
            "再不用额度要发霉了",
            "给榜一大哥刷存在感",
            "醒醒你的Code在哭泣",
            "订阅是捐希望工程了？",
            "潜力股还是摆烂股？",
            "别人在卷你在躺",
            "有点拉胯啊兄弟",
        ];
        let index = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize % talks.len();
        talks[index]
    }

    fn get_last_place_talk_static() -> &'static str {
        let talks = [
            "细狗，你行不行啊？",
            "年纪轻轻就开始养生啦？",
            "这福气给你你要不要",
            "垫底啦菜就多练！",
            "用量AI都要饿哭了",
            "会员是拼多多砍的吧",
            "别摸了驴都不敢歇",
            "细狗の王就是你",
            "生产队驴看了都摇头",
        ];
        let index = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as usize % talks.len();
        talks[index]
    }

    fn format_ranking_info(&self, info: &RankingInfo) -> String {
        match info.status {
            RankingStatus::Success => {
                if let (Some(rank), Some(total)) = (info.current_rank, info.total_participants) {
                    if total == 0 {
                        return "📊 \x1b[36m无排名数据\x1b[0m".to_string();
                    }

                    // 根据排名选择图标和颜色
                    let (icon, color) = match rank {
                        1 => ("🥇", "\x1b[33m"), // 金色
                        2 => ("🥈", "\x1b[37m"), // 银色
                        3 => ("🥉", "\x1b[31m"), // 铜色
                        _ => ("📊", "\x1b[36m"), // 青色
                    };

                    format!("{} {}{}\x1b[0m", icon, color, rank)
                } else {
                    "📊 \x1b[36m排名获取中\x1b[0m".to_string()
                }
            },
            RankingStatus::Error => {
                "📊 \x1b[31m排名错误\x1b[0m".to_string()
            },
        }
    }
}

impl Segment for RankingSegment {
    fn render(&self, _input: &InputData) -> String {
        if !self.enabled {
            return String::new();
        }

        let ranking_info = self.get_ranking_info();
        self.format_ranking_info(&ranking_info)
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug)]
struct RankingInfo {
    current_rank: Option<usize>,
    total_participants: Option<usize>,
    status: RankingStatus,
}

#[derive(Debug, PartialEq)]
enum RankingStatus {
    Success,
    Error,
}

// API响应结构
#[derive(Deserialize, Serialize)]
struct PeerSpendingResponse {
    account_id: String,
    date: String,
    peers: Vec<PeerRecord>,
    timezone: String,
}

#[derive(Deserialize, Serialize)]
struct PeerRecord {
    display_name: String,
    spent_usd_today: String,
    user_id: String,
}

// 用户信息API响应结构
#[derive(Deserialize, Serialize)]
struct UserInfoResponse {
    daily_spent_usd: String,
    // 其他字段可以根据需要添加
}
