// macOS Applications scanner
use super::{Package, PackageSource, Scanner};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

pub struct ApplicationsScanner {
    scan_paths: Vec<PathBuf>,
}

impl ApplicationsScanner {
    pub fn new() -> Self {
        let mut scan_paths = vec![PathBuf::from("/Applications")];

        // Also scan user Applications if it exists
        if let Some(home) = dirs::home_dir() {
            let user_apps = home.join("Applications");
            if user_apps.exists() {
                scan_paths.push(user_apps);
            }
        }

        Self { scan_paths }
    }

    fn get_app_version(&self, app_path: &PathBuf) -> Option<String> {
        // Try to read version from Info.plist
        let plist_path = app_path.join("Contents/Info.plist");
        if !plist_path.exists() {
            return None;
        }

        // Use defaults command to read plist
        let output = Command::new("defaults")
            .args(["read", &plist_path.to_string_lossy(), "CFBundleShortVersionString"])
            .output()
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8(output.stdout).ok()?;
            return Some(version.trim().to_string());
        }

        // Try alternative key
        let output = Command::new("defaults")
            .args(["read", &plist_path.to_string_lossy(), "CFBundleVersion"])
            .output()
            .ok()?;

        if output.status.success() {
            let version = String::from_utf8(output.stdout).ok()?;
            return Some(version.trim().to_string());
        }

        None
    }

    fn get_app_name(&self, app_path: &PathBuf) -> Option<String> {
        // Get the app name from the .app bundle name
        app_path
            .file_stem()
            .map(|name| name.to_string_lossy().to_string())
    }
}

impl Scanner for ApplicationsScanner {
    fn scan(&self) -> Result<Vec<Package>> {
        let mut packages = Vec::new();

        for scan_path in &self.scan_paths {
            if !scan_path.exists() {
                continue;
            }

            // Read directory entries (non-recursive, only top level .app bundles)
            match fs::read_dir(scan_path) {
                Ok(entries) => {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();

                        // Only process .app bundles
                        if let Some(ext) = path.extension() {
                            if ext == "app" {
                                if let Some(name) = self.get_app_name(&path) {
                                    let mut package = Package::new(name, PackageSource::Applications);
                                    package.version = self.get_app_version(&path);
                                    package.binary_path = Some(path.clone());

                                    // Calculate size
                                    package.size_bytes = crate::utils::size::calculate_directory_size(&path).ok();

                                    packages.push(package);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to scan {}: {}", scan_path.display(), e);
                }
            }
        }

        Ok(packages)
    }

    fn is_available(&self) -> bool {
        // Applications scanning is always available on macOS
        self.scan_paths.iter().any(|p| p.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_available() {
        let scanner = ApplicationsScanner::new();
        println!("Applications scanner available: {}", scanner.is_available());
    }

    #[test]
    #[ignore] // Run manually
    fn test_scan_applications() {
        let scanner = ApplicationsScanner::new();
        let packages = scanner.scan().unwrap();
        println!("Found {} applications", packages.len());
        for pkg in packages.iter().take(10) {
            println!("  - {} ({})", pkg.name, pkg.version.as_deref().unwrap_or("?"));
        }
    }
}
