// File access time detection
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;

/// Get the file access time (atime) for a binary
/// Note: On macOS, atime may not be reliable as it can be disabled with noatime mount option
pub fn get_binary_atime(binary_path: &Path) -> Result<Option<DateTime<Utc>>> {
    if !binary_path.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(binary_path)?;

    // Get the accessed time
    match metadata.accessed() {
        Ok(system_time) => {
            let datetime: DateTime<Utc> = system_time.into();
            Ok(Some(datetime))
        }
        Err(_) => {
            // atime might not be available
            Ok(None)
        }
    }
}

/// Get the file modification time (mtime) as a fallback
/// This is more reliable than atime on macOS
pub fn get_binary_mtime(binary_path: &Path) -> Result<Option<DateTime<Utc>>> {
    if !binary_path.exists() {
        return Ok(None);
    }

    let metadata = fs::metadata(binary_path)?;

    match metadata.modified() {
        Ok(system_time) => {
            let datetime: DateTime<Utc> = system_time.into();
            Ok(Some(datetime))
        }
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    #[ignore] // Run manually
    fn test_get_binary_atime() {
        let path = PathBuf::from("/usr/bin/git");
        if path.exists() {
            let atime = get_binary_atime(&path).unwrap();
            println!("atime: {:?}", atime);

            let mtime = get_binary_mtime(&path).unwrap();
            println!("mtime: {:?}", mtime);
        }
    }
}
