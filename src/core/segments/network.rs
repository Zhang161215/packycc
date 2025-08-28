use super::Segment;
use crate::config::InputData;
use std::process::Command;

pub struct NetworkSegment {
    enabled: bool,
    target_host: String,
}

impl NetworkSegment {
    pub fn new(enabled: bool) -> Self {
        Self { 
            enabled,
            target_host: "share.api.packycode.com".to_string(),
        }
    }

    fn get_network_info(&self) -> NetworkInfo {
        let (latency, status) = match self.ping_host(&self.target_host) {
            Some(latency) => (Some(latency), NetworkStatus::Connected),
            None => (None, NetworkStatus::Unreachable),
        };

        NetworkInfo {
            host: self.target_host.clone(),
            latency,
            status,
        }
    }

    fn ping_host(&self, host: &str) -> Option<u32> {
        // 临时解决方案：在Windows下使用TCP连接检测替代ping
        if cfg!(target_os = "windows") {
            use std::net::{TcpStream, ToSocketAddrs};
            use std::time::{Duration, Instant};
            
            // 尝试解析地址
            let addr_str = format!("{}:443", host);
            match addr_str.to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(socket_addr) = addrs.next() {
                        let start = Instant::now();
                        match TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1500)) {
                            Ok(_) => {
                                let duration = start.elapsed();
                                return Some((duration.as_millis() as u32).min(9999));
                            }
                            Err(_) => return None,
                        }
                    }
                }
                Err(_) => return None,
            }
            None
        } else {
            // Linux使用原来的ping逻辑
            let output = Command::new("ping")
                .args(["-c", "1", "-W", "3", host])
                .output()
                .ok()?;
                
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                // Linux/Unix格式
                for line in output_str.lines() {
                    if line.contains("time=") {
                        if let Some(time_part) = line.split("time=").nth(1) {
                            if let Some(ms_part) = time_part.split(" ms").next() {
                                if let Ok(latency) = ms_part.parse::<f32>() {
                                    return Some((latency as u32).min(9999));
                                }
                            }
                        }
                    }
                }
            }
            None
        }
    }

    fn format_network_info(&self, info: &NetworkInfo) -> String {
        match info.status {
            NetworkStatus::Connected => {
                let latency_display = info.latency
                    .map(|ms| {
                        // 根据延迟给数值着色
                        if ms < 100 {
                            format!("\x1b[32m{}ms\x1b[0m", ms) // 绿色
                        } else if ms < 300 {
                            format!("\x1b[33m{}ms\x1b[0m", ms) // 黄色
                        } else {
                            format!("\x1b[31m{}ms\x1b[0m", ms) // 红色
                        }
                    })
                    .unwrap_or_else(|| "N/A".to_string());

                // 根据延迟选择图标
                let icon = info.latency.map(|ms| {
                    if ms < 100 {
                        "🟩" // 绿色方块 - 低延迟
                    } else if ms < 300 {
                        "🟨" // 黄色方块 - 中等延迟
                    } else {
                        "🟥" // 红色方块 - 高延迟
                    }
                }).unwrap_or("🟦"); // 蓝色方块 - 未知延迟

                // 只显示图标和延迟，不显示域名
                format!("{} {}", icon, latency_display)
            },
            NetworkStatus::Unreachable => {
                format!("🟥 \x1b[31mUnreachable\x1b[0m")
            },
        }
    }
}

impl Segment for NetworkSegment {
    fn render(&self, _input: &InputData) -> String {
        if !self.enabled {
            return String::new();
        }

        let network_info = self.get_network_info();
        self.format_network_info(&network_info)
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug)]
struct NetworkInfo {
    host: String,
    latency: Option<u32>, // 延迟(毫秒)
    status: NetworkStatus,
}

#[derive(Debug, PartialEq)]
enum NetworkStatus {
    Connected,     // 已连接且可达
    Unreachable,   // 不可达
}
