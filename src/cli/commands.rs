// Command implementations
use anyhow::Result;
use std::path::PathBuf;
use std::time::Instant;
use super::{OutputFormat, SortField};
use crate::scanner::{Scanner, homebrew::HomebrewScanner, npm::NpmScanner, pip::PipScanner, cargo::CargoScanner, applications::ApplicationsScanner};
use crate::storage::{Database, database};
use colored::Colorize;

pub fn scan(source: Option<String>, quick: bool) -> Result<()> {
    let start = Instant::now();
    println!("🔍 Scanning packages...");

    let mut all_packages = Vec::new();

    // Scan Homebrew
    if source.is_none() || source.as_deref() == Some("homebrew") || source.as_deref() == Some("brew") {
        let scanner = HomebrewScanner::new();
        if scanner.is_available() {
            print!("  {} Homebrew... ", "✓".green());
            match scanner.scan() {
                Ok(packages) => {
                    println!("{} packages", packages.len().to_string().cyan());
                    all_packages.extend(packages);
                }
                Err(e) => {
                    println!("{}", format!("Error: {}", e).red());
                }
            }
        } else {
            println!("  {} Homebrew (not installed)", "✗".yellow());
        }
    }

    // Scan npm
    if source.is_none() || source.as_deref() == Some("npm") {
        let scanner = NpmScanner::new();
        if scanner.is_available() {
            print!("  {} npm (global)... ", "✓".green());
            match scanner.scan() {
                Ok(packages) => {
                    println!("{} packages", packages.len().to_string().cyan());
                    all_packages.extend(packages);
                }
                Err(e) => {
                    println!("{}", format!("Error: {}", e).red());
                }
            }
        } else {
            println!("  {} npm (not installed)", "✗".yellow());
        }
    }

    // Scan pip
    if source.is_none() || source.as_deref() == Some("pip") || source.as_deref() == Some("python") {
        let scanner = PipScanner::new();
        if scanner.is_available() {
            print!("  {} pip/pipx... ", "✓".green());
            match scanner.scan() {
                Ok(packages) => {
                    println!("{} packages", packages.len().to_string().cyan());
                    all_packages.extend(packages);
                }
                Err(e) => {
                    println!("{}", format!("Error: {}", e).red());
                }
            }
        } else {
            println!("  {} pip (not installed)", "✗".yellow());
        }
    }

    // Scan cargo
    if source.is_none() || source.as_deref() == Some("cargo") || source.as_deref() == Some("rust") {
        let scanner = CargoScanner::new();
        if scanner.is_available() {
            print!("  {} cargo... ", "✓".green());
            match scanner.scan() {
                Ok(packages) => {
                    println!("{} packages", packages.len().to_string().cyan());
                    all_packages.extend(packages);
                }
                Err(e) => {
                    println!("{}", format!("Error: {}", e).red());
                }
            }
        } else {
            println!("  {} cargo (not installed)", "✗".yellow());
        }
    }

    // Scan Applications
    if source.is_none() || source.as_deref() == Some("applications") || source.as_deref() == Some("apps") {
        let scanner = ApplicationsScanner::new();
        if scanner.is_available() {
            print!("  {} Applications... ", "✓".green());
            match scanner.scan() {
                Ok(packages) => {
                    println!("{} apps", packages.len().to_string().cyan());
                    all_packages.extend(packages);
                }
                Err(e) => {
                    println!("{}", format!("Error: {}", e).red());
                }
            }
        }
    }

    let duration = start.elapsed();

    println!("\n📊 Scan complete: {} packages found", all_packages.len().to_string().cyan().bold());

    // Display some statistics
    let formulae_count = all_packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Homebrew))
        .count();
    let casks_count = all_packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::HomebrewCask))
        .count();
    let npm_count = all_packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Npm))
        .count();
    let pip_count = all_packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Pip) || matches!(p.source, crate::scanner::PackageSource::Pipx))
        .count();
    let cargo_count = all_packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Cargo))
        .count();
    let apps_count = all_packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Applications))
        .count();

    if formulae_count > 0 {
        println!("   └── {} Homebrew formulae", formulae_count);
    }
    if casks_count > 0 {
        println!("   └── {} Homebrew casks", casks_count);
    }
    if npm_count > 0 {
        println!("   └── {} npm global packages", npm_count);
    }
    if pip_count > 0 {
        println!("   └── {} pip/pipx packages", pip_count);
    }
    if cargo_count > 0 {
        println!("   └── {} cargo binaries", cargo_count);
    }
    if apps_count > 0 {
        println!("   └── {} Applications", apps_count);
    }

    // Calculate total size
    let total_size: u64 = all_packages.iter()
        .filter_map(|p| p.size_bytes)
        .sum();

    if total_size > 0 {
        println!("   └── {} total", crate::utils::size::format_size(total_size).cyan());
    }

    // Gather usage information
    if !quick {
        println!("\n🔎 Gathering usage information...");
        let start_usage = Instant::now();

        use indicatif::{ProgressBar, ProgressStyle};

        let pb = ProgressBar::new(all_packages.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("━━╺")
        );

        for package in &mut all_packages {
            pb.set_message(package.name.clone());

            match crate::usage::aggregate_usage(package) {
                Ok(usage_info) => {
                    package.last_used = usage_info.last_used;
                    package.usage_count = usage_info.usage_count;
                }
                Err(e) => {
                    // Don't fail the scan if usage tracking fails
                    pb.println(format!("  Warning: Failed to get usage for {}: {}", package.name, e));
                }
            }

            pb.inc(1);
        }

        pb.finish_and_clear();

        let usage_duration = start_usage.elapsed();
        println!("  Usage tracking complete in {:.2}s", usage_duration.as_secs_f64());
    }

    // Save to database
    if !all_packages.is_empty() {
        print!("\n💾 Saving to database... ");
        match save_packages_to_db(&all_packages, &source, duration.as_millis() as i64) {
            Ok(_) => println!("{}", "done".green()),
            Err(e) => println!("{}", format!("Error: {}", e).red()),
        }
    }

    Ok(())
}

