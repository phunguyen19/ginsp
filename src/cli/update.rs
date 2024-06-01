use crate::cli::{Cli, CommandHandler};
use crate::error::GinspError;
use crate::{cli, service};

pub struct Update {}

impl Update {
    pub fn new() -> Self {
        Self {}
    }
}

impl CommandHandler for Update {
    fn execute(&self, cli: &Cli) -> anyhow::Result<()> {
        // get branches from the command line
        let update_cmd = match cli.subcommand {
            cli::SubCommand::Update(ref update_cmd) => update_cmd,
            _ => return Err(GinspError::Cli("Invalid subcommand".to_string()).into()),
        };

        service::git::Git::validate_git_installed()?;

        service::git::Git::validate_git_repo()?;

        if update_cmd.verbose {
            println!("Fetching all branches.");
        }
        service::git::Git::fetch_all()
            .map(|std| {
                if update_cmd.verbose {
                    service::git::Git::print_std(std);
                }
            })
            .map_err(|err| {
                GinspError::Git(format!("Fail to fetch all branches. Error: {}", err))
            })?;

        for branch in update_cmd.branches.iter() {
            if update_cmd.verbose {
                println!("Checking out branch: {}", branch);
            }
            service::git::Git::checkout_branch(branch)
                .map(|std| {
                    if update_cmd.verbose {
                        service::git::Git::print_std(std);
                    }
                })
                .map_err(|err| {
                    GinspError::Git(format!("Fail to checkout branch. Error: {}", err))
                })?;

            if update_cmd.verbose {
                println!("Pulling branch: {}", branch);
            }
            service::git::Git::pull_branch()
                .map(|std| {
                    if update_cmd.verbose {
                        service::git::Git::print_std(std);
                    }
                })
                .map_err(|err| GinspError::Git(format!("Fail to pull branch. Error: {}", err)))?;
        }

        anyhow::Ok(())
    }
}
