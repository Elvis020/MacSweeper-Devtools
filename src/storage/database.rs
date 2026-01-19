// Database operations (CRUD for packages, usage events, scans)
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use crate::scanner::{Package, PackageSource};
use chrono::{DateTime, Utc};

/// Insert or update a package in the database
pub fn upsert_package(conn: &Connection, package: &Package) -> Result<i64> {
    let source_str = format!("{:?}", package.source);
    let version_str = package.version.as_deref();
    let binary_path_str = package.binary_path.as_ref().map(|p| p.to_string_lossy().to_string());
    let install_date_str = package.install_date.map(|dt| dt.to_rfc3339());
    let last_used_str = package.last_used.map(|dt| dt.to_rfc3339());

    conn.execute(
        "INSERT INTO packages (name, source, version, binary_path, install_date, size_bytes, is_dependency, last_used, usage_count)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(name, source) DO UPDATE SET
            version = excluded.version,
            binary_path = excluded.binary_path,
            install_date = excluded.install_date,
            size_bytes = excluded.size_bytes,
            is_dependency = excluded.is_dependency,
            last_used = excluded.last_used,
            usage_count = excluded.usage_count,
            last_seen = CURRENT_TIMESTAMP",
        params![
            &package.name,
            &source_str,
            version_str,
            binary_path_str,
            install_date_str,
            package.size_bytes.map(|s| s as i64),
            package.is_dependency,
            last_used_str,
            package.usage_count as i64,
        ],
    )?;

    // Get the package ID
    let package_id: i64 = conn.query_row(
        "SELECT id FROM packages WHERE name = ?1 AND source = ?2",
        params![&package.name, &source_str],
        |row| row.get(0),
    )?;

    // Store dependencies
    store_dependencies(conn, package_id, &package.dependencies)?;

    Ok(package_id)
}

/// Store package dependencies
fn store_dependencies(conn: &Connection, package_id: i64, dependencies: &[String]) -> Result<()> {
    // First, delete existing dependencies
    conn.execute(
        "DELETE FROM package_dependencies WHERE package_id = ?1",
        params![package_id],
    )?;

    // Insert new dependencies
    for dep in dependencies {
        conn.execute(
            "INSERT INTO package_dependencies (package_id, dependency_name) VALUES (?1, ?2)",
            params![package_id, dep],
        )?;
    }

    Ok(())
}

/// Get all packages from the database
pub fn get_packages(conn: &Connection) -> Result<Vec<Package>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source, version, binary_path, install_date,
                size_bytes, is_dependency, last_used, usage_count
         FROM packages
         ORDER BY name"
    )?;

    let packages = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let source_str: String = row.get(2)?;
        let source = parse_package_source(&source_str);

        let version: Option<String> = row.get(3)?;
        let binary_path_str: Option<String> = row.get(4)?;
        let binary_path = binary_path_str.map(PathBuf::from);

        let install_date_str: Option<String> = row.get(5)?;
        let install_date = install_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let size_bytes: Option<i64> = row.get(6)?;
        let is_dependency: bool = row.get(7)?;

        let last_used_str: Option<String> = row.get(8)?;
        let last_used = last_used_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let usage_count: u32 = row.get(9).unwrap_or(0);

        Ok((id, Package {
            name,
            version,
            source,
            install_date,
            size_bytes: size_bytes.map(|s| s as u64),
            binary_path,
            is_dependency,
            dependencies: Vec::new(), // Will be populated below
            dependents: Vec::new(),
            last_used,
            usage_count,
        }))
    })?;

    let mut result = Vec::new();
    for pkg_result in packages {
        let (id, mut pkg) = pkg_result?;
        pkg.dependencies = get_package_dependencies(conn, id)?;
        result.push(pkg);
    }

    Ok(result)
}

use std::path::PathBuf;

fn parse_package_source(s: &str) -> PackageSource {
    match s {
        "Homebrew" => PackageSource::Homebrew,
        "HomebrewCask" => PackageSource::HomebrewCask,
        "MacAppStore" => PackageSource::MacAppStore,
        "Npm" => PackageSource::Npm,
        "Pip" => PackageSource::Pip,
        "Pipx" => PackageSource::Pipx,
        "Cargo" => PackageSource::Cargo,
        "Gem" => PackageSource::Gem,
        "Go" => PackageSource::Go,
        "Composer" => PackageSource::Composer,
        "Applications" => PackageSource::Applications,
        "LocalBin" => PackageSource::LocalBin,
        _ => PackageSource::LocalBin, // Default fallback
    }
}

