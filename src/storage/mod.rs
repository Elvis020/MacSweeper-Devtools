// Storage module - SQLite database for tracking
pub mod database;
pub mod migrations;

use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use dirs;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;
        Ok(Self { conn })
    }

    /// Create a database in the default location (~/.local/share/macsweep/macsweep.db)
    pub fn default() -> Result<Self> {
        let db_path = Self::default_path()?;
        Self::new(db_path)
    }

    /// Get the default database path
    pub fn default_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine local data directory"))?;

        Ok(data_dir.join("macsweep").join("macsweep.db"))
    }

    /// Initialize the database schema
    pub fn init(&self) -> Result<()> {
        migrations::run_migrations(&self.conn)?;
        Ok(())
    }

    /// Get a reference to the connection
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Begin a transaction
    pub fn transaction(&mut self) -> Result<rusqlite::Transaction> {
        Ok(self.conn.transaction()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path().to_path_buf()).unwrap();
        db.init().unwrap();

        // Verify tables were created
        let table_count: i64 = db.conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0)
            )
            .unwrap();

        assert!(table_count >= 3); // packages, usage_events, scans
    }
}
