-- SQLite schema for MacSweep

CREATE TABLE IF NOT EXISTS packages (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    version TEXT,
    binary_path TEXT,
    install_date TEXT,
    first_seen TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

CREATE TABLE IF NOT EXISTS usage_events (
    id INTEGER PRIMARY KEY,
    package_id INTEGER REFERENCES packages(id),
    event_type TEXT NOT NULL,  -- 'shell_history', 'spotlight', 'atime', 'manual'
    event_date TEXT NOT NULL,
    details TEXT,  -- JSON for extra info
    UNIQUE(package_id, event_type, event_date)
);

CREATE TABLE IF NOT EXISTS scans (
    id INTEGER PRIMARY KEY,
    scan_date TEXT DEFAULT CURRENT_TIMESTAMP,
    scan_type TEXT,  -- 'full', 'quick', 'source:homebrew'
    packages_found INTEGER,
    duration_ms INTEGER
);

CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
CREATE INDEX IF NOT EXISTS idx_packages_source ON packages(source);
CREATE INDEX IF NOT EXISTS idx_usage_events_package_id ON usage_events(package_id);
CREATE INDEX IF NOT EXISTS idx_usage_events_event_date ON usage_events(event_date);
