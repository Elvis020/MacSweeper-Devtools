// npm global package scanner
use super::{Package, PackageSource, Scanner};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

pub struct NpmScanner;

#[derive(Debug, Deserialize)]
struct NpmList {
    dependencies: Option<HashMap<String, NpmPackage>>,
}

#[derive(Debug, Deserialize)]
struct NpmPackage {
    version: String,
    #[serde(default)]
    overridden: bool,
}

impl NpmScanner {
    pub fn new() -> Self {
        Self
    }

    fn get_global_packages(&self) -> Result<Vec<Package>> {
        let output = Command::new("npm")
            .args(["list", "-g", "--depth=0", "--json"])
            .output()
            .context("Failed to run npm list -g")?;

        if !output.status.success() {
            // npm may return non-zero even on success with warnings
            // So we'll try to parse anyway
        }

        let json = String::from_utf8(output.stdout)
            .context("Failed to parse npm output as UTF-8")?;

        let list: NpmList = serde_json::from_str(&json)
            .context("Failed to parse npm list JSON")?;

        let mut packages = Vec::new();

        if let Some(deps) = list.dependencies {
            for (name, pkg_info) in deps {
                // Skip npm itself as it's a special package
                if name == "npm" {
                    continue;
                }

                let mut package = Package::new(name.clone(), PackageSource::Npm);
                package.version = Some(pkg_info.version);

                // Try to find the binary path
                package.binary_path = self.find_npm_binary(&name);

                // Try to calculate size
                if let Some(ref bin_path) = package.binary_path {
                    if let Some(parent) = bin_path.parent() {
                        if let Some(node_modules) = parent.parent() {
                            let pkg_path = node_modules.join("lib/node_modules").join(&name);
                            package.size_bytes = crate::utils::size::calculate_directory_size(&pkg_path).ok();
                        }
                    }
                }

                packages.push(package);
            }
        }

        Ok(packages)
    }

    fn find_npm_binary(&self, package_name: &str) -> Option<std::path::PathBuf> {
        // Try to find the binary using which
        if let Ok(path) = which::which(package_name) {
            return Some(path);
        }

        // Also check common patterns for package names with @org/package format
        if package_name.contains('/') {
            // Extract the package name part after the slash
            if let Some(bin_name) = package_name.split('/').last() {
                if let Ok(path) = which::which(bin_name) {
                    return Some(path);
                }
            }
        }

        None
    }
}

impl Scanner for NpmScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        self.get_global_packages()
    }

    fn is_available(&self) -> bool {
        which::which("npm").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_available() {
        let scanner = NpmScanner::new();
        println!("npm available: {}", scanner.is_available());
    }

    #[test]
    #[ignore] // Run this manually as it requires npm to be installed
    fn test_scan_npm_packages() {
        let scanner = NpmScanner::new();
        if scanner.is_available() {
            let packages = scanner.scan().unwrap();
            println!("Found {} npm packages", packages.len());
            for pkg in packages.iter().take(5) {
                println!("  - {} ({})", pkg.name, pkg.version.as_deref().unwrap_or("?"));
            }
        }
    }
}
