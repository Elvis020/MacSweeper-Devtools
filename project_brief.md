# MacSweep - Mac Package Hygiene Tool

## Project Overview

Build a command-line tool (with potential GUI later) that scans a macOS system for installed packages across multiple package managers, tracks their usage, identifies orphaned/unused packages, and helps users safely clean up their system.

## Problem Statement

Developers accumulate packages over time from Homebrew, npm, pip, cargo, etc. Most don't know:
- What packages they have installed
- When they last used each package
- Which packages are orphaned dependencies
- How much disk space they could recover

MacSweep solves this by providing visibility and actionable cleanup recommendations.

## Tech Stack

- **Language:** Rust (fast, single binary, great for CLI tools)
- **CLI Framework:** `clap` (argument parsing)
- **TUI (optional):** `ratatui` for interactive terminal UI
- **Database:** `rusqlite` (SQLite for tracking usage history)
- **Output:** `tabled` or `comfy-table` for pretty tables
- **JSON:** `serde` + `serde_json` for structured output
- **Async:** `tokio` for parallel scanning
- **Colors:** `colored` or `owo-colors` for terminal colors

## Core Features

### 1. Package Discovery

Scan and aggregate packages from multiple sources:

```rust
enum PackageSource {
    Homebrew,           // brew list --formula
    HomebrewCask,       // brew list --cask
    MacAppStore,        // mas list (if installed)
    Npm,                // npm list -g --depth=0
    Pip,                // pip list / pip3 list
    Pipx,               // pipx list
    Cargo,              // ~/.cargo/bin + cargo install --list
    Gem,                // gem list
    Go,                 // ~/go/bin
    Composer,           // composer global show
    Applications,       // /Applications/*.app
    LocalBin,           // /usr/local/bin, ~/.local/bin
}

struct Package {
    name: String,
    version: Option<String>,
    source: PackageSource,
    install_date: Option<DateTime<Utc>>,
    size_bytes: Option<u64>,
    binary_path: Option<PathBuf>,
    is_dependency: bool,
    dependencies: Vec<String>,
    dependents: Vec<String>,      // packages that depend on this
    last_used: Option<DateTime<Utc>>,
    usage_count: u32,             // from shell history
    usage_sources: Vec<UsageSource>,
}

enum UsageSource {
    ShellHistory { count: u32, last_used: DateTime<Utc> },
    SpotlightMetadata { last_used: DateTime<Utc> },
    FileAccessTime { atime: DateTime<Utc> },
    Manual,  // user marked as used
}
```

### 2. Usage Detection

Implement multiple strategies to detect package usage:

#### a) Shell History Analysis
```rust
// Parse shell history files
// ~/.zsh_history
// ~/.bash_history
// ~/.local/share/fish/fish_history

fn parse_shell_history(history_path: &Path) -> Vec<HistoryEntry> {
    // Extract commands and timestamps
    // Match against known package binaries
    // Count frequency and last usage
}
```

#### b) Spotlight Metadata (for GUI apps)
```rust
// Use mdls command to get last used date
// mdls -name kMDItemLastUsedDate /Applications/SomeApp.app

fn get_spotlight_last_used(app_path: &Path) -> Option<DateTime<Utc>> {
    let output = Command::new("mdls")
        .args(["-name", "kMDItemLastUsedDate", "-raw"])
        .arg(app_path)
        .output()?;
    // Parse output
}
```

#### c) File Access Time
```rust
// Get atime from file metadata
// Note: macOS may have atime disabled (noatime mount option)

fn get_binary_atime(binary_path: &Path) -> Option<DateTime<Utc>> {
    let metadata = fs::metadata(binary_path)?;
    let atime = metadata.accessed()?;
    Some(DateTime::from(atime))
}
```

#### d) Homebrew-specific
```rust
// Parse Homebrew's install receipts
// /usr/local/Cellar/*/INSTALL_RECEIPT.json
// Or: /opt/homebrew/Cellar/*/INSTALL_RECEIPT.json (Apple Silicon)

fn get_brew_install_date(formula: &str) -> Option<DateTime<Utc>> {
    // Read INSTALL_RECEIPT.json
    // Extract "time" field
}
```

