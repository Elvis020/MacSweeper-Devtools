// pip/pip3/pipx package scanner
use super::{Package, PackageSource, Scanner};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

pub struct PipScanner;

#[derive(Debug, Deserialize)]
struct PipPackage {
    name: String,
    version: String,
}

impl PipScanner {
    pub fn new() -> Self {
        Self
    }

    fn scan_pip_executable(&self, pip_cmd: &str) -> Result<Vec<Package>> {
        let output = Command::new(pip_cmd)
            .args(["list", "--format=json"])
            .output()
            .context(format!("Failed to run {} list", pip_cmd))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{} list failed: {}", pip_cmd, stderr);
        }

        let json = String::from_utf8(output.stdout)
            .context(format!("Failed to parse {} output as UTF-8", pip_cmd))?;

        let pip_packages: Vec<PipPackage> = serde_json::from_str(&json)
            .context(format!("Failed to parse {} list JSON", pip_cmd))?;

        let mut packages = Vec::new();

        for pip_pkg in pip_packages {
            // Skip pip and setuptools as they're base packages
            if pip_pkg.name == "pip" || pip_pkg.name == "setuptools" || pip_pkg.name == "wheel" {
                continue;
            }

            let mut package = Package::new(pip_pkg.name.clone(), PackageSource::Pip);
            package.version = Some(pip_pkg.version);

            // Try to find the binary path (many Python packages install console scripts)
            package.binary_path = self.find_pip_binary(&pip_pkg.name);

            packages.push(package);
        }

        Ok(packages)
    }

    fn scan_pipx(&self) -> Result<Vec<Package>> {
        let output = Command::new("pipx")
            .args(["list", "--short"])
            .output()
            .context("Failed to run pipx list")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Failed to parse pipx output as UTF-8")?;

        let mut packages = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let name = parts[0].to_string();
            let version = parts.get(1).map(|v| v.to_string());

            let mut package = Package::new(name.clone(), PackageSource::Pipx);
            package.version = version;
            package.binary_path = which::which(&name).ok();

            packages.push(package);
        }

        Ok(packages)
    }

    fn find_pip_binary(&self, package_name: &str) -> Option<std::path::PathBuf> {
        // Convert package name to potential binary names
        // e.g., "some-package" -> "some-package", "some_package"
        let variants = vec![
            package_name.to_string(),
            package_name.replace('-', "_"),
            package_name.replace('_', "-"),
        ];

        for variant in variants {
            if let Ok(path) = which::which(&variant) {
                return Some(path);
            }
        }

        None
    }
}

impl Scanner for PipScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        let mut all_packages = Vec::new();

        // Try pip3 first (preferred on macOS)
        if which::which("pip3").is_ok() {
            match self.scan_pip_executable("pip3") {
                Ok(mut packages) => all_packages.append(&mut packages),
                Err(e) => eprintln!("Warning: Failed to scan pip3: {}", e),
            }
        }
        // Try pip if pip3 not available
        else if which::which("pip").is_ok() {
            match self.scan_pip_executable("pip") {
                Ok(mut packages) => all_packages.append(&mut packages),
                Err(e) => eprintln!("Warning: Failed to scan pip: {}", e),
            }
        }

        // Try pipx if available
        if which::which("pipx").is_ok() {
            match self.scan_pipx() {
                Ok(mut packages) => all_packages.append(&mut packages),
                Err(e) => eprintln!("Warning: Failed to scan pipx: {}", e),
            }
        }

        Ok(all_packages)
    }

    fn is_available(&self) -> bool {
        which::which("pip").is_ok() || which::which("pip3").is_ok() || which::which("pipx").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_available() {
        let scanner = PipScanner::new();
        println!("pip/pip3/pipx available: {}", scanner.is_available());
    }

    #[test]
    #[ignore] // Run manually as it requires pip to be installed
    fn test_scan_pip_packages() {
        let scanner = PipScanner::new();
        if scanner.is_available() {
            let packages = scanner.scan().unwrap();
            println!("Found {} pip packages", packages.len());
            for pkg in packages.iter().take(5) {
                println!("  - {} ({})", pkg.name, pkg.version.as_deref().unwrap_or("?"));
            }
        }
    }
}
