// Database schema migrations
use anyhow::Result;
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<()> {
    create_packages_table(conn)?;
    create_package_dependencies_table(conn)?;
    create_usage_events_table(conn)?;
    create_scans_table(conn)?;
    create_cleanups_table(conn)?;
    create_indexes(conn)?;
    Ok(())
}

fn create_packages_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS packages (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            source TEXT NOT NULL,
            version TEXT,
            binary_path TEXT,
            install_date TEXT,
            size_bytes INTEGER,
            is_dependency BOOLEAN DEFAULT 0,
            last_used TEXT,
            usage_count INTEGER DEFAULT 0,
            first_seen TEXT DEFAULT CURRENT_TIMESTAMP,
            last_seen TEXT DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(name, source)
        )",
        [],
    )?;
    Ok(())
}

fn create_package_dependencies_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS package_dependencies (
            id INTEGER PRIMARY KEY,
            package_id INTEGER NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
            dependency_name TEXT NOT NULL,
            UNIQUE(package_id, dependency_name)
        )",
        [],
    )?;
    Ok(())
}

fn create_usage_events_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS usage_events (
            id INTEGER PRIMARY KEY,
            package_id INTEGER REFERENCES packages(id) ON DELETE CASCADE,
            event_type TEXT NOT NULL,
            event_date TEXT NOT NULL,
            details TEXT,
            UNIQUE(package_id, event_type, event_date)
        )",
        [],
    )?;
    Ok(())
}

fn create_scans_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scans (
            id INTEGER PRIMARY KEY,
            scan_date TEXT DEFAULT CURRENT_TIMESTAMP,
            scan_type TEXT,
            packages_found INTEGER,
            duration_ms INTEGER
        )",
        [],
    )?;
    Ok(())
}

fn create_cleanups_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS cleanups (
            id INTEGER PRIMARY KEY,
            cleanup_date TEXT DEFAULT CURRENT_TIMESTAMP,
            backup_manifest_path TEXT NOT NULL,
            packages_removed INTEGER,
            space_recovered INTEGER,
            can_undo BOOLEAN DEFAULT 1
        )",
        [],
    )?;
    Ok(())
}

fn create_indexes(conn: &Connection) -> Result<()> {
    // Index for package lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_packages_source ON packages(source)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_packages_last_used ON packages(last_used)",
        [],
    )?;

    // Index for usage events
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_usage_events_package_id ON usage_events(package_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_usage_events_event_date ON usage_events(event_date)",
        [],
    )?;

    // Index for dependencies
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_package_dependencies_package_id ON package_dependencies(package_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_package_dependencies_dependency_name ON package_dependencies(dependency_name)",
        [],
    )?;

    Ok(())
}