### 3. Orphan Detection

```rust
// For Homebrew
fn get_orphaned_brew_packages() -> Vec<String> {
    // Run: brew autoremove --dry-run
    // Parse output for packages that would be removed
}

fn get_brew_leaves() -> Vec<String> {
    // Run: brew leaves
    // These are top-level packages (not dependencies)
}

fn analyze_dependency_tree(packages: &[Package]) -> DependencyAnalysis {
    // Build dependency graph
    // Find packages with no dependents (leaves)
    // Find packages that are deps but whose parent is uninstalled
}
```

### 4. Disk Space Analysis

```rust
fn calculate_package_size(package: &Package) -> u64 {
    match package.source {
        PackageSource::Homebrew => {
            // du -sh /usr/local/Cellar/{package}
            // Or parse: brew info --json {package}
        },
        PackageSource::HomebrewCask => {
            // Size of /Applications/{app}.app
        },
        PackageSource::Npm => {
            // Size of global node_modules
        },
        // ... etc
    }
}
```

### 5. Local Database for Tracking

```sql
-- SQLite schema for tracking usage over time

CREATE TABLE packages (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL,
    version TEXT,
    binary_path TEXT,
    install_date TEXT,
    first_seen TEXT DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, source)
);

CREATE TABLE usage_events (
    id INTEGER PRIMARY KEY,
    package_id INTEGER REFERENCES packages(id),
    event_type TEXT NOT NULL,  -- 'shell_history', 'spotlight', 'atime', 'manual'
    event_date TEXT NOT NULL,
    details TEXT,  -- JSON for extra info
    UNIQUE(package_id, event_type, event_date)
);

CREATE TABLE scans (
    id INTEGER PRIMARY KEY,
    scan_date TEXT DEFAULT CURRENT_TIMESTAMP,
    scan_type TEXT,  -- 'full', 'quick', 'source:homebrew'
    packages_found INTEGER,
    duration_ms INTEGER
);
```

## CLI Interface

### Commands

```bash
# Main commands
macsweep scan              # Scan all package sources
macsweep scan --source brew # Scan only Homebrew
macsweep list              # List all tracked packages
macsweep list --unused 30  # Packages not used in 30 days
macsweep list --orphaned   # Show orphaned dependencies
macsweep list --large      # Sort by disk space
macsweep info <package>    # Detailed info about a package
macsweep clean             # Interactive cleanup wizard
macsweep clean --dry-run   # Show what would be removed
macsweep history <package> # Show usage history
macsweep stats             # Summary statistics
macsweep export            # Export to JSON
macsweep daemon start      # Start background usage tracking (future)
```

