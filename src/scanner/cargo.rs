// Cargo binaries scanner
use super::{Package, PackageSource, Scanner};
use anyhow::{Context, Result};
use regex::Regex;
use lazy_static::lazy_static;
use std::fs;
use std::process::Command;

pub struct CargoScanner;

lazy_static! {
    static ref CARGO_INSTALL_RE: Regex = Regex::new(r"^(\S+)\s+v([0-9.]+):").unwrap();
}

impl CargoScanner {
    pub fn new() -> Self {
        Self
    }

    fn scan_cargo_install_list(&self) -> Result<Vec<Package>> {
        let output = Command::new("cargo")
            .args(["install", "--list"])
            .output()
            .context("Failed to run cargo install --list")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8(output.stdout)
            .context("Failed to parse cargo output as UTF-8")?;

        let mut packages = Vec::new();

        for line in stdout.lines() {
            if let Some(caps) = CARGO_INSTALL_RE.captures(line) {
                let name = caps[1].to_string();
                let version = caps[2].to_string();

                let mut package = Package::new(name.clone(), PackageSource::Cargo);
                package.version = Some(version);

                // Try to find the binary
                package.binary_path = self.find_cargo_binary(&name);

                packages.push(package);
            }
        }

        Ok(packages)
    }

    fn scan_cargo_bin_directory(&self) -> Result<Vec<Package>> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        let bin_dir = home.join(".cargo/bin");

        if !bin_dir.exists() {
            return Ok(Vec::new());
        }

        let mut packages = Vec::new();

        for entry in fs::read_dir(&bin_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip symlinks (rustup creates many symlinks)
            if path.is_symlink() {
                continue;
            }

            // Skip if not executable
            if !is_executable(&path) {
                continue;
            }

            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy().to_string();

                // Skip known rustup binaries
                if name_str.starts_with("rust") || name_str.starts_with("cargo") || name_str == "rustup" {
                    continue;
                }

                let mut package = Package::new(name_str.clone(), PackageSource::Cargo);
                package.binary_path = Some(path.clone());

                // Try to get version by running --version
                package.version = get_binary_version(&path);

                packages.push(package);
            }
        }

        Ok(packages)
    }

    fn find_cargo_binary(&self, package_name: &str) -> Option<std::path::PathBuf> {
        let home = dirs::home_dir()?;
        let bin_path = home.join(".cargo/bin").join(package_name);

        if bin_path.exists() {
            Some(bin_path)
        } else {
            which::which(package_name).ok()
        }
    }
}

impl Scanner for CargoScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        let mut all_packages = Vec::new();

        // First try cargo install --list (more reliable for version info)
        match self.scan_cargo_install_list() {
            Ok(mut packages) => all_packages.append(&mut packages),
            Err(e) => eprintln!("Warning: Failed to scan cargo install list: {}", e),
        }

        // If cargo install --list returned nothing, scan the bin directory
        if all_packages.is_empty() {
            match self.scan_cargo_bin_directory() {
                Ok(mut packages) => all_packages.append(&mut packages),
                Err(e) => eprintln!("Warning: Failed to scan cargo bin directory: {}", e),
            }
        }

        Ok(all_packages)
    }

    fn is_available(&self) -> bool {
        which::which("cargo").is_ok()
    }
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        permissions.mode() & 0o111 != 0
    } else {
        false
    }
}

#[cfg(not(unix))]
fn is_executable(_path: &std::path::Path) -> bool {
    true // On non-Unix, assume everything is potentially executable
}

fn get_binary_version(path: &std::path::Path) -> Option<String> {
    // Try running with --version flag
    let output = Command::new(path)
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).ok()?;
        // Try to extract version number from output
        // Common formats: "name 1.2.3", "name v1.2.3", "1.2.3"
        let version_re = Regex::new(r"v?(\d+\.\d+\.\d+)").ok()?;
        if let Some(caps) = version_re.captures(&stdout) {
            return Some(caps[1].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_available() {
        let scanner = CargoScanner::new();
        println!("cargo available: {}", scanner.is_available());
    }

    #[test]
    #[ignore] // Run manually
    fn test_scan_cargo_binaries() {
        let scanner = CargoScanner::new();
        if scanner.is_available() {
            let packages = scanner.scan().unwrap();
            println!("Found {} cargo binaries", packages.len());
            for pkg in packages.iter().take(5) {
                println!("  - {} ({})", pkg.name, pkg.version.as_deref().unwrap_or("?"));
            }
        }
    }
}
