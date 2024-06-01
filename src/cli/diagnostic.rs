use crate::cli::{Cli, CommandHandler};
use crate::config::Config;
use crate::git;

pub struct Diagnostic {}

impl Diagnostic {
    pub fn new() -> Self {
        Self {}
    }
}

impl CommandHandler for Diagnostic {
    fn execute(&self, _cli: &Cli) -> anyhow::Result<()> {
        git::Git::validate_git_installed()?;
        println!("Git is installed.");
        git::Git::validate_git_repo()?;
        println!("Git repository is valid.");
        let _ = Config::read_config_file_from_home_dir()?;
        println!("Config file is valid.");

        // TODO: diagnostic for project management tool
        println!("Project management tool (skipped).");

        println!("Diagnostic done.");
        Ok(())
    }
}
