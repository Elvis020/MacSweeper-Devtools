# MacSweep 🧹

A powerful macOS package hygiene tool written in Rust that helps you identify and remove unused packages, orphaned dependencies, and reclaim disk space.

## Features

### 📦 Multi-Source Package Scanning
- **Homebrew** - Formulae and casks
- **npm** - Global packages
- **pip/pipx** - Python packages
- **cargo** - Rust binaries
- **Applications** - macOS .app bundles

### 🔍 Intelligent Usage Tracking
MacSweep uses multiple data sources to accurately determine when packages were last used:
- **Spotlight Metadata** - For GUI applications (kMDItemLastUsedDate, kMDItemUseCount)
- **Shell History** - Tracks CLI tool usage across zsh, bash, and fish
- **File Access Times** - Fallback for packages without better data

### 🎯 Smart Cleanup Recommendations
- **Orphan Detection** - Finds dependencies no longer needed by any package
- **Usage-Based Analysis** - Identifies packages unused for 30+ days
- **Size-Aware Prioritization** - Helps you recover the most disk space
- **Severity Levels**:
  - **Safe** - Orphaned dependencies (safe to remove)
  - **Review** - Unused 90+ days (should review)
  - **Warning** - Unused 30-90 days (check if needed)

### 💾 Persistent Package Database
- SQLite database tracks all scanned packages
- Historical usage data and installation dates
- Dependency relationships
- Scan history and metadata

### ✨ Interactive Cleanup Wizard
- **Multi-Select Interface** - Pick exactly which packages to remove
- **Visual Indicators** - See severity levels and disk space for each package
- **Safe & Reversible** - Applications moved to Trash (not permanently deleted)
- **Progress Bars** - Real-time feedback during cleanup operations

## Installation

```bash
# Build from source
git clone https://github.com/yourusername/mac-sweeper
cd mac-sweeper
cargo build --release

# Copy to your PATH
cp target/release/macsweep /usr/local/bin/
```

## Usage

### Scan for Packages

```bash
# Full scan across all sources
macsweep scan

# Scan specific source
macsweep scan --source homebrew
macsweep scan --source applications
macsweep scan --source npm

# Quick scan (skip usage tracking)
macsweep scan --quick
```

### List Packages

```bash
# List all packages
macsweep list

# List from specific source
macsweep list --source homebrew

# Find unused packages
macsweep list --unused 30   # Unused for 30+ days
macsweep list --unused 90   # Unused for 90+ days

# Find orphaned dependencies
macsweep list --orphaned

# Sort and limit results
macsweep list --sort size --limit 20
macsweep list --sort last-used --limit 10

# Different output formats
macsweep --format json list
macsweep --format csv list > packages.csv
```

### View Statistics & Recommendations

```bash
# See overall statistics and cleanup recommendations
macsweep stats
```

Example output:
```
📈 MacSweep Statistics

═══ Package Overview ═══
Total packages: 234
Total size: 38.6 GB

Source breakdown:
  Homebrew formulae: 168
  Homebrew casks: 11
  Applications: 51
  npm packages: 4

═══ Usage Statistics ═══
Packages with usage data: 102
Packages without usage data: 132

═══ Cleanup Recommendations ═══
Found 60 cleanup opportunities
Potential space savings: 14.5 GB

Safe to Remove: (2)
  • icu4c@76 - Orphaned dependency (81.1 MB)
  • icu4c@77 - Orphaned dependency (81.0 MB)

Review Recommended: (38)
  • Microsoft Word - Not used in 199 days (~6 months) (2.3 GB)
  • Microsoft Excel - Not used in 546 days (~18 months) (2.0 GB)
  ...
```

### Clean Up Packages

```bash
# Preview what would be removed (dry-run)
macsweep clean --dry-run

# Interactive mode - select packages to remove
macsweep clean --interactive
# or
macsweep clean -i

# Actually remove packages (with confirmation prompt)
macsweep clean

# Skip confirmation prompt
macsweep clean --yes

# Clean specific source only
macsweep clean --source homebrew --dry-run
```

### Backup & Undo

MacSweep automatically creates a backup manifest before every cleanup operation, allowing you to undo changes if needed.

