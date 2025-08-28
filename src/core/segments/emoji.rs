use super::Segment;
use crate::config::InputData;
use std::time::{SystemTime, UNIX_EPOCH};

// 末尾表情段：基于当前时间每 2 秒切换一次表情
pub struct EmojiSegment {
    enabled: bool,
}

impl EmojiSegment {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl Segment for EmojiSegment {
    fn render(&self, _input: &InputData) -> String {
        if !self.enabled {
            return String::new();
        }
        // 分组三连：每次显示同类/关联的一组，而不是随机三枚
        // 组例子：三种不同颜色的心形、猫脸三连、挥手三连等
        const GROUPS: &[&[&str; 3]] = &[
            // 心形分组
            &["❤️","🧡","💛"],
            &["💚","💙","💜"],
            &["🤎","🖤","🤍"],
            &["💖","💗","💓"],
            // 猫脸分组
            &["😺","😸","😹"],
            // 笑脸分组
            &["😀","😃","😄"],
            // 手势分组
            &["👋","🤚","✋"],
            &["✌","🤟","🤘"],
            // 猴子三连
            &["🙈","🙉","🙊"],
            // 其他
            &["🚀","🌟","🎉"],
            
            // 天气
            &["🌈","☀️","🌤️"],
           
            &["🙀","😿","😾"],
            &["😠","😡","🤬"],
        ];

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        // 每 2 秒切换到下一组
        let group_idx = ((now.as_secs() / 2) as usize) % GROUPS.len();
        format!("{} {} {}", GROUPS[group_idx][0], GROUPS[group_idx][1], GROUPS[group_idx][2])
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

