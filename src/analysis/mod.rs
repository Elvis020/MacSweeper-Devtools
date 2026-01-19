// Analysis module - orphan detection, dependency graphs, recommendations
pub mod orphans;
pub mod dependencies;
pub mod recommendations;

use crate::scanner::Package;

pub struct DependencyAnalysis {
    pub leaves: Vec<String>,
    pub orphans: Vec<String>,
}
