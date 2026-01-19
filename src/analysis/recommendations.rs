// Cleanup recommendations engine
use crate::scanner::Package;
use anyhow::Result;
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub package: String,
    pub reason: String,
    pub severity: RecommendationSeverity,
    pub size_recoverable: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationSeverity {
    Safe,      // Orphaned dependencies - can be removed safely
    Review,    // Unused 90+ days - should review before removing
    Warning,   // Unused 30-90 days - check if still needed
}

pub fn generate_recommendations(packages: &[Package]) -> Result<Vec<Recommendation>> {
    let mut recommendations = Vec::new();
    let now = Utc::now();

    // Get orphaned packages from Homebrew
    let orphan_names = crate::analysis::orphans::get_orphaned_brew_packages()
        .unwrap_or_else(|_| Vec::new());
    let orphan_set: std::collections::HashSet<_> = orphan_names.iter()
        .map(|s| s.as_str())
        .collect();

    for package in packages {
        // Check if package is orphaned
        if orphan_set.contains(package.name.as_str()) {
            recommendations.push(Recommendation {
                package: package.name.clone(),
                reason: format!("Orphaned dependency - no longer required by any installed package"),
                severity: RecommendationSeverity::Safe,
                size_recoverable: package.size_bytes.unwrap_or(0),
            });
            continue; // Don't double-count orphans
        }

        // Check if package is unused for extended period
        if let Some(last_used) = package.last_used {
            let days_since_use = (now - last_used).num_days();

            if days_since_use >= 180 {
                // 6+ months unused
                recommendations.push(Recommendation {
                    package: package.name.clone(),
                    reason: format!("Not used in {} days (~{} months)",
                        days_since_use, days_since_use / 30),
                    severity: RecommendationSeverity::Review,
                    size_recoverable: package.size_bytes.unwrap_or(0),
                });
            } else if days_since_use >= 90 {
                // 3-6 months unused
                recommendations.push(Recommendation {
                    package: package.name.clone(),
                    reason: format!("Not used in {} days (~{} months)",
                        days_since_use, days_since_use / 30),
                    severity: RecommendationSeverity::Review,
                    size_recoverable: package.size_bytes.unwrap_or(0),
                });
            } else if days_since_use >= 30 {
                // 1-3 months unused
                recommendations.push(Recommendation {
                    package: package.name.clone(),
                    reason: format!("Not used in {} days", days_since_use),
                    severity: RecommendationSeverity::Warning,
                    size_recoverable: package.size_bytes.unwrap_or(0),
                });
            }
        } else {
            // Never used (no usage data)
            // Only recommend if it's also large (>100MB)
            if let Some(size) = package.size_bytes {
                if size > 100 * 1024 * 1024 { // 100 MB
                    recommendations.push(Recommendation {
                        package: package.name.clone(),
                        reason: format!("No usage data found - {} in size", format_size(size)),
                        severity: RecommendationSeverity::Review,
                        size_recoverable: size,
                    });
                }
            }
        }
    }

    // Sort by size (largest first) within each severity level
    recommendations.sort_by(|a, b| {
        match (a.severity, b.severity) {
            (RecommendationSeverity::Safe, RecommendationSeverity::Safe) => {
                b.size_recoverable.cmp(&a.size_recoverable)
            }
            (RecommendationSeverity::Safe, _) => std::cmp::Ordering::Less,
            (_, RecommendationSeverity::Safe) => std::cmp::Ordering::Greater,
            (RecommendationSeverity::Review, RecommendationSeverity::Review) => {
                b.size_recoverable.cmp(&a.size_recoverable)
            }
            (RecommendationSeverity::Review, RecommendationSeverity::Warning) => {
                std::cmp::Ordering::Less
            }
            (RecommendationSeverity::Warning, RecommendationSeverity::Review) => {
                std::cmp::Ordering::Greater
            }
            (RecommendationSeverity::Warning, RecommendationSeverity::Warning) => {
                b.size_recoverable.cmp(&a.size_recoverable)
            }
        }
    });

    Ok(recommendations)
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_recommendations_for_unused_packages() {
        let now = Utc::now();
        let mut package = crate::scanner::Package::new("old-package".to_string(), crate::scanner::PackageSource::Homebrew);
        package.last_used = Some(now - Duration::days(200)); // Unused for 200 days
        package.size_bytes = Some(100 * 1024 * 1024); // 100 MB

        let packages = vec![package];
        let recommendations = generate_recommendations(&packages).unwrap();

        assert_eq!(recommendations.len(), 1);
        assert_eq!(recommendations[0].package, "old-package");
        assert_eq!(recommendations[0].severity, RecommendationSeverity::Review);
    }

    #[test]
    fn test_recommendations_for_recent_packages() {
        let now = Utc::now();
        let mut package = crate::scanner::Package::new("recent-package".to_string(), crate::scanner::PackageSource::Homebrew);
        package.last_used = Some(now - Duration::days(5)); // Used 5 days ago
        package.size_bytes = Some(50 * 1024 * 1024);

        let packages = vec![package];
        let recommendations = generate_recommendations(&packages).unwrap();

        // Should not recommend removal for recently used packages
        assert_eq!(recommendations.len(), 0);
    }

    #[test]
    fn test_recommendations_severity_order() {
        let now = Utc::now();

        let mut safe_pkg = crate::scanner::Package::new("safe-pkg".to_string(), crate::scanner::PackageSource::Homebrew);
        safe_pkg.size_bytes = Some(10 * 1024 * 1024);

        let mut review_pkg = crate::scanner::Package::new("review-pkg".to_string(), crate::scanner::PackageSource::Homebrew);
        review_pkg.last_used = Some(now - Duration::days(180));
        review_pkg.size_bytes = Some(200 * 1024 * 1024);

        let mut warning_pkg = crate::scanner::Package::new("warning-pkg".to_string(), crate::scanner::PackageSource::Homebrew);
        warning_pkg.last_used = Some(now - Duration::days(45));
        warning_pkg.size_bytes = Some(50 * 1024 * 1024);

        let packages = vec![warning_pkg, review_pkg, safe_pkg];
        let recommendations = generate_recommendations(&packages).unwrap();

        // Should be ordered by severity: Safe first, then Review, then Warning
        // Within same severity, ordered by size (largest first)
        assert!(recommendations.len() >= 2);

        // Find review recommendation (should be before warning)
        let review_idx = recommendations.iter().position(|r| r.severity == RecommendationSeverity::Review);
        let warning_idx = recommendations.iter().position(|r| r.severity == RecommendationSeverity::Warning);

        if let (Some(rev), Some(warn)) = (review_idx, warning_idx) {
            assert!(rev < warn, "Review recommendations should come before Warning");
        }
    }

    #[test]
    fn test_large_unused_package_recommendation() {
        let mut package = crate::scanner::Package::new("large-unused".to_string(), crate::scanner::PackageSource::Homebrew);
        package.last_used = None; // Never used
        package.size_bytes = Some(150 * 1024 * 1024); // 150 MB

        let packages = vec![package];
        let recommendations = generate_recommendations(&packages).unwrap();

        // Large packages without usage data should be recommended for review
        assert_eq!(recommendations.len(), 1);
        assert_eq!(recommendations[0].severity, RecommendationSeverity::Review);
        assert!(recommendations[0].reason.contains("No usage data"));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 bytes");
        assert_eq!(format_size(512), "512 bytes");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(1536 * 1024 * 1024), "1.5 GB");
    }
}
