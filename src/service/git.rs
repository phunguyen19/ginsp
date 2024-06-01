use crate::error::GinspError;
use std::process::{Command, Output};

pub struct Git {}

impl Git {
    pub fn fetch_all() -> anyhow::Result<Output, GinspError> {
        Self::run_git_command(&["fetch", "--all", "--prune", "--tags"])
    }

    pub fn validate_git_installed() -> anyhow::Result<Output, GinspError> {
        Self::run_git_command(&["--version"])
    }

    pub fn validate_git_repo() -> anyhow::Result<Output, GinspError> {
        Self::run_git_command(&["status"])
    }

    pub fn checkout_branch(branch: &str) -> anyhow::Result<Output, GinspError> {
        Self::run_git_command(&["checkout", branch])
    }

    pub fn pull_branch() -> anyhow::Result<Output, GinspError> {
        Self::run_git_command(&["pull"])
    }

    pub fn get_current_branch() -> anyhow::Result<String, GinspError> {
        let output = Self::run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let branch =
            String::from_utf8(output.stdout).map_err(|err| GinspError::System(err.to_string()))?;
        Ok(branch.trim().to_string())
    }

    fn run_git_command(args: &[&str]) -> anyhow::Result<Output, GinspError> {
        let output = Command::new("git")
            .args(args)
            .output()
            .map_err(|err| GinspError::System(err.to_string()))?;

        if output.status.success() {
            Ok(output)
        } else {
            let err = String::from_utf8(output.stderr)
                .map_err(|err| GinspError::System(err.to_string()))?;
            Err(GinspError::Git(err))
        }
    }
}
