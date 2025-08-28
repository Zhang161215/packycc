use crate::config::{Config, InputData};
use crate::core::segments::{
    DirectorySegment, GitSegment, ModelSegment, QuotaSegment, Segment, UsageSegment, SpinnerSegment, NetworkSegment, RankingSegment,
};

pub struct StatusLineGenerator {
    config: Config,
}

impl StatusLineGenerator {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn generate(&self, input: &InputData) -> String {
        // 正常的statusline生成，不包含测速信息
        self.generate_normal_statusline(input)
    }

    fn generate_normal_statusline(&self, input: &InputData) -> String {
        let mut segments: Vec<String> = Vec::new();

        // Assemble segments with proper colors
        // Spinner at the very beginning (bright bold green like directory + white background + single space padding)
        if self.config.segments.spinner {
            let spinner = SpinnerSegment::new(true);
            let frame = spinner.render(input);
            if !frame.is_empty() {
                segments.push(format!("\x1b[1;32;47m {} \x1b[0m", frame)); // 粗体亮绿色+白背景+前后各1个空格
            }
        }

        if self.config.segments.model {
            let model_segment = ModelSegment::new(true);
            let content = model_segment.render(input);
            segments.push(format!("\x1b[1;36m{}\x1b[0m", content));
        }

        if self.config.segments.directory {
            let dir_segment = DirectorySegment::new(true);
            let dir_name = dir_segment.render(input);
            // 使用 Emoji 图标 + 绿字目录名
            segments.push(format!("\x1b[1;33m📁\x1b[0m \x1b[1;32m{}\x1b[0m", dir_name));
        }

        if self.config.segments.git {
            let git_segment = GitSegment::new(true);
            let git_output = git_segment.render(input);
            if !git_output.is_empty() {
                segments.push(format!("\x1b[1;34m{}\x1b[0m", git_output));
            }
        }

        if self.config.segments.usage {
            let usage_segment = UsageSegment::new(true);
            let content = usage_segment.render(input);
            segments.push(format!("\x1b[1;35m{}\x1b[0m", content));
        }

        if self.config.segments.quota {
            let quota_segment = QuotaSegment::new_with_config(true, self.config.jwt_token.clone());
            let content = quota_segment.render(input);
            if !content.is_empty() {
                segments.push(format!("\x1b[1;93m{}\x1b[0m", content));
            }
        }

        if self.config.segments.network {
            let network_segment = NetworkSegment::new(true);
            let content = network_segment.render(input);
            if !content.is_empty() {
                segments.push(format!("\x1b[1;94m{}\x1b[0m", content)); // 亮蓝色
            }
        }

        // 排名功能已集成到quota segment中，不再单独显示

        // 末尾表情：每 2 秒切换一枚（基于当前时间计算）
        if self.config.segments.emoji {
            let emoji_segment = crate::core::segments::EmojiSegment::new(true);
            let content = emoji_segment.render(input);
            if !content.is_empty() {
                segments.push(content);
            }
        }

        // Join segments with white separator
        segments.join("\x1b[37m | \x1b[0m")
    }
}
