// Shell history parser (zsh, bash, fish)
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: Option<DateTime<Utc>>,
}

impl HistoryEntry {
    /// Extract the base command (first word) from the full command
    pub fn base_command(&self) -> Option<String> {
        self.command
            .split_whitespace()
            .next()
            .map(|s| s.to_string())
    }

    /// Check if this command invokes a specific binary/package
    pub fn invokes_binary(&self, binary_name: &str) -> bool {
        let cmd = self.command.to_lowercase();
        let bin = binary_name.to_lowercase();

        // Check if the command starts with the binary name
        if cmd.starts_with(&bin) {
            return true;
        }

        // Check if it's used in a pipe or chain
        let words: Vec<&str> = cmd.split_whitespace().collect();
        words.iter().any(|w| {
            // Remove common prefixes
            let word = w.trim_start_matches("sudo");
            word == bin || word.starts_with(&format!("{}/", bin))
        })
    }
}

lazy_static! {
    static ref ZSH_HISTORY_RE: Regex = Regex::new(r"^: (\d+):0;(.*)$").unwrap();
}

/// Parse zsh history file (~/.zsh_history)
/// Format: `: timestamp:0;command`
pub fn parse_zsh_history(history_path: &Path) -> Result<Vec<HistoryEntry>> {
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(history_path)
        .context(format!("Failed to open zsh history: {:?}", history_path))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_command = String::new();
    let mut current_timestamp: Option<DateTime<Utc>> = None;

    for line_result in reader.lines() {
        let line = line_result?;

        // Check if this is a new entry
        if let Some(caps) = ZSH_HISTORY_RE.captures(&line) {
            // Save previous entry if exists
            if !current_command.is_empty() {
                entries.push(HistoryEntry {
                    command: current_command.trim().to_string(),
                    timestamp: current_timestamp,
                });
            }

            // Parse new entry
            let timestamp_str = &caps[1];
            let timestamp_num: i64 = timestamp_str.parse().unwrap_or(0);
            current_timestamp = Utc.timestamp_opt(timestamp_num, 0).single();
            current_command = caps[2].to_string();
        } else {
            // Continuation of previous command (multiline)
            if !current_command.is_empty() {
                current_command.push('\n');
                current_command.push_str(&line);
            }
        }
    }

    // Don't forget the last entry
    if !current_command.is_empty() {
        entries.push(HistoryEntry {
            command: current_command.trim().to_string(),
            timestamp: current_timestamp,
        });
    }

    Ok(entries)
}

/// Parse bash history file (~/.bash_history)
/// Format: plain commands (no timestamps unless HISTTIMEFORMAT set)
/// If HISTTIMEFORMAT is set, timestamps appear as lines starting with '#'
pub fn parse_bash_history(history_path: &Path) -> Result<Vec<HistoryEntry>> {
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(history_path)
        .context(format!("Failed to open bash history: {:?}", history_path))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_timestamp: Option<DateTime<Utc>> = None;

    for line_result in reader.lines() {
        let line = line_result?;

        // Check if this is a timestamp line (starts with #)
        if line.starts_with('#') {
            // Try to parse as timestamp
            if let Ok(timestamp_num) = line[1..].trim().parse::<i64>() {
                current_timestamp = Utc.timestamp_opt(timestamp_num, 0).single();
                continue;
            }
        }

        // Regular command line
        if !line.is_empty() {
            entries.push(HistoryEntry {
                command: line,
                timestamp: current_timestamp,
            });
            current_timestamp = None; // Reset after using
        }
    }

    Ok(entries)
}

/// Parse fish history file (~/.local/share/fish/fish_history)
/// Format: YAML-like
/// ```
/// - cmd: ls -la
///   when: 1234567890
/// ```
pub fn parse_fish_history(history_path: &Path) -> Result<Vec<HistoryEntry>> {
    if !history_path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(history_path)
        .context(format!("Failed to open fish history: {:?}", history_path))?;

    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut current_command: Option<String> = None;
    let mut current_timestamp: Option<DateTime<Utc>> = None;

    for line_result in reader.lines() {
        let line = line_result?;
        let trimmed = line.trim();

        if let Some(cmd) = trimmed.strip_prefix("- cmd:") {
            // New entry
            current_command = Some(cmd.trim().to_string());
        } else if let Some(when) = trimmed.strip_prefix("when:") {
            // Timestamp for current entry
            if let Ok(timestamp_num) = when.trim().parse::<i64>() {
                current_timestamp = Utc.timestamp_opt(timestamp_num, 0).single();
            }

            // If we have both command and timestamp, save the entry
            if let Some(cmd) = current_command.take() {
                entries.push(HistoryEntry {
                    command: cmd,
                    timestamp: current_timestamp,
                });
                current_timestamp = None;
            }
        }
    }

    Ok(entries)
}

/// Get the default history path for the current shell
pub fn get_default_history_path() -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;

    // Try to detect shell from SHELL environment variable
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            return Some(home.join(".zsh_history"));
        } else if shell.contains("bash") {
            return Some(home.join(".bash_history"));
        } else if shell.contains("fish") {
            return Some(home.join(".local/share/fish/fish_history"));
        }
    }

    // Fallback: try to find any existing history file
    let zsh = home.join(".zsh_history");
    if zsh.exists() {
        return Some(zsh);
    }

    let bash = home.join(".bash_history");
    if bash.exists() {
        return Some(bash);
    }

    let fish = home.join(".local/share/fish/fish_history");
    if fish.exists() {
        return Some(fish);
    }

    None
}

/// Parse all available shell history files
pub fn parse_all_history() -> Result<Vec<HistoryEntry>> {
    let mut all_entries = Vec::new();

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(all_entries),
    };

    // Try zsh
    let zsh_path = home.join(".zsh_history");
    if let Ok(entries) = parse_zsh_history(&zsh_path) {
        all_entries.extend(entries);
    }

    // Try bash
    let bash_path = home.join(".bash_history");
    if let Ok(entries) = parse_bash_history(&bash_path) {
        all_entries.extend(entries);
    }

    // Try fish
    let fish_path = home.join(".local/share/fish/fish_history");
    if let Ok(entries) = parse_fish_history(&fish_path) {
        all_entries.extend(entries);
    }

    // Sort by timestamp (newest first)
    all_entries.sort_by(|a, b| {
        match (a.timestamp, b.timestamp) {
            (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    Ok(all_entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_zsh_history() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, ": 1234567890:0;ls -la").unwrap();
        writeln!(temp_file, ": 1234567900:0;git status").unwrap();
        writeln!(temp_file, ": 1234567910:0;brew install wget").unwrap();

        let entries = parse_zsh_history(temp_file.path()).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].command, "ls -la");
        assert_eq!(entries[1].command, "git status");
        assert_eq!(entries[2].command, "brew install wget");
        assert!(entries[0].timestamp.is_some());
    }

    #[test]
    fn test_invokes_binary() {
        let entry = HistoryEntry {
            command: "git status".to_string(),
            timestamp: None,
        };
        assert!(entry.invokes_binary("git"));
        assert!(!entry.invokes_binary("npm"));

        let entry2 = HistoryEntry {
            command: "sudo npm install".to_string(),
            timestamp: None,
        };
        assert!(entry2.invokes_binary("npm"));
    }

    #[test]
    fn test_base_command() {
        let entry = HistoryEntry {
            command: "git status --short".to_string(),
            timestamp: None,
        };
        assert_eq!(entry.base_command(), Some("git".to_string()));
    }
}