/// Get dependencies for a package
fn get_package_dependencies(conn: &Connection, package_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT dependency_name FROM package_dependencies WHERE package_id = ?1"
    )?;

    let deps = stmt.query_map(params![package_id], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(deps)
}

/// Update package usage information
pub fn update_package_usage(
    conn: &Connection,
    package_id: i64,
    last_used: DateTime<Utc>,
    usage_count: u32,
) -> Result<()> {
    conn.execute(
        "UPDATE packages SET last_used = ?1, usage_count = ?2 WHERE id = ?3",
        params![last_used.to_rfc3339(), usage_count, package_id],
    )?;
    Ok(())
}

/// Record a usage event
pub fn insert_usage_event(
    conn: &Connection,
    package_id: i64,
    event_type: &str,
    event_date: DateTime<Utc>,
    details: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO usage_events (package_id, event_type, event_date, details)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(package_id, event_type, event_date) DO NOTHING",
        params![package_id, event_type, event_date.to_rfc3339(), details],
    )?;
    Ok(())
}

/// Record a scan
pub fn insert_scan(
    conn: &Connection,
    scan_type: &str,
    packages_found: i64,
    duration_ms: i64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO scans (scan_type, packages_found, duration_ms)
         VALUES (?1, ?2, ?3)",
        params![scan_type, packages_found, duration_ms],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Record a cleanup operation
pub fn insert_cleanup(
    conn: &Connection,
    backup_manifest_path: &str,
    packages_removed: i64,
    space_recovered: i64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO cleanups (backup_manifest_path, packages_removed, space_recovered)
         VALUES (?1, ?2, ?3)",
        params![backup_manifest_path, packages_removed, space_recovered],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Get package by name and source
pub fn get_package_by_name(conn: &Connection, name: &str, source: &PackageSource) -> Result<Option<Package>> {
    let source_str = format!("{:?}", source);

    let mut stmt = conn.prepare(
        "SELECT id, name, source, version, binary_path, install_date,
                size_bytes, is_dependency, last_used, usage_count
         FROM packages
         WHERE name = ?1 AND source = ?2"
    )?;

    let result = stmt.query_row(params![name, source_str], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let source_str: String = row.get(2)?;
        let source = parse_package_source(&source_str);

        let version: Option<String> = row.get(3)?;
        let binary_path_str: Option<String> = row.get(4)?;
        let binary_path = binary_path_str.map(PathBuf::from);

        let install_date_str: Option<String> = row.get(5)?;
        let install_date = install_date_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let size_bytes: Option<i64> = row.get(6)?;
        let is_dependency: bool = row.get(7)?;

        let last_used_str: Option<String> = row.get(8)?;
        let last_used = last_used_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let usage_count: u32 = row.get(9).unwrap_or(0);

        Ok((id, Package {
            name,
            version,
            source,
            install_date,
            size_bytes: size_bytes.map(|s| s as u64),
            binary_path,
            is_dependency,
            dependencies: Vec::new(),
            dependents: Vec::new(),
            last_used,
            usage_count,
        }))
    });

    match result {
        Ok((id, mut pkg)) => {
            pkg.dependencies = get_package_dependencies(conn, id)?;
            Ok(Some(pkg))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::storage::Database;

    #[test]
    fn test_upsert_package() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.init().unwrap();

        let mut package = Package::new("test-package".to_string(), PackageSource::Homebrew);
        package.version = Some("1.0.0".to_string());

        let id = upsert_package(db.conn(), &package).unwrap();
        assert!(id > 0);

        // Update the package
        package.version = Some("2.0.0".to_string());
        let id2 = upsert_package(db.conn(), &package).unwrap();
        assert_eq!(id, id2); // Should be the same ID

        // Verify version was updated
        let retrieved = get_package_by_name(db.conn(), "test-package", &PackageSource::Homebrew).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().version, Some("2.0.0".to_string()));
    }
}
