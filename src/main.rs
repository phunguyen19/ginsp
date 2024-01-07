mod commands;
mod config;
mod error;
mod utils;
mod view;

use anyhow::{Ok, Result};
use clap::Parser;

use crate::commands::{command_diff, command_update};
use crate::utils::exit_with_error;
use crate::view::print_result;

fn main() -> Result<()> {
    let options = commands::Cli::parse();

    match &options.subcommand {
        commands::SubCommand::Version => {
            println!("ginsp version {}", env!("CARGO_PKG_VERSION"));
        }
        commands::SubCommand::Update(update_cmd) => {
            command_update(update_cmd.branches.clone())?;
        }
        commands::SubCommand::DiffMessage(diff_cmd) => {
            let commands::DiffResult {
                source_branch,
                unique_to_source,
                target_branch,
                unique_to_target,
            } = command_diff(diff_cmd)?;

            print_result(&source_branch, unique_to_source);
            print_result(&target_branch, unique_to_target);
        }
    }

    Ok(())
}
