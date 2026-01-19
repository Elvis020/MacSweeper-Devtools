// Execute package removal commands
use anyhow::{Context, Result};
use std::process::Command;
use crate::scanner::{Package, PackageSource};

pub fn remove_package(package: &Package, dry_run: bool) -> Result<bool> {
    if dry_run {
        // In dry-run mode, silently succeed (progress bar shows package name)
        return Ok(true);
    }

    // Progress bar shows the package name being removed

    match package.source {
        PackageSource::Homebrew | PackageSource::HomebrewCask => {
            remove_homebrew_package(&package.name)
        }
        PackageSource::Npm => {
            remove_npm_package(&package.name)
        }
        PackageSource::Pip | PackageSource::Pipx => {
            remove_pip_package(&package.name, &package.source)
        }
        PackageSource::Cargo => {
            remove_cargo_package(&package.name)
        }
        PackageSource::Applications => {
            remove_application(package)
        }
        _ => {
            eprintln!("  ⚠️  Cannot remove package from source: {:?}", package.source);
            Ok(false)
        }
    }
}

fn remove_homebrew_package(name: &str) -> Result<bool> {
    let output = Command::new("brew")
        .args(["uninstall", name])
        .output()
        .context("Failed to execute brew uninstall")?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("    ✗ Failed to remove {}: {}", name, stderr.trim());
        Ok(false)
    }
}

fn remove_npm_package(name: &str) -> Result<bool> {
    let output = Command::new("npm")
        .args(["uninstall", "-g", name])
        .output()
        .context("Failed to execute npm uninstall")?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("    ✗ Failed to remove {}: {}", name, stderr.trim());
        Ok(false)
    }
}

fn remove_pip_package(name: &str, source: &PackageSource) -> Result<bool> {
    let command = match source {
        PackageSource::Pipx => "pipx",
        _ => "pip3",
    };

    let output = Command::new(command)
        .args(["uninstall", "-y", name])
        .output()
        .context(format!("Failed to execute {} uninstall", command))?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("    ✗ Failed to remove {}: {}", name, stderr.trim());
        Ok(false)
    }
}

fn remove_cargo_package(name: &str) -> Result<bool> {
    let output = Command::new("cargo")
        .args(["uninstall", name])
        .output()
        .context("Failed to execute cargo uninstall")?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("    ✗ Failed to remove {}: {}", name, stderr.trim());
        Ok(false)
    }
}

fn remove_application(package: &Package) -> Result<bool> {
    if let Some(ref path) = package.binary_path {
        // Move to trash instead of deleting directly (safer)
        let output = Command::new("osascript")
            .args([
                "-e",
                &format!("tell application \"Finder\" to delete POSIX file \"{}\"", path.display())
            ])
            .output()
            .context("Failed to move application to trash")?;

        if output.status.success() {
            Ok(true)
        } else {
            eprintln!("    ✗ Failed to move {} to trash", package.name);
            Ok(false)
        }
    } else {
        eprintln!("    ✗ No binary path found for {}", package.name);
        Ok(false)
    }
}

fn format_source(source: &PackageSource) -> String {
    match source {
        PackageSource::Homebrew => "Homebrew".to_string(),
        PackageSource::HomebrewCask => "Homebrew Cask".to_string(),
        PackageSource::Npm => "npm".to_string(),
        PackageSource::Pip => "pip".to_string(),
        PackageSource::Pipx => "pipx".to_string(),
        PackageSource::Cargo => "cargo".to_string(),
        PackageSource::Applications => "Applications".to_string(),
        _ => format!("{:?}", source),
    }
}
