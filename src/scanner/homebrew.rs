// Homebrew package scanner
use super::{Package, PackageSource, Scanner};
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

pub struct HomebrewScanner {
    prefix: String,
}

#[derive(Debug, Deserialize)]
struct BrewInfo {
    formulae: Vec<BrewFormula>,
    casks: Vec<BrewCask>,
}

#[derive(Debug, Deserialize)]
struct BrewFormula {
    name: String,
    #[serde(default)]
    desc: Option<String>,
    versions: BrewVersions,
    #[serde(default)]
    installed: Vec<BrewInstalled>,
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BrewCask {
    token: String,
    #[serde(default)]
    desc: Option<String>,
    version: String,
    #[serde(default)]
    installed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BrewVersions {
    stable: String,
}

#[derive(Debug, Deserialize)]
struct BrewInstalled {
    version: String,
    #[serde(default)]
    time: Option<i64>,
    #[serde(default)]
    runtime_dependencies: Vec<BrewDependency>,
}

#[derive(Debug, Deserialize)]
struct BrewDependency {
    full_name: String,
}

impl HomebrewScanner {
    pub fn new() -> Self {
        // Detect Homebrew prefix (Apple Silicon vs Intel)
        let prefix = Self::get_brew_prefix().unwrap_or_else(|_| "/opt/homebrew".to_string());
        Self { prefix }
    }

    fn get_brew_prefix() -> Result<String> {
        let output = Command::new("brew")
            .args(["--prefix"])
            .output()
            .context("Failed to run brew --prefix")?;

        if !output.status.success() {
            anyhow::bail!("brew --prefix failed");
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn get_installed_info(&self) -> Result<BrewInfo> {
        let output = Command::new("brew")
            .args(["info", "--json=v2", "--installed"])
            .output()
            .context("Failed to run brew info --json=v2 --installed")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("brew info failed: {}", stderr);
        }

        let json = String::from_utf8(output.stdout)?;
        let info: BrewInfo = serde_json::from_str(&json)
            .context("Failed to parse brew info JSON")?;

        Ok(info)
    }

    fn calculate_formula_size(&self, name: &str) -> Option<u64> {
        // Calculate size of formula in Cellar
        let cellar_path = PathBuf::from(&self.prefix).join("Cellar").join(name);
        crate::utils::size::calculate_directory_size(&cellar_path).ok()
    }

    fn find_formula_binary(&self, name: &str) -> Option<PathBuf> {
        // Check common binary path
        let bin_path = PathBuf::from(&self.prefix).join("bin").join(name);
        if bin_path.exists() {
            Some(bin_path)
        } else {
            None
        }
    }

    fn scan_formulae(&self) -> Result<Vec<Package>> {
        let info = self.get_installed_info()?;
        let mut packages = Vec::new();

        for formula in info.formulae {
            // Use the first installed version
            let installed = formula.installed.first();

            let version = installed
                .map(|i| i.version.clone())
                .or_else(|| Some(formula.versions.stable.clone()));

            let install_date = installed
                .and_then(|i| i.time)
                .and_then(|ts| {
                    Utc.timestamp_opt(ts, 0).single()
                });

            let dependencies: Vec<String> = installed
                .map(|i| {
                    i.runtime_dependencies
                        .iter()
                        .map(|d| d.full_name.clone())
                        .collect()
                })
                .unwrap_or_else(|| formula.dependencies.clone());

            let mut package = Package::new(formula.name.clone(), PackageSource::Homebrew);
            package.version = version;
            package.install_date = install_date;
            package.size_bytes = self.calculate_formula_size(&formula.name);
            package.binary_path = self.find_formula_binary(&formula.name);
            package.dependencies = dependencies;
            package.is_dependency = false; // Will be determined later

            packages.push(package);
        }

        Ok(packages)
    }

    fn scan_casks(&self) -> Result<Vec<Package>> {
        let info = self.get_installed_info()?;
        let mut packages = Vec::new();

        for cask in info.casks {
            let mut package = Package::new(cask.token.clone(), PackageSource::HomebrewCask);
            package.version = Some(cask.version);
            // Note: Cask install time is harder to determine from JSON
            // We could parse the cask directory metadata if needed

            // Casks typically install to /Applications
            let app_path = PathBuf::from("/Applications")
                .join(format!("{}.app", Self::guess_app_name(&cask.token)));

            if app_path.exists() {
                package.binary_path = Some(app_path.clone());
                package.size_bytes = crate::utils::size::calculate_directory_size(&app_path).ok();
            }

            packages.push(package);
        }

        Ok(packages)
    }

    fn guess_app_name(token: &str) -> String {
        // Convert cask token to likely app name
        // e.g., "visual-studio-code" -> "Visual Studio Code"
        token
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Scanner for HomebrewScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();

        // Scan formulae
        match self.scan_formulae() {
            Ok(mut formulae) => packages.append(&mut formulae),
            Err(e) => eprintln!("Warning: Failed to scan Homebrew formulae: {}", e),
        }

        // Scan casks
        match self.scan_casks() {
            Ok(mut casks) => packages.append(&mut casks),
            Err(e) => eprintln!("Warning: Failed to scan Homebrew casks: {}", e),
        }

        Ok(packages)
    }

    fn is_available(&self) -> bool {
        which::which("brew").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_app_name() {
        assert_eq!(HomebrewScanner::guess_app_name("visual-studio-code"), "Visual Studio Code");
        assert_eq!(HomebrewScanner::guess_app_name("docker"), "Docker");
        assert_eq!(HomebrewScanner::guess_app_name("alt-tab"), "Alt Tab");
    }

    #[test]
    fn test_scanner_available() {
        let scanner = HomebrewScanner::new();
        // This test will pass if brew is installed
        println!("Homebrew available: {}", scanner.is_available());
    }
}
