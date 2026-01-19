#!/bin/bash
# Setup script for creating and publishing MacSweep Homebrew tap

set -e

echo "🍺 MacSweep Homebrew Tap Setup"
echo "================================"
echo ""

# Check if we're in the right directory
if [ ! -f "homebrew/macsweep.rb" ]; then
    echo "❌ Error: Please run this script from the mac-sweeper directory"
    exit 1
fi

echo "Step 1: Creating homebrew-tap repository..."
echo ""
echo "Please create a new GitHub repository with these settings:"
echo "  - Repository name: homebrew-tap"
echo "  - Visibility: Public"
echo "  - Do NOT initialize with README"
echo ""
echo "Create it at: https://github.com/new"
echo ""
read -p "Press Enter once you've created the repository..."

# Clone into parent directory
cd ..
echo ""
echo "Step 2: Cloning homebrew-tap repository..."
git clone git@github.com:Elvis020/homebrew-tap.git
cd homebrew-tap

echo ""
echo "Step 3: Setting up Formula directory..."
mkdir -p Formula

echo ""
echo "Step 4: Copying macsweep formula..."
cp ../mac-sweeper/homebrew/macsweep.rb Formula/

echo ""
echo "Step 5: Creating README..."
cat > README.md <<'EOF'
# Elvis020's Homebrew Tap

Custom Homebrew formulae for macOS development tools.

## Installation

```bash
brew tap Elvis020/tap
```

## Available Formulae

### MacSweep

Mac Package Hygiene Tool - Find and clean unused packages

```bash
brew install macsweep
```

MacSweep helps you identify and remove unused packages across multiple package managers including Homebrew, npm, pip, cargo, and macOS Applications.

**Features:**
- Multi-source package scanning
- Intelligent usage tracking via Spotlight metadata and shell history
- Smart cleanup recommendations with severity levels
- Interactive package selection wizard
- Automatic backup/undo support
- Beautiful progress bars and table output

**Documentation:** https://github.com/Elvis020/MacSweeper-Devtools

## Usage

After installation:

```bash
# Scan your system
macsweep scan

# View recommendations
macsweep stats

# Interactive cleanup
macsweep clean --interactive
```

## Troubleshooting

If you encounter any issues:

1. Update the tap: `brew update`
2. Reinstall: `brew reinstall macsweep`
3. Report issues: https://github.com/Elvis020/MacSweeper-Devtools/issues

## Contributing

Contributions welcome! Please submit pull requests to the respective formula repositories.
EOF

echo ""
echo "Step 6: Committing and pushing to GitHub..."
git add .
git commit -m "Add macsweep formula and initial README"
git push origin main

echo ""
echo "✅ Homebrew tap setup complete!"
echo ""
echo "🎉 Installation Instructions"
echo "============================"
echo ""
echo "Anyone can now install MacSweep with:"
echo ""
echo "  brew tap Elvis020/tap"
echo "  brew install macsweep"
echo ""
echo "Test it yourself:"
echo ""
echo "  cd ../mac-sweeper"
echo "  brew tap Elvis020/tap"
echo "  brew install macsweep"
echo "  macsweep --version"
echo ""
