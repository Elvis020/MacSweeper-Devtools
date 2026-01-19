use anyhow::Result;
use clap::Parser;

mod cli;
mod scanner;
mod usage;
mod analysis;
mod storage;
mod cleanup;
mod utils;

use cli::Cli;

fn main() -> Result<()> {
    // Initialize tracing/logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Execute the command
    cli::execute(cli)?;

    Ok(())
}
