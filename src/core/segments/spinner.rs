use super::Segment;
use crate::config::InputData;
use std::time::{SystemTime, UNIX_EPOCH};

// Spinner 段：在状态栏最前方显示单帧“动画”，帧随时间轮换（目前为颜文字）
// 说明：真实刷新由宿主控制；本段仅基于当前时间计算帧索引
pub struct SpinnerSegment {
    enabled: bool,
    mode: &'static str, // 目前简单用常量模式，可后续接入配置
}

impl SpinnerSegment {
    pub fn new(enabled: bool) -> Self {
        Self { enabled, mode: "kaomoji" }
    }

    fn frames(&self) -> &'static [&'static str] {
        match self.mode {
            // 热门颜文字动画
            "kaomoji" => &[
                "(｡◕‿◕｡)",   // 开心
                "(＾◡＾)",      // 愉快
                "(´∀｀)",      // 高兴
                "\\(^o^)/",    // 万岁
                "(≧∇≦)",      // 兴奋
                "(◕‿◕)",      // 微笑
                "(╯✧▽✧)╯",   // 欢呼
                "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧", // 庆祝
                "(*^▽^*)",    // 害羞开心
                "(◉‿◉)",      // 眯眼笑
                "(*´∀`)~♥",   // 心
                "( ´•̥̥̥ω•̥̥̥` )", // 惊讶
                "(●´ω｀●)ゞ",
                "_(:3 ⌒ﾞ)_",
                "_:(´□`」 ∠):_",
                "▼・ᴥ・▼",
                "ʕ´• ᴥ•̥`ʔ",
                "(›´ω`‹ )"
            ],
            _ => &[
                "(｡◕‿◕｡)",   // 默认也使用颜文字
                "(＾◡＾)", 
                "(´∀｀)",
                "\\(^o^)/",
            ],
        }
    }
}

impl Segment for SpinnerSegment {
    fn render(&self, _input: &InputData) -> String {
        if !self.enabled { return String::new(); }
        let frames = self.frames();
        if frames.is_empty() { return String::new(); }
        // 以时间片选择帧：500ms 一帧（稍慢一些让颜文字更易读）
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
        let bucket = (now.as_millis() / 500) as usize;
        let idx = bucket % frames.len();
        frames[idx].to_string()
    }

    fn enabled(&self) -> bool { self.enabled }
}

