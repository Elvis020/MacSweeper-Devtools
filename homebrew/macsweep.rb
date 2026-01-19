class Macsweep < Formula
  desc "Mac package hygiene tool - find and clean unused packages"
  homepage "https://github.com/yourusername/mac-sweeper"
  url "https://github.com/yourusername/mac-sweeper/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "" # Will be filled when creating a release
  license "MIT"
  head "https://github.com/yourusername/mac-sweeper.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  def caveats
    <<~EOS
      MacSweep scans packages from multiple sources:
        - Homebrew (formulae and casks)
        - npm (global packages)
        - pip/pipx (Python packages)
        - cargo (Rust binaries)
        - macOS Applications

      Get started:
        macsweep scan           # Scan your system
        macsweep stats          # View cleanup recommendations
        macsweep clean --dry-run  # Preview cleanup

      Database location:
        ~/Library/Application Support/macsweep/macsweep.db

      For more information:
        macsweep --help
    EOS
  end

  test do
    # Test that the binary runs
    assert_match "MacSweep", shell_output("#{bin}/macsweep --version")

    # Test scan command (dry run)
    system "#{bin}/macsweep", "scan", "--quick"

    # Test list command
    system "#{bin}/macsweep", "list", "--limit", "5"
  end
end
