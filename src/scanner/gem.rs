// Ruby gems scanner
use super::{Package, PackageSource, Scanner};
use anyhow::Result;

pub struct GemScanner;

impl GemScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Scanner for GemScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();

        // TODO: Run `gem list` to get installed gems
        // TODO: Parse output and create Package structs

        Ok(packages)
    }

    fn is_available(&self) -> bool {
        which::which("gem").is_ok()
    }
}
