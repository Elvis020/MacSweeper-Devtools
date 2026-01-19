// Cleanup module - safe package removal
pub mod executor;
pub mod backup;

use anyhow::Result;

pub struct CleanupPlan {
    pub packages_to_remove: Vec<String>,
    pub size_to_recover: u64,
}

impl CleanupPlan {
    pub fn new() -> Self {
        Self {
            packages_to_remove: Vec::new(),
            size_to_recover: 0,
        }
    }
}
