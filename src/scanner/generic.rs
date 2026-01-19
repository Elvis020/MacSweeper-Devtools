// Generic binary scanner for /usr/local/bin, ~/.local/bin, etc.
use super::{Package, PackageSource, Scanner};
use anyhow::Result;

pub struct GenericBinaryScanner {
    paths: Vec<String>,
}

impl GenericBinaryScanner {
    pub fn new(paths: Vec<String>) -> Self {
        Self { paths }
    }
}

impl Scanner for GenericBinaryScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();

        // TODO: Scan specified directories for executable files
        // TODO: Create Package structs for each binary

        Ok(packages)
    }

    fn is_available(&self) -> bool {
        // Always available
        true
    }
}
