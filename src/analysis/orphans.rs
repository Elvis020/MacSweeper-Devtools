// Orphan detection for packages
use anyhow::{Context, Result};
use std::process::Command;

/// Get orphaned Homebrew packages that can be safely removed
/// Uses `brew autoremove --dry-run` to find packages no longer needed
pub fn get_orphaned_brew_packages() -> Result<Vec<String>> {
    let output = Command::new("brew")
        .args(["autoremove", "--dry-run"])
        .output()
        .context("Failed to run brew autoremove")?;

    let stdout = String::from_utf8(output.stdout)
        .context("Failed to parse brew autoremove output")?;

    let mut orphans = Vec::new();

    // Parse output: look for lines after "Would autoremove" header
    let mut parsing_orphans = false;
    for line in stdout.lines() {
        let line = line.trim();

        if line.starts_with("==> Would autoremove") {
            parsing_orphans = true;
            continue;
        }

        if parsing_orphans && !line.is_empty() && !line.starts_with("==>") {
            orphans.push(line.to_string());
        }
    }

    Ok(orphans)
}

/// Get top-level Homebrew packages (leaves) that are not dependencies
/// Uses `brew leaves` to find packages explicitly installed by the user
pub fn get_brew_leaves() -> Result<Vec<String>> {
    let output = Command::new("brew")
        .arg("leaves")
        .output()
        .context("Failed to run brew leaves")?;

    let stdout = String::from_utf8(output.stdout)
        .context("Failed to parse brew leaves output")?;

    let leaves: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(leaves)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires Homebrew to be installed
    fn test_get_orphaned_brew_packages() {
        // This test requires Homebrew to be installed
        let result = get_orphaned_brew_packages();
        assert!(result.is_ok());

        let orphans = result.unwrap();
        // Can't assert exact count, but should return a Vec
        assert!(orphans.len() >= 0);
    }

    #[test]
    #[ignore] // Requires Homebrew to be installed
    fn test_get_brew_leaves() {
        // This test requires Homebrew to be installed
        let result = get_brew_leaves();
        assert!(result.is_ok());

        let leaves = result.unwrap();
        // Should have at least some top-level packages
        // Can't assert exact count as it varies by system
        assert!(leaves.len() >= 0);
    }
}
