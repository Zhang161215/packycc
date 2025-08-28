use super::Segment;
use crate::config::InputData;
use chrono::Local;

// 时间段：显示当前本地时间，格式 HH:mm:ss
pub struct TimeSegment {
    enabled: bool,
}

impl TimeSegment {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Segment for TimeSegment {
    fn render(&self, _input: &InputData) -> String {
        if !self.enabled {
            return String::new();
        }
        let now = Local::now();
        // 使用 24 小时制 HH:mm:ss
        let time_str = now.format("%H:%M:%S").to_string();
        format!("{} {}", "\u{1F552}", time_str) // 🕒 HH:mm:ss
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

