mod cli;
mod commands;
mod config;
mod error;
mod service;
mod utils;

use crate::cli::CommandHandler;
use crate::commands::{command_diff, command_update};
use anyhow::{Ok, Result};
use clap::Parser;

fn main() -> Result<()> {
    let options = cli::Cli::parse();

    match &options.subcommand {
        cli::SubCommand::Version => {
            cli::version::Version::new().execute(&options)?;
        }
        cli::SubCommand::Update(update_cmd) => {
            command_update(update_cmd.branches.clone())?;
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