### Command Arguments

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "macsweep")]
#[command(about = "Mac Package Hygiene Tool - Find and clean unused packages")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(long, default_value = "table")]
    format: OutputFormat,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Scan system for installed packages
    Scan {
        /// Only scan specific source
        #[arg(long)]
        source: Option<PackageSource>,

        /// Skip usage detection (faster)
        #[arg(long)]
        quick: bool,
    },

    /// List packages
    List {
        /// Filter by source
        #[arg(long)]
        source: Option<PackageSource>,

        /// Show packages unused for N days
        #[arg(long)]
        unused: Option<u32>,

        /// Show only orphaned packages
        #[arg(long)]
        orphaned: bool,

        /// Sort by size (largest first)
        #[arg(long)]
        large: bool,

        /// Sort by: name, size, last_used, install_date
        #[arg(long, default_value = "name")]
        sort: SortField,

        /// Limit results
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Show package details
    Info {
        package: String,
    },

    /// Interactive cleanup
    Clean {
        /// Dry run - show what would be removed
        #[arg(long)]
        dry_run: bool,

        /// Auto-confirm (dangerous!)
        #[arg(long)]
        yes: bool,

        /// Only clean specific source
        #[arg(long)]
        source: Option<PackageSource>,
    },

    /// Show usage history for a package
    History {
        package: String,
    },

    /// Show summary statistics
    Stats,

    /// Export data
    Export {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum SortField {
    Name,
    Size,
    LastUsed,
    InstallDate,
    UsageCount,
}
```

### Example Output

```
$ macsweep scan
🔍 Scanning package sources...
  ✓ Homebrew (145 packages)
  ✓ Homebrew Casks (32 packages)
  ✓ npm global (18 packages)
  ✓ pip (45 packages)
  ✓ cargo (12 packages)
  ✓ Applications (89 apps)

📊 Scan complete: 341 packages found
   └── 23 orphaned dependencies
   └── 47 not used in 30+ days
   └── 12.4 GB potentially recoverable

Run `macsweep list --unused 30` to see unused packages.
```

```
$ macsweep list --unused 30 --limit 10

┌─────────────────┬──────────┬─────────┬─────────────┬───────────┐
│ Package         │ Source   │ Size    │ Last Used   │ Status    │
├─────────────────┼──────────┼─────────┼─────────────┼───────────┤
│ imagemagick     │ Homebrew │ 245 MB  │ 89 days ago │ 🟡 Unused │
│ php@8.1         │ Homebrew │ 180 MB  │ 67 days ago │ 🟡 Unused │
│ mysql           │ Homebrew │ 420 MB  │ 45 days ago │ 🟡 Unused │
│ create-react-app│ npm      │ 12 MB   │ 120 days ago│ 🔴 Stale  │
│ virtualenv      │ pip      │ 8 MB    │ 90 days ago │ 🟡 Unused │
│ ...             │          │         │             │           │
└─────────────────┴──────────┴─────────┴─────────────┴───────────┘

Total: 47 packages | 2.1 GB recoverable
```

```
$ macsweep info imagemagick

📦 imagemagick
   Source:       Homebrew
   Version:      7.1.1-21
   Install Date: 2024-03-15
   Size:         245 MB
   Binary:       /opt/homebrew/bin/magick

📊 Usage
   Shell History: 3 invocations (last: 89 days ago)
   File Access:   92 days ago

🔗 Dependencies (12)
   jpeg-turbo, libpng, libtiff, libheif, webp...

⚠️  Status: Unused for 89 days

💡 Recommendation: Consider removing with `brew uninstall imagemagick`
```

```
$ macsweep clean --dry-run

🧹 Cleanup Recommendations

ORPHANED DEPENDENCIES (safe to remove):
  • libyaml (8 MB) - no longer needed
  • readline (2 MB) - no longer needed
  • xz (1 MB) - no longer needed

UNUSED 90+ DAYS (review recommended):
  • imagemagick (245 MB) - last used 89 days ago
  • php@8.1 (180 MB) - last used 67 days ago

CACHE/TEMP FILES:
  • Homebrew cache (1.2 GB)
  • npm cache (340 MB)
  • pip cache (89 MB)

Total recoverable: 2.1 GB

Run `macsweep clean` to proceed with cleanup.
```

## Project Structure

```
macsweep/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── lib.rs               # Library exports
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs      # Command definitions
│   │   └── output.rs        # Formatting (table, json, csv)
│   ├── scanner/
│   │   ├── mod.rs
│   │   ├── homebrew.rs      # Homebrew scanning
│   │   ├── npm.rs           # npm global packages
│   │   ├── pip.rs           # pip/pip3/pipx
│   │   ├── cargo.rs         # Cargo binaries
│   │   ├── applications.rs  # /Applications scanning
│   │   ├── gem.rs           # Ruby gems
│   │   └── generic.rs       # Generic binary scanning
│   ├── usage/
│   │   ├── mod.rs
│   │   ├── shell_history.rs # Parse zsh/bash/fish history
│   │   ├── spotlight.rs     # macOS Spotlight metadata
│   │   ├── atime.rs         # File access times
│   │   └── aggregator.rs    # Combine usage sources
│   ├── analysis/
│   │   ├── mod.rs
│   │   ├── orphans.rs       # Orphan detection
│   │   ├── dependencies.rs  # Dependency graph
│   │   └── recommendations.rs # Cleanup suggestions
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── database.rs      # SQLite operations
│   │   └── migrations.rs    # DB schema migrations
│   ├── cleanup/
│   │   ├── mod.rs
│   │   ├── executor.rs      # Execute removal commands
│   │   └── backup.rs        # Backup before removal
│   └── utils/
│       ├── mod.rs
│       ├── size.rs          # Disk size calculations
│       ├── date.rs          # Date formatting
│       └── process.rs       # Run shell commands
├── tests/
│   ├── scanner_tests.rs
│   ├── usage_tests.rs
│   └── integration_tests.rs
└── resources/
    └── schema.sql           # SQLite schema
```

## Cargo.toml

```toml
[package]
name = "macsweep"
version = "0.1.0"
edition = "2021"
description = "Mac Package Hygiene Tool - Find and clean unused packages"
license = "MIT"
repository = "https://github.com/yourusername/macsweep"
keywords = ["macos", "homebrew", "cleanup", "packages", "cli"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
# CLI
clap = { version = "4", features = ["derive", "cargo"] }
colored = "2"
comfy-table = "7"
indicatif = "0.17"          # Progress bars
dialoguer = "0.11"          # Interactive prompts

# Async
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
csv = "1"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# File system
walkdir = "2"
dirs = "5"                   # Standard directories
which = "6"                  # Find binaries

# Error handling
anyhow = "1"
thiserror = "1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Parsing
regex = "1"
lazy_static = "1"

[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"

[[bin]]
name = "macsweep"
path = "src/main.rs"
```

## Implementation Phases

### Phase 1: Core MVP (Week 1)
- [ ] Project setup, CLI skeleton
- [ ] Homebrew scanner (formula + casks)
- [ ] Basic list command with table output
- [ ] Shell history parsing (zsh)
- [ ] SQLite storage

### Phase 2: Extended Scanning (Week 2)
- [ ] npm, pip, cargo scanners
- [ ] Applications folder scanning
- [ ] Spotlight metadata integration
- [ ] Disk size calculations

### Phase 3: Analysis (Week 3)
- [ ] Orphan detection
- [ ] Dependency graph building
- [ ] Usage aggregation from multiple sources
- [ ] Recommendations engine

### Phase 4: Cleanup (Week 4)
- [ ] Dry-run cleanup
- [ ] Safe removal execution
- [ ] Backup/undo support
- [ ] Interactive cleanup wizard

### Phase 5: Polish (Week 5)
- [ ] Progress bars and better UX
- [ ] JSON/CSV export
- [ ] Comprehensive tests
- [ ] Documentation
- [ ] Homebrew formula for distribution

## Future Enhancements

- **GUI App**: Tauri-based desktop app
- **Background Daemon**: Track usage in real-time
- **Scheduled Reports**: Weekly email/notification summaries
- **Cloud Sync**: Track across multiple Macs
- **Team Features**: Share cleanup configs with team
- **AI Recommendations**: Smart suggestions based on project type

## Distribution

```bash
# Install via Homebrew (future)
brew install macsweep

# Or via cargo
cargo install macsweep

# Or download binary from releases
curl -sSL https://github.com/user/macsweep/releases/latest/download/macsweep-macos -o macsweep
chmod +x macsweep
sudo mv macsweep /usr/local/bin/
```

## Example Usage Flow

```bash
# First time setup
macsweep scan

# Check what's unused
macsweep list --unused 60

# Get details on a specific package
macsweep info imagemagick

# See cleanup recommendations
macsweep clean --dry-run

# Execute cleanup (interactive)
macsweep clean

# Export for backup/review
macsweep export -o packages.json

# Quick stats
macsweep stats
```

## Notes for Implementation

1. **macOS Permissions**: Some features may require Full Disk Access (System Preferences → Security & Privacy → Privacy → Full Disk Access)

2. **Apple Silicon vs Intel**: Homebrew paths differ:
   - Intel: `/usr/local/Cellar`, `/usr/local/bin`
   - Apple Silicon: `/opt/homebrew/Cellar`, `/opt/homebrew/bin`

3. **Shell History Formats**:
   - zsh: `: timestamp:0;command` format
   - bash: plain commands (no timestamps unless HISTTIMEFORMAT set)
   - fish: YAML-like format

4. **Rate Limiting**: Be careful with commands like `brew info` - batch where possible

5. **Caching**: Cache expensive operations (disk size, dependency trees) in SQLite

---

Good luck building MacSweep! This could be a really useful tool for the developer community. 🚀
