# MacSweep Release Guide

This guide walks you through publishing MacSweep to GitHub and making it installable via Homebrew.

## Prerequisites

- GitHub account (Elvis020)
- Git configured with your credentials
- GitHub CLI (`gh`) installed (optional but recommended)

## Step 1: Create GitHub Repository

### Option A: Using GitHub Web Interface

1. Go to https://github.com/new
2. Repository name: `mac-sweeper`
3. Description: `Mac Package Hygiene Tool - Find and clean unused packages`
4. Visibility: **Public**
5. **Do NOT** initialize with README, .gitignore, or license (we already have these)
6. Click "Create repository"

### Option B: Using GitHub CLI (Recommended)

```bash
# Install gh if you haven't already
brew install gh

# Authenticate with GitHub
gh auth login

# Create the repository
gh repo create mac-sweeper \
  --public \
  --description "Mac Package Hygiene Tool - Find and clean unused packages" \
  --source=. \
  --remote=origin \
  --push
```

## Step 2: Push Code to GitHub

### If you used Option A (Web Interface):

```bash
# Add the remote
git remote add origin https://github.com/Elvis020/mac-sweeper.git

# Rename branch to main (if it's currently master)
git branch -M main

# Push to GitHub
git push -u origin main
```

### If you used Option B (GitHub CLI):

The code is already pushed! Skip to Step 3.

## Step 3: Create v0.1.0 Release

### Option A: Using GitHub Web Interface

1. Go to https://github.com/Elvis020/mac-sweeper/releases/new
2. Tag version: `v0.1.0`
3. Target: `main`
4. Release title: `MacSweep v0.1.0 - Initial Release`
5. Description:

```markdown
# MacSweep v0.1.0 🧹

First public release of MacSweep, a comprehensive macOS package hygiene tool.

## Features

- 📦 Multi-source package scanning (Homebrew, npm, pip/pipx, cargo, Applications)
- 🔍 Intelligent usage tracking via Spotlight metadata, shell history, and file access times
- 🎯 Smart cleanup recommendations with severity levels (Safe/Review/Warning)
- ✨ Interactive cleanup wizard with multi-select interface
- 💾 Automatic backup/undo support for all cleanup operations
- 📊 Beautiful progress bars and table formatting
- 📤 JSON/CSV export capabilities

## Installation

### From Source

```bash
git clone https://github.com/Elvis020/mac-sweeper.git
cd mac-sweeper
cargo build --release
cp target/release/macsweep /usr/local/bin/
```

### Via Homebrew (Coming Soon)

```bash
brew tap Elvis020/tap
brew install macsweep
```

## Quick Start

```bash
# Scan your system
macsweep scan

# View recommendations
macsweep stats

# Preview cleanup
macsweep clean --dry-run

# Interactive cleanup
macsweep clean --interactive
```

## Documentation

See [README.md](https://github.com/Elvis020/mac-sweeper/blob/main/README.md) for complete documentation.

## What's Next

- Dependency graph visualization
- Web UI for package management
- Support for additional package managers (MacPorts, mas)
```

6. Click "Publish release"

### Option B: Using GitHub CLI

```bash
gh release create v0.1.0 \
  --title "MacSweep v0.1.0 - Initial Release" \
  --notes-file - <<EOF
# MacSweep v0.1.0 🧹

First public release of MacSweep, a comprehensive macOS package hygiene tool.

## Features

- 📦 Multi-source package scanning (Homebrew, npm, pip/pipx, cargo, Applications)
- 🔍 Intelligent usage tracking via Spotlight metadata, shell history, and file access times
- 🎯 Smart cleanup recommendations with severity levels
- ✨ Interactive cleanup wizard with multi-select interface
- 💾 Automatic backup/undo support
- 📊 Beautiful progress bars and table formatting

## Installation

See [README.md](https://github.com/Elvis020/mac-sweeper/blob/main/README.md) for installation instructions.
EOF
```

## Step 4: Calculate SHA256 Hash for Homebrew

After creating the release, GitHub automatically generates a source code tarball. Calculate its SHA256 hash:

```bash
# Download and hash the release tarball
curl -L https://github.com/Elvis020/mac-sweeper/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
```

Copy the resulting hash (it will be a long hex string like `abc123def456...`).

## Step 5: Update Homebrew Formula

Edit `homebrew/macsweep.rb` and replace the empty sha256 line:

```ruby
# Before:
sha256 ""

# After (use your actual hash):
sha256 "your_calculated_hash_here"
```

Commit this change:

```bash
git add homebrew/macsweep.rb
git commit -m "Add SHA256 hash to Homebrew formula"
git push origin main
```

## Step 6: Create Homebrew Tap Repository

A Homebrew "tap" is a repository of formulae. Create one:

### Using GitHub Web Interface:

1. Go to https://github.com/new
2. Repository name: `homebrew-tap` (must start with `homebrew-`)
3. Description: `Homebrew formulae for Elvis020's tools`
4. Visibility: **Public**
5. Initialize with README: **Yes**
6. Click "Create repository"

### Using GitHub CLI:

```bash
gh repo create homebrew-tap \
  --public \
  --description "Homebrew formulae for Elvis020's tools" \
  --clone
```

## Step 7: Publish Formula to Tap

```bash
# Clone your tap repository (if you used web interface)
cd ..
git clone https://github.com/Elvis020/homebrew-tap.git
cd homebrew-tap

# Create Formula directory
mkdir -p Formula

# Copy the formula
cp ../mac-sweeper/homebrew/macsweep.rb Formula/

# Commit and push
git add Formula/macsweep.rb
git commit -m "Add macsweep formula"
git push origin main
```

## Step 8: Test Installation

Now anyone can install MacSweep via Homebrew:

```bash
# Add your tap
brew tap Elvis020/tap

# Install macsweep
brew install macsweep

# Test it
macsweep --version
macsweep scan --quick
```

## Step 9: Update README with Installation Instructions

Update the Installation section in README.md:

```markdown
## Installation

### Via Homebrew (Recommended)

```bash
brew tap Elvis020/tap
brew install macsweep
```

### From Source

```bash
git clone https://github.com/Elvis020/mac-sweeper.git
cd mac-sweeper
cargo build --release
cp target/release/macsweep /usr/local/bin/
```

### From Cargo

```bash
cargo install --git https://github.com/Elvis020/mac-sweeper
```
```

Commit and push this change:

```bash
git add README.md
git commit -m "Update installation instructions with Homebrew tap"
git push origin main
```

## Troubleshooting

### Formula Audit Fails

Run Homebrew's audit tool to check for issues:

```bash
brew audit --new-formula Formula/macsweep.rb
```

### Installation Fails

Common issues:
- **SHA256 mismatch**: Recalculate the hash and update the formula
- **Build fails**: Ensure Rust version requirements are met
- **Tests fail**: Check that the binary path is correct in the test block

### Testing Formula Locally

Before publishing to tap:

```bash
# Install directly from the formula file
brew install --build-from-source ./homebrew/macsweep.rb

# Uninstall
brew uninstall macsweep
```

## Next Steps

1. **Submit to Homebrew Core** (optional): After the tap is stable, you can submit to the official Homebrew repository
2. **Publish to crates.io**: Make it installable via `cargo install macsweep`
3. **Create distribution packages**: Consider .pkg installer for macOS
4. **Set up CI/CD**: Automate testing and releases with GitHub Actions

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Creating GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [Publishing to crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
