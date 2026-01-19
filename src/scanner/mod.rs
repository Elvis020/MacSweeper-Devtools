// Package scanner module - detects packages from various sources
pub mod homebrew;
pub mod npm;
pub mod pip;
pub mod cargo;
pub mod applications;
pub mod gem;
pub mod generic;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PackageSource {
    Homebrew,
    HomebrewCask,
    MacAppStore,
    Npm,
    Pip,
    Pipx,
    Cargo,
    Gem,
    Go,
    Composer,
    Applications,
    LocalBin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: Option<String>,
    pub source: PackageSource,
    pub install_date: Option<DateTime<Utc>>,
    pub size_bytes: Option<u64>,
    pub binary_path: Option<PathBuf>,
    pub is_dependency: bool,
    pub dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u32,
}

impl Package {
    pub fn new(name: String, source: PackageSource) -> Self {
        Self {
            name,
            version: None,
            source,
            install_date: None,
            size_bytes: None,
            binary_path: None,
            is_dependency: false,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            last_used: None,
            usage_count: 0,
        }
    }
}

/// Trait for package scanners
pub trait Scanner {
    fn scan(&self) -> anyhow::Result<Vec<Package>>;
    fn is_available(&self) -> bool;
}
