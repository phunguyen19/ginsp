mod commands;
mod config;
mod credentials;
mod error;
mod utils;
mod view;

use anyhow::{Ok, Result};
use clap::Parser;

use crate::commands::{command_diff, command_update};
use crate::utils::{validate_git, validate_git_repo};

fn main() -> Result<()> {
    let cred = credentials::Credentials::read_credential_file()?;

    println!("{:?}", cred);

    let options = commands::Cli::parse();

    // run validate git for certain commands
    match &options.subcommand {
        // skip these commands
        commands::SubCommand::Version => {}
        // run for the rest
        _ => {
            validate_git()?;
            validate_git_repo()?;
        }
    };

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
