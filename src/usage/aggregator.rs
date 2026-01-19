// Aggregates usage information from multiple sources
use super::{UsageInfo, UsageSource};
use crate::scanner::{Package, PackageSource};
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Aggregate usage information from all available sources
pub fn aggregate_usage(package: &Package) -> Result<UsageInfo> {
    let mut info = UsageInfo::new();

    // For Applications, use Spotlight metadata
    if package.source == PackageSource::Applications || package.source == PackageSource::HomebrewCask {
        if let Some(ref app_path) = package.binary_path {
            // Get Spotlight metadata
            match super::spotlight::get_spotlight_usage(app_path) {
                Ok((last_used, use_count)) => {
                    if let Some(dt) = last_used {
                        info.sources.push(UsageSource::SpotlightMetadata { last_used: dt });

                        // Update aggregated values
                        if info.last_used.is_none() || info.last_used.unwrap() < dt {
                            info.last_used = Some(dt);
                        }
                    }

                    if let Some(count) = use_count {
                        info.usage_count = count;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to get Spotlight metadata for {}: {}", package.name, e);
                }
            }
        }
    }

    // For CLI tools and binaries, check shell history
    if let Some(ref bin_path) = package.binary_path {
        // Try to find usage in shell history
        match find_in_shell_history(&package.name, bin_path) {
            Ok(Some((last_used, count))) => {
                info.sources.push(UsageSource::ShellHistory {
                    count,
                    last_used,
                });

                // Update aggregated values
                if info.last_used.is_none() || info.last_used.unwrap() < last_used {
                    info.last_used = Some(last_used);
                }
                info.usage_count += count;
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Warning: Failed to check shell history: {}", e);
            }
        }
    }

    // Check file access time as fallback
    if let Some(ref bin_path) = package.binary_path {
        match super::atime::get_binary_atime(bin_path) {
            Ok(Some(atime)) => {
                info.sources.push(UsageSource::FileAccessTime { atime });

                // Only use atime if we don't have better data
                if info.last_used.is_none() {
                    info.last_used = Some(atime);
                }
            }
            Ok(None) => {}
            Err(e) => {
                eprintln!("Warning: Failed to get file access time: {}", e);
            }
        }
    }

    Ok(info)
}

/// Find package usage in shell history
fn find_in_shell_history(
    package_name: &str,
    _bin_path: &std::path::Path,
) -> Result<Option<(DateTime<Utc>, u32)>> {
    // Parse all shell histories
    let entries = super::shell_history::parse_all_history()?;

    let mut count = 0;
    let mut last_used: Option<DateTime<Utc>> = None;

    // Count occurrences and find last usage
    for entry in entries {
        if entry.invokes_binary(package_name) {
            count += 1;
            if let Some(ts) = entry.timestamp {
                if last_used.is_none() || last_used.unwrap() < ts {
                    last_used = Some(ts);
                }
            }
        }
    }

    if count > 0 && last_used.is_some() {
        Ok(Some((last_used.unwrap(), count)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Run manually
    fn test_aggregate_usage() {
        let mut package = Package::new("git".to_string(), PackageSource::Homebrew);
        package.binary_path = Some(std::path::PathBuf::from("/usr/bin/git"));

        let info = aggregate_usage(&package).unwrap();
        println!("Last used: {:?}", info.last_used);
        println!("Usage count: {}", info.usage_count);
        println!("Sources: {:?}", info.sources);
    }
}
