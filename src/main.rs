mod cli;
mod config;
mod error;
mod service;
mod utils;

use anyhow::{Ok, Result};

fn main() -> Result<()> {
    cli::Cli::run()?;
    Ok(())
}
