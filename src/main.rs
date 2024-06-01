mod cli;
mod commands;
mod config;
mod error;
mod service;
mod utils;

use crate::cli::CommandHandler;
use crate::commands::command_diff;
use anyhow::{Ok, Result};
use clap::Parser;

fn main() -> Result<()> {
    let options = cli::Cli::parse();

    match &options.subcommand {
        cli::SubCommand::Version => {
            cli::version::Version::new().execute(&options)?;
        }
        cli::SubCommand::Update(_) => {
            cli::update::Update::new().execute(&options)?;
        }
        cli::SubCommand::DiffMessage(diff_cmd) => {
            command_diff(diff_cmd)?;
        }
        cli::SubCommand::Diagnostic => {
            cli::diagnostic::Diagnostic::new().execute(&options)?;
        }
    }

    Ok(())
}
