// Process/command execution utilities
use anyhow::{Context, Result};
use std::process::Command;

pub fn run_command(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .context(format!("Failed to execute: {} {:?}", program, args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr);
    }

    let stdout = String::from_utf8(output.stdout)
        .context("Failed to parse command output as UTF-8")?;

    Ok(stdout)
}

pub fn command_exists(program: &str) -> bool {
    which::which(program).is_ok()
}
