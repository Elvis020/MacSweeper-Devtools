// macOS Spotlight metadata for GUI apps
use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use std::path::Path;
use std::process::Command;

lazy_static! {
    // Pattern for parsing mdls datetime: "2026-01-18 21:35:48 +0000"
    static ref MDLS_DATETIME_RE: Regex =
        Regex::new(r"(\d{4})-(\d{2})-(\d{2}) (\d{2}):(\d{2}):(\d{2})").unwrap();

    // Pattern for extracting numeric values
    static ref MDLS_NUMBER_RE: Regex = Regex::new(r"=\s*(\d+)").unwrap();
}

/// Get the last used date for an application from Spotlight metadata
pub fn get_spotlight_last_used(app_path: &Path) -> Result<Option<DateTime<Utc>>> {
    let output = Command::new("mdls")
        .args(["-name", "kMDItemLastUsedDate", "-raw"])
        .arg(app_path)
        .output()
        .context("Failed to run mdls command")?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .context("Failed to parse mdls output as UTF-8")?;

    parse_mdls_datetime(&stdout)
}

/// Get the usage count for an application from Spotlight metadata
pub fn get_spotlight_use_count(app_path: &Path) -> Result<Option<u32>> {
    let output = Command::new("mdls")
        .args(["-name", "kMDItemUseCount"])
        .arg(app_path)
        .output()
        .context("Failed to run mdls command")?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8(output.stdout)
        .context("Failed to parse mdls output as UTF-8")?;

    // Parse the output: "kMDItemUseCount = 1033"
    if let Some(caps) = MDLS_NUMBER_RE.captures(&stdout) {
        if let Ok(count) = caps[1].parse::<u32>() {
            return Ok(Some(count));
        }
    }

    Ok(None)
}

/// Get both last used date and use count in one call (more efficient)
pub fn get_spotlight_usage(app_path: &Path) -> Result<(Option<DateTime<Utc>>, Option<u32>)> {
    let output = Command::new("mdls")
        .args(["-name", "kMDItemLastUsedDate", "-name", "kMDItemUseCount"])
        .arg(app_path)
        .output()
        .context("Failed to run mdls command")?;

    if !output.status.success() {
        return Ok((None, None));
    }

    let stdout = String::from_utf8(output.stdout)
        .context("Failed to parse mdls output as UTF-8")?;

    let last_used = parse_mdls_datetime(&stdout)?;
    let use_count = if let Some(caps) = MDLS_NUMBER_RE.captures(&stdout) {
        caps[1].parse::<u32>().ok()
    } else {
        None
    };

    Ok((last_used, use_count))
}

fn parse_mdls_datetime(output: &str) -> Result<Option<DateTime<Utc>>> {
    // Check if the value is "(null)"
    if output.contains("(null)") {
        return Ok(None);
    }

    // Try to parse datetime from the output
    if let Some(caps) = MDLS_DATETIME_RE.captures(output) {
        let year: i32 = caps[1].parse()?;
        let month: u32 = caps[2].parse()?;
        let day: u32 = caps[3].parse()?;
        let hour: u32 = caps[4].parse()?;
        let minute: u32 = caps[5].parse()?;
        let second: u32 = caps[6].parse()?;

        let naive_date = chrono::NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| anyhow::anyhow!("Invalid date"))?;
        let naive_time = chrono::NaiveTime::from_hms_opt(hour, minute, second)
            .ok_or_else(|| anyhow::anyhow!("Invalid time"))?;
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);
        return Ok(Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc)));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mdls_datetime() {
        use chrono::Datelike;

        let output = "kMDItemLastUsedDate = 2026-01-18 21:35:48 +0000";
        let result = parse_mdls_datetime(output).unwrap();
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2026);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 18);
    }

    #[test]
    fn test_parse_mdls_null() {
        let output = "kMDItemLastUsedDate = (null)";
        let result = parse_mdls_datetime(output).unwrap();
        assert!(result.is_none());
    }

    #[test]
    #[ignore] // Run manually on macOS
    fn test_get_spotlight_usage() {
        let app_path = Path::new("/Applications/Arc.app");
        if app_path.exists() {
            let (last_used, use_count) = get_spotlight_usage(app_path).unwrap();
            println!("Last used: {:?}", last_used);
            println!("Use count: {:?}", use_count);
        }
    }
}
