mod cli;
mod config;
mod error;
mod git;
mod jira;

use anyhow::{Ok, Result};

fn main() -> Result<()> {
    cli::Cli::run()?;
    Ok(())
}
