// Usage tracking module - detects when packages were last used
pub mod shell_history;
pub mod spotlight;
pub mod atime;
pub mod aggregator;

// Re-export the main aggregator function for convenience
pub use aggregator::aggregate_usage;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageSource {
    ShellHistory { count: u32, last_used: DateTime<Utc> },
    SpotlightMetadata { last_used: DateTime<Utc> },
    FileAccessTime { atime: DateTime<Utc> },
    Manual,
}

#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u32,
    pub sources: Vec<UsageSource>,
}

impl UsageInfo {
    pub fn new() -> Self {
        Self {
            last_used: None,
            usage_count: 0,
            sources: Vec::new(),
        }
    }
}
