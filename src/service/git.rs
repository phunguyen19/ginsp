use crate::error::GinspError;
use std::process::Command;

type ProcessCommandStdout = String;

pub struct Git {}

impl Git {
    pub fn fetch_all() -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["fetch", "--all", "--prune", "--tags"])
    }

    pub fn validate_git_installed() -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["--version"])
    }

    pub fn validate_git_repo() -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["status"])
    }

    pub fn checkout_branch(branch: &str) -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["checkout", branch])
    }

    pub fn pull_branch() -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["pull"])
    }

    pub fn get_current_branch() -> anyhow::Result<ProcessCommandStdout, GinspError> {
        let output = Self::run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let branch = output.trim();
        Ok(branch.trim().to_string())
    }

    pub fn cherry_pick(hash: &String) -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["cherry-pick", hash])
    }

    pub fn cherry_pick_abort() -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["cherry-pick", "--abort"])
    }

    pub fn reset_hard(hash: &String) -> anyhow::Result<ProcessCommandStdout, GinspError> {
        Self::run_git_command(&["reset", "--hard", hash])
    }

    pub fn print_std(stdout: ProcessCommandStdout) {
        println!("{}", stdout);
    }

    fn run_git_command(args: &[&str]) -> anyhow::Result<ProcessCommandStdout, GinspError> {
        let output = Command::new("git")
            .args(args)
            .output()
            .map_err(|err| GinspError::System(err.to_string()))?;

        if output.status.success() {
            Ok(output.stdout.iter().map(|&x| x as char).collect())
        } else {
            let err = String::from_utf8(output.stderr)
                .map_err(|err| GinspError::System(err.to_string()))?;
            Err(GinspError::Git(err))
        }
    }
}
