mod commands;
mod config;
mod error;
mod service;
mod utils;

use anyhow::{Ok, Result};
use clap::Parser;

use crate::commands::{command_diff, command_update};

fn main() -> Result<()> {
    let options = commands::Cli::parse();

    match &options.subcommand {
        commands::SubCommand::Version => {
            commands::command_version()?;
        }
        commands::SubCommand::Update(update_cmd) => {
            command_update(update_cmd.branches.clone())?;
        }
        commands::SubCommand::DiffMessage(diff_cmd) => {
            command_diff(diff_cmd)?;
        }
    }

    Ok(())
}
