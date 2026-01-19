// Backup and undo support for cleanup operations
use anyhow::{Context, Result};
use crate::scanner::{Package, PackageSource};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupManifest {
    pub backup_id: String,
    pub created_at: String,
    pub packages: Vec<BackupPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BackupPackage {
    pub name: String,
    pub source: String,
    pub version: Option<String>,
    pub binary_path: Option<String>,
    pub size_bytes: Option<u64>,
}

/// Get the backup directory path
fn get_backup_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let backup_dir = home.join("Library/Application Support/macsweep/backups");

    if !backup_dir.exists() {
        fs::create_dir_all(&backup_dir)?;
    }

    Ok(backup_dir)
}

/// Create a backup manifest before removing packages
pub fn create_backup(packages: &[Package]) -> Result<String> {
    let backup_id = format!("cleanup_{}", Utc::now().format("%Y%m%d_%H%M%S"));

    let backup_packages: Vec<BackupPackage> = packages.iter().map(|p| {
        BackupPackage {
            name: p.name.clone(),
            source: format!("{:?}", p.source),
            version: p.version.clone(),
            binary_path: p.binary_path.as_ref().map(|pb| pb.to_string_lossy().to_string()),
            size_bytes: p.size_bytes,
        }
    }).collect();

    let manifest = BackupManifest {
        backup_id: backup_id.clone(),
        created_at: Utc::now().to_rfc3339(),
        packages: backup_packages,
    };

    let backup_dir = get_backup_dir()?;
    let manifest_path = backup_dir.join(format!("{}.json", backup_id));

    let json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, json)?;

    println!("  📋 Backup created: {}", backup_id);
    println!("     Manifest: {}", manifest_path.display());

    Ok(manifest_path.to_string_lossy().to_string())
}

/// Restore packages from a backup manifest
pub fn restore_backup(backup_id: &str) -> Result<()> {
    let backup_dir = get_backup_dir()?;
    let manifest_path = backup_dir.join(format!("{}.json", backup_id));

    if !manifest_path.exists() {
        anyhow::bail!("Backup not found: {}", backup_id);
    }

    let json = fs::read_to_string(&manifest_path)?;
    let manifest: BackupManifest = serde_json::from_str(&json)?;

    println!("🔄 Restoring from backup: {}", manifest.backup_id);
    println!("   Created: {}", manifest.created_at);
    println!("   Packages: {}\n", manifest.packages.len());

    let mut success_count = 0;
    let mut failed_count = 0;

    for pkg in &manifest.packages {
        print!("  Restoring {} ({})... ", pkg.name, pkg.source);

        match restore_package(pkg) {
            Ok(true) => {
                println!("✓");
                success_count += 1;
            }
            Ok(false) => {
                println!("⚠ Already installed");
                success_count += 1;
            }
            Err(e) => {
                println!("✗ {}", e);
                failed_count += 1;
            }
        }
    }

    println!("\n📊 Restore Summary:");
    println!("   Restored: {}", success_count);
    if failed_count > 0 {
        println!("   Failed: {}", failed_count);
    }

    Ok(())
}

fn restore_package(pkg: &BackupPackage) -> Result<bool> {
    let source = parse_package_source(&pkg.source);

    match source {
        PackageSource::Homebrew | PackageSource::HomebrewCask => {
            restore_homebrew_package(&pkg.name)
        }
        PackageSource::Npm => {
            restore_npm_package(&pkg.name)
        }
        PackageSource::Pip | PackageSource::Pipx => {
            restore_pip_package(&pkg.name, &source)
        }
        PackageSource::Cargo => {
            restore_cargo_package(&pkg.name)
        }
        PackageSource::Applications => {
            // Applications can't be auto-restored - they were moved to trash
            println!("(check Trash)");
            Ok(false)
        }
        _ => {
            anyhow::bail!("Cannot restore packages from source: {:?}", source)
        }
    }
}

fn restore_homebrew_package(name: &str) -> Result<bool> {
    let output = Command::new("brew")
        .args(["install", name])
        .output()
        .context("Failed to execute brew install")?;

    Ok(output.status.success())
}

fn restore_npm_package(name: &str) -> Result<bool> {
    let output = Command::new("npm")
        .args(["install", "-g", name])
        .output()
        .context("Failed to execute npm install")?;

    Ok(output.status.success())
}

fn restore_pip_package(name: &str, source: &PackageSource) -> Result<bool> {
    let command = match source {
        PackageSource::Pipx => "pipx",
        _ => "pip3",
    };

    let output = Command::new(command)
        .args(["install", name])
        .output()
        .context(format!("Failed to execute {} install", command))?;

    Ok(output.status.success())
}

fn restore_cargo_package(name: &str) -> Result<bool> {
    let output = Command::new("cargo")
        .args(["install", name])
        .output()
        .context("Failed to execute cargo install")?;

    Ok(output.status.success())
}

fn parse_package_source(source_str: &str) -> PackageSource {
    match source_str {
        "Homebrew" => PackageSource::Homebrew,
        "HomebrewCask" => PackageSource::HomebrewCask,
        "Npm" => PackageSource::Npm,
        "Pip" => PackageSource::Pip,
        "Pipx" => PackageSource::Pipx,
        "Cargo" => PackageSource::Cargo,
        "Applications" => PackageSource::Applications,
        _ => PackageSource::Homebrew, // Default fallback
    }
}

/// List all available backups
pub fn list_backups() -> Result<Vec<String>> {
    let backup_dir = get_backup_dir()?;

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(backup_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(file_stem) = path.file_stem() {
                backups.push(file_stem.to_string_lossy().to_string());
            }
        }
    }

    backups.sort();
    backups.reverse(); // Most recent first

    Ok(backups)
}