fn save_packages_to_db(packages: &[crate::scanner::Package], source: &Option<String>, duration_ms: i64) -> Result<()> {
    let db = Database::default()?;
    db.init()?;

    let conn = db.conn();

    // Save all packages
    for package in packages {
        database::upsert_package(conn, package)?;
    }

    // Record the scan
    let scan_type = source.as_deref().unwrap_or("full");
    database::insert_scan(conn, scan_type, packages.len() as i64, duration_ms)?;

    Ok(())
}

pub fn list(
    source: Option<String>,
    unused: Option<u32>,
    orphaned: bool,
    large: bool,
    sort: SortField,
    limit: Option<usize>,
    format: OutputFormat,
) -> Result<()> {
    // Load packages from database
    let db = Database::default()?;
    db.init()?;

    let mut packages = database::get_packages(db.conn())?;

    if packages.is_empty() {
        println!("No packages found. Run {} first.", "macsweep scan".cyan());
        return Ok(());
    }

    // Apply filters
    if let Some(source_filter) = source {
        packages.retain(|p| {
            let source_str = format!("{:?}", p.source).to_lowercase();
            source_str.contains(&source_filter.to_lowercase())
        });
    }

    if let Some(unused_days) = unused {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(unused_days as i64);
        packages.retain(|p| {
            p.last_used.is_none() || p.last_used.unwrap() < cutoff
        });
    }

    if orphaned {
        // Get orphaned packages from Homebrew
        match crate::analysis::orphans::get_orphaned_brew_packages() {
            Ok(orphan_names) => {
                let orphan_set: std::collections::HashSet<_> = orphan_names.iter()
                    .map(|s| s.as_str())
                    .collect();
                packages.retain(|p| orphan_set.contains(p.name.as_str()));

                if packages.is_empty() {
                    println!("No orphaned packages found.");
                    return Ok(());
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to detect orphaned packages: {}", e);
                // Fall back to dependency-based detection
                packages.retain(|p| p.is_dependency);
            }
        }
    }

    // Apply sorting
    match sort {
        SortField::Name => packages.sort_by(|a, b| a.name.cmp(&b.name)),
        SortField::Size => packages.sort_by(|a, b| {
            b.size_bytes.unwrap_or(0).cmp(&a.size_bytes.unwrap_or(0))
        }),
        SortField::LastUsed => packages.sort_by(|a, b| {
            match (a.last_used, b.last_used) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time), // Most recent first
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        }),
        SortField::InstallDate => packages.sort_by(|a, b| {
            match (a.install_date, b.install_date) {
                (Some(a_time), Some(b_time)) => b_time.cmp(&a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        }),
        SortField::UsageCount => packages.sort_by(|a, b| b.usage_count.cmp(&a.usage_count)),
    }

    // Apply large flag (sort by size descending)
    if large {
        packages.sort_by(|a, b| {
            b.size_bytes.unwrap_or(0).cmp(&a.size_bytes.unwrap_or(0))
        });
    }

    // Apply limit
    if let Some(lim) = limit {
        packages.truncate(lim);
    }

    // Display packages
    match format {
        OutputFormat::Table => display_packages_table(&packages),
        OutputFormat::Json => display_packages_json(&packages)?,
        OutputFormat::Csv => display_packages_csv(&packages)?,
    }

    Ok(())
}

use chrono;
use crate::cli::output;

fn display_packages_table(packages: &[crate::scanner::Package]) {
    use comfy_table::{Table, Cell, Color, Attribute, ContentArrangement};

    let mut table = Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Set headers
    table.set_header(vec![
        Cell::new("Package").add_attribute(Attribute::Bold),
        Cell::new("Source").add_attribute(Attribute::Bold),
        Cell::new("Version").add_attribute(Attribute::Bold),
        Cell::new("Size").add_attribute(Attribute::Bold),
        Cell::new("Install Date").add_attribute(Attribute::Bold),
        Cell::new("Last Used").add_attribute(Attribute::Bold),
    ]);

    // Add rows
    for pkg in packages {
        let source_str = format!("{:?}", pkg.source);
        let version_str = pkg.version.as_deref().unwrap_or("-");
        let size_str = pkg.size_bytes
            .map(|s| crate::utils::size::format_size(s))
            .unwrap_or_else(|| "-".to_string());

        let install_date_str = pkg.install_date
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());

        let last_used_str = if let Some(last_used) = pkg.last_used {
            let days = crate::utils::date::days_since(&last_used);
            crate::utils::date::format_days_ago(days)
        } else {
            "Never".to_string()
        };

        table.add_row(vec![
            Cell::new(&pkg.name),
            Cell::new(source_str).fg(Color::Cyan),
            Cell::new(version_str),
            Cell::new(size_str),
            Cell::new(install_date_str),
            Cell::new(last_used_str),
        ]);
    }

    println!("\n{}", table);
    println!("\nTotal: {} packages", packages.len().to_string().cyan().bold());

    // Show total size
    let total_size: u64 = packages.iter()
        .filter_map(|p| p.size_bytes)
        .sum();
    if total_size > 0 {
        println!("Total size: {}", crate::utils::size::format_size(total_size).cyan().bold());
    }
}

fn display_packages_json(packages: &[crate::scanner::Package]) -> Result<()> {
    let json = serde_json::to_string_pretty(packages)?;
    println!("{}", json);
    Ok(())
}

fn display_packages_csv(packages: &[crate::scanner::Package]) -> Result<()> {
    use std::io;
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Write headers
    wtr.write_record(&["name", "source", "version", "size_bytes", "install_date", "last_used"])?;

    // Write data
    for pkg in packages {
        wtr.write_record(&[
            &pkg.name,
            &format!("{:?}", pkg.source),
            pkg.version.as_deref().unwrap_or(""),
            &pkg.size_bytes.map(|s| s.to_string()).unwrap_or_default(),
            &pkg.install_date.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            &pkg.last_used.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
        ])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn info(package: &str) -> Result<()> {
    println!("📦 Package info for: {}", package);
    // TODO: Implement info logic
    Ok(())
}

pub fn clean(dry_run: bool, yes: bool, source: Option<String>, interactive: bool) -> Result<()> {
    println!("🧹 MacSweep Cleanup\n");

    // Load packages from database
    let db = Database::default()?;
    db.init()?;

    let packages = database::get_packages(db.conn())?;

    if packages.is_empty() {
        println!("No packages found. Run {} first.", "macsweep scan".cyan());
        return Ok(());
    }

    // Generate recommendations
    let recommendations = crate::analysis::recommendations::generate_recommendations(&packages)?;

    if recommendations.is_empty() {
        println!("{}", "No cleanup recommendations at this time. ✨".green());
        return Ok(());
    }

    // Filter by source if specified
    let mut recommendations = recommendations;
    if let Some(ref source_filter) = source {
        recommendations.retain(|r| {
            if let Some(pkg) = packages.iter().find(|p| p.name == r.package) {
                let source_str = format!("{:?}", pkg.source).to_lowercase();
                source_str.contains(&source_filter.to_lowercase())
            } else {
                false
            }
        });

        if recommendations.is_empty() {
            println!("No cleanup recommendations for source: {}", source_filter);
            return Ok(());
        }
    }

    // Summary
    let total_recoverable: u64 = recommendations.iter()
        .map(|r| r.size_recoverable)
        .sum();

    println!("{}", "Packages to remove:".bold());
    println!("  Total: {}", recommendations.len().to_string().yellow());
    println!("  Potential space savings: {}\n", crate::utils::size::format_size(total_recoverable).green().bold());

    // Show what will be removed
    for (idx, rec) in recommendations.iter().enumerate() {
        let severity_icon = match rec.severity {
            crate::analysis::recommendations::RecommendationSeverity::Safe => "✓",
            crate::analysis::recommendations::RecommendationSeverity::Review => "⚠",
            crate::analysis::recommendations::RecommendationSeverity::Warning => "•",
        };
        let size_str = crate::utils::size::format_size(rec.size_recoverable);
        println!("  {} {} - {} ({})",
            severity_icon,
            rec.package.cyan(),
            rec.reason,
            size_str.yellow()
        );

        // Limit display to prevent overwhelming output
        if idx >= 19 && recommendations.len() > 20 {
            println!("  ... and {} more", recommendations.len() - 20);
            break;
        }
    }
    println!();

    if dry_run {
        println!("{}", "[DRY RUN MODE] - No packages will be removed".yellow().bold());
        println!("Run without --dry-run to actually remove packages.\n");
    }

    // Interactive mode - let user select packages
    if interactive && !dry_run {
        use dialoguer::{theme::ColorfulTheme, MultiSelect};

        println!("{}", "Select packages to remove (Space to select, Enter to confirm):".bold());
        println!();

        let items: Vec<String> = recommendations.iter().map(|r| {
            let severity_icon = match r.severity {
                crate::analysis::recommendations::RecommendationSeverity::Safe => "✓",
                crate::analysis::recommendations::RecommendationSeverity::Review => "⚠",
                crate::analysis::recommendations::RecommendationSeverity::Warning => "•",
            };
            let size_str = crate::utils::size::format_size(r.size_recoverable);
            format!("{} {} - {} ({})", severity_icon, r.package, r.reason, size_str)
        }).collect();

        let selected = MultiSelect::with_theme(&ColorfulTheme::default())
            .items(&items)
            .interact()?;

        if selected.is_empty() {
            println!("No packages selected. Cleanup cancelled.");
            return Ok(());
        }

        // Filter recommendations to only selected ones
        let selected_recs: Vec<_> = selected.iter()
            .map(|&idx| recommendations[idx].clone())
            .collect();
        recommendations = selected_recs;

        let new_total: u64 = recommendations.iter()
            .map(|r| r.size_recoverable)
            .sum();

        println!("\n{} packages selected", recommendations.len().to_string().green().bold());
        println!("Space to recover: {}\n", crate::utils::size::format_size(new_total).green().bold());
    }

    // Confirm before proceeding (unless --yes flag)
    if !dry_run && !yes && !interactive {
        use std::io::{self, Write};
        print!("Proceed with cleanup? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    }

    // Create backup before cleanup
    let backup_manifest_path = if !dry_run {
        println!("\n{}", "Creating backup...".bold());
        let packages_to_remove: Vec<_> = recommendations.iter()
            .filter_map(|r| packages.iter().find(|p| p.name == r.package))
            .cloned()
            .collect();

        match crate::cleanup::backup::create_backup(&packages_to_remove) {
            Ok(path) => Some(path),
            Err(e) => {
                eprintln!("⚠️  Warning: Failed to create backup: {}", e);
                eprintln!("   Proceeding without backup...");
                None
            }
        }
    } else {
        None
    };

    // Perform cleanup
    println!("\n{}", "Starting cleanup...".bold());

    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new(recommendations.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("━━╺")
    );

    let mut removed_count = 0;
    let mut failed_count = 0;
    let mut total_recovered: u64 = 0;

    for rec in &recommendations {
        pb.set_message(rec.package.clone());

        // Find the package
        if let Some(package) = packages.iter().find(|p| p.name == rec.package) {
            match crate::cleanup::executor::remove_package(package, dry_run) {
                Ok(true) => {
                    removed_count += 1;
                    total_recovered += rec.size_recoverable;
                }
                Ok(false) => {
                    failed_count += 1;
                }
                Err(e) => {
                    pb.println(format!("  ✗ Error removing {}: {}", package.name, e));
                    failed_count += 1;
                }
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Summary
    println!("\n{}", "Cleanup Summary:".bold());
    if dry_run {
        println!("  Would remove: {}", removed_count.to_string().green());
        println!("  Would recover: {}", crate::utils::size::format_size(total_recovered).green().bold());
    } else {
        println!("  Successfully removed: {}", removed_count.to_string().green());
        if failed_count > 0 {
            println!("  Failed: {}", failed_count.to_string().red());
        }
        println!("  Space recovered: {}", crate::utils::size::format_size(total_recovered).green().bold());

        // Record cleanup in database
        if removed_count > 0 && backup_manifest_path.is_some() {
            if let Err(e) = database::insert_cleanup(
                db.conn(),
                backup_manifest_path.as_ref().unwrap(),
                removed_count as i64,
                total_recovered as i64,
            ) {
                eprintln!("Warning: Failed to record cleanup in database: {}", e);
            }
        }

        // Show undo instructions
        if removed_count > 0 {
            println!("\n💡 Run {} to update the database", "macsweep scan".cyan());
            if backup_manifest_path.is_some() {
                println!("💡 Run {} to undo this cleanup", "macsweep undo".cyan());
            }
        }
    }

    Ok(())
}

pub fn history(package: &str) -> Result<()> {
    println!("📊 Usage history for: {}", package);
    // TODO: Implement history logic
    Ok(())
}

pub fn stats() -> Result<()> {
    println!("📈 MacSweep Statistics\n");

    // Load packages from database
    let db = Database::default()?;
    db.init()?;

    let packages = database::get_packages(db.conn())?;

    if packages.is_empty() {
        println!("No packages found. Run {} first.", "macsweep scan".cyan());
        return Ok(());
    }

    // Overall statistics
    println!("{}",  "═══ Package Overview ═══".cyan().bold());
    println!("Total packages: {}", packages.len().to_string().yellow().bold());

    let total_size: u64 = packages.iter()
        .filter_map(|p| p.size_bytes)
        .sum();
    println!("Total size: {}", crate::utils::size::format_size(total_size).yellow().bold());

    // Breakdown by source
    let homebrew_count = packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Homebrew))
        .count();
    let casks_count = packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::HomebrewCask))
        .count();
    let apps_count = packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Applications))
        .count();
    let npm_count = packages.iter()
        .filter(|p| matches!(p.source, crate::scanner::PackageSource::Npm))
        .count();

    println!("\n{}",  "Source breakdown:".bold());
    if homebrew_count > 0 {
        println!("  Homebrew formulae: {}", homebrew_count.to_string().cyan());
    }
    if casks_count > 0 {
        println!("  Homebrew casks: {}", casks_count.to_string().cyan());
    }
    if apps_count > 0 {
        println!("  Applications: {}", apps_count.to_string().cyan());
    }
    if npm_count > 0 {
        println!("  npm packages: {}", npm_count.to_string().cyan());
    }

    // Usage statistics
    let used_count = packages.iter()
        .filter(|p| p.last_used.is_some())
        .count();
    let never_used_count = packages.len() - used_count;

    println!("\n{}", "═══ Usage Statistics ═══".cyan().bold());
    println!("Packages with usage data: {}", used_count.to_string().green());
    println!("Packages without usage data: {}", never_used_count.to_string().yellow());

    // Generate cleanup recommendations
    println!("\n{}", "═══ Cleanup Recommendations ═══".cyan().bold());

    let recommendations = crate::analysis::recommendations::generate_recommendations(&packages)?;

    if recommendations.is_empty() {
        println!("{}", "No cleanup recommendations at this time. ✨".green());
        return Ok(());
    }

    let total_recoverable: u64 = recommendations.iter()
        .map(|r| r.size_recoverable)
        .sum();

    println!("Found {} cleanup opportunities", recommendations.len().to_string().yellow().bold());
    println!("Potential space savings: {}\n", crate::utils::size::format_size(total_recoverable).green().bold());

    // Group by severity
    let safe_recs: Vec<_> = recommendations.iter()
        .filter(|r| r.severity == crate::analysis::recommendations::RecommendationSeverity::Safe)
        .collect();
    let review_recs: Vec<_> = recommendations.iter()
        .filter(|r| r.severity == crate::analysis::recommendations::RecommendationSeverity::Review)
        .collect();
    let warning_recs: Vec<_> = recommendations.iter()
        .filter(|r| r.severity == crate::analysis::recommendations::RecommendationSeverity::Warning)
        .collect();

    if !safe_recs.is_empty() {
        println!("{} ({})", "Safe to Remove:".green().bold(), safe_recs.len());
        for rec in safe_recs.iter().take(5) {
            let size_str = crate::utils::size::format_size(rec.size_recoverable);
            println!("  • {} - {} ({})", rec.package.cyan(), rec.reason, size_str.yellow());
        }
        if safe_recs.len() > 5 {
            println!("  ... and {} more", safe_recs.len() - 5);
        }
        println!();
    }

    if !review_recs.is_empty() {
        println!("{} ({})", "Review Recommended:".yellow().bold(), review_recs.len());
        for rec in review_recs.iter().take(5) {
            let size_str = crate::utils::size::format_size(rec.size_recoverable);
            println!("  • {} - {} ({})", rec.package.cyan(), rec.reason, size_str.yellow());
        }
        if review_recs.len() > 5 {
            println!("  ... and {} more", review_recs.len() - 5);
        }
        println!();
    }

    if !warning_recs.is_empty() {
        println!("{} ({})", "Consider Checking:".cyan().bold(), warning_recs.len());
        for rec in warning_recs.iter().take(5) {
            let size_str = crate::utils::size::format_size(rec.size_recoverable);
            println!("  • {} - {} ({})", rec.package, rec.reason, size_str.yellow());
        }
        if warning_recs.len() > 5 {
            println!("  ... and {} more", warning_recs.len() - 5);
        }
    }

    println!("\n💡 Run {} to see orphaned packages", "macsweep list --orphaned".cyan());
    println!("💡 Run {} to see unused packages", "macsweep list --unused 90".cyan());

    Ok(())
}

pub fn export(output: Option<PathBuf>) -> Result<()> {
    println!("💾 Exporting data to: {:?}", output.unwrap_or_else(|| PathBuf::from("stdout")));
    // TODO: Implement export logic
    Ok(())
}

pub fn undo(backup_id: Option<String>, list: bool) -> Result<()> {
    if list {
        // List available backups
        println!("📋 Available Backups:\n");

        let backups = crate::cleanup::backup::list_backups()?;

        if backups.is_empty() {
            println!("No backups found.");
            return Ok(());
        }

        for (idx, backup) in backups.iter().enumerate() {
            println!("  {}. {}", idx + 1, backup.cyan());
        }

        println!("\nRestore a backup with: {}", "macsweep undo <backup_id>".cyan());
        return Ok(());
    }

    // Restore from backup
    let backup_to_restore = if let Some(id) = backup_id {
        id
    } else {
        // Use most recent backup
        let backups = crate::cleanup::backup::list_backups()?;

        if backups.is_empty() {
            println!("No backups found to restore.");
            return Ok(());
        }

        let most_recent = &backups[0];
        println!("No backup ID specified, using most recent: {}", most_recent.cyan());
        most_recent.clone()
    };

    crate::cleanup::backup::restore_backup(&backup_to_restore)?;

    Ok(())
}
