// CLI module - handles command line interface
pub mod commands;
pub mod output;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "macsweep")]
#[command(about = "Mac Package Hygiene Tool - Find and clean unused packages")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(long, default_value = "table")]
    pub format: OutputFormat,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan system for installed packages
    Scan {
        /// Only scan specific source
        #[arg(long)]
        source: Option<String>,

        /// Skip usage detection (faster)
        #[arg(long)]
        quick: bool,
    },

    /// List packages
    List {
        /// Filter by source
        #[arg(long)]
        source: Option<String>,

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
        source: Option<String>,

        /// Interactive mode - select packages to remove
        #[arg(long, short)]
        interactive: bool,
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

    /// Undo last cleanup operation
    Undo {
        /// Specific backup ID to restore (optional)
        backup_id: Option<String>,

        /// List available backups
        #[arg(long)]
        list: bool,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SortField {
    Name,
    Size,
    LastUsed,
    InstallDate,
    UsageCount,
}

/// Execute the CLI command
pub fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Scan { source, quick } => {
            commands::scan(source, quick)?;
        }
        Commands::List { source, unused, orphaned, large, sort, limit } => {
            commands::list(source, unused, orphaned, large, sort, limit, cli.format)?;
        }
        Commands::Info { package } => {
            commands::info(&package)?;
        }
        Commands::Clean { dry_run, yes, source, interactive } => {
            commands::clean(dry_run, yes, source, interactive)?;
        }
        Commands::History { package } => {
            commands::history(&package)?;
        }
        Commands::Stats => {
            commands::stats()?;
        }
        Commands::Export { output } => {
            commands::export(output)?;
        }
        Commands::Undo { backup_id, list } => {
            commands::undo(backup_id, list)?;
        }
    }
    Ok(())
}