```bash
# Backups are created automatically during cleanup
macsweep clean  # Creates backup before removing packages

# List available backups
macsweep undo --list

# Restore from most recent backup
macsweep undo

# Restore from specific backup
macsweep undo cleanup_20260118_224530
```

**Backup Details:**
- Backup manifests stored in `~/Library/Application Support/macsweep/backups/`
- Each cleanup creates a timestamped JSON manifest
- Undo automatically reinstalls removed packages using their respective package managers
- Applications moved to Trash (can be manually restored from Trash)

### Export Data

```bash
# Export to JSON
macsweep --format json list > packages.json

# Export to CSV
macsweep --format csv list > packages.csv
```

## How It Works

### Usage Detection

MacSweep combines multiple signals to determine when a package was last used:

1. **For GUI Applications** (e.g., Visual Studio Code, Chrome):
   - Queries Spotlight metadata via `mdls` command
   - Reads `kMDItemLastUsedDate` and `kMDItemUseCount`
   - Most accurate for applications launched through Spotlight/Launchpad

2. **For CLI Tools** (e.g., git, npm, cargo):
   - Parses shell history files (~/.zsh_history, ~/.bash_history, ~/.local/share/fish/fish_history)
   - Matches command invocations against package names
   - Counts usage frequency

3. **Fallback** (when other methods fail):
   - Checks file access times (atime/mtime)
   - Less reliable on macOS due to `noatime` optimization

### Package Removal

MacSweep uses the appropriate package manager for each source:

- **Homebrew**: `brew uninstall <package>`
- **npm**: `npm uninstall -g <package>`
- **pip**: `pip3 uninstall -y <package>`
- **pipx**: `pipx uninstall <package>`
- **cargo**: `cargo uninstall <package>`
- **Applications**: Moves to Trash via AppleScript (recoverable!)

## Safety Features

- **Automatic Backups** - Every cleanup creates a backup manifest before removal
- **Undo Support** - Restore removed packages with `macsweep undo`
- **Dry-run mode** - Preview all changes before applying
- **User confirmation** - Prompts before removing packages
- **Interactive selection** - Choose exactly which packages to remove
- **Trash vs Delete** - Applications moved to Trash, not permanently deleted
- **Severity levels** - Clear indication of removal safety (Safe/Review/Warning)
- **Progress tracking** - Real-time feedback during operations

## Database Location

```
~/Library/Application Support/macsweep/macsweep.db
```

## Requirements

- macOS (uses Spotlight, AppleScript, and macOS-specific metadata)
- Rust 1.70+ (for building from source)

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- scan
```

## Example Session

```bash
# Initial scan
$ macsweep scan
🔍 Scanning packages...
  ✓ Homebrew... 179 packages
  ✓ npm (global)... 4 packages
  ✓ pip/pipx... 0 packages
  ✓ cargo... 0 packages
  ✓ Applications... 51 apps

📊 Scan complete: 234 packages found
   └── 168 Homebrew formulae
   └── 11 Homebrew casks
   └── 4 npm global packages
   └── 51 Applications
   └── 38.6 GB total

🔎 Gathering usage information...
  Usage tracking complete in 3.94s

💾 Saving to database... done

# Check stats
$ macsweep stats
📈 MacSweep Statistics
...
Found 60 cleanup opportunities
Potential space savings: 14.5 GB

# Preview cleanup
$ macsweep clean --dry-run
🧹 MacSweep Cleanup

Packages to remove:
  Total: 60
  Potential space savings: 14.5 GB

[DRY RUN MODE] - No packages will be removed
...

# Remove orphans only
$ macsweep clean --source homebrew --dry-run
Would remove: 2 packages
Would recover: 162.0 MB
```

## Roadmap

- [x] Interactive cleanup wizard ✅
- [x] Backup/undo support for cleanups ✅
- [x] Comprehensive test suite ✅
- [ ] Dependency graph visualization
- [ ] Homebrew formula for easy installation
- [ ] Additional package managers (MacPorts, mas)
- [ ] Web UI for package management

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
- [rusqlite](https://github.com/rusqlite/rusqlite) for database
- [indicatif](https://github.com/console-rs/indicatif) for progress bars
- [comfy-table](https://github.com/Nukesor/comfy-table) for beautiful tables

---

**Made with ❤️ and Rust**
