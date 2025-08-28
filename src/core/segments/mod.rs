pub mod directory;
pub mod git;
pub mod model;
pub mod quota;
pub mod usage;
pub mod time;
pub mod emoji;
pub mod spinner;
pub mod network;
pub mod ranking;

use crate::config::InputData;

pub trait Segment {
    fn render(&self, input: &InputData) -> String;
    fn enabled(&self) -> bool;
}

// Re-export all segment types
pub use directory::DirectorySegment;
pub use git::GitSegment;
pub use model::ModelSegment;
pub use quota::QuotaSegment;
pub use usage::UsageSegment;
pub use time::TimeSegment;
pub use emoji::EmojiSegment;
pub use spinner::SpinnerSegment;
pub use network::NetworkSegment;
pub use ranking::RankingSegment;
