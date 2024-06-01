mod cli;
mod config;
mod error;
mod service;

use anyhow::{Ok, Result};

fn main() -> Result<()> {
    cli::Cli::run()?;
    Ok(())
}
