mod utils;
mod config;
mod error;

use anyhow::{Ok, Result};
use clap::Parser;
use indexmap::indexmap;

use crate::utils::exit_with_error;
use std::{collections::HashSet, process::Command};

/// Small utils tools to update local git and compare the commits.
#[derive(Parser, Debug)]
#[clap(name = "ginsp")]
struct Cli {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Version of the tool.
    Version,

    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    Update(Update),

    /// Compare two branches by commit messages.
    DiffMessage(DiffMessage),
}

#[derive(Parser, Debug)]
struct Update {
    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    #[clap(name = "branches", required = true)]
    branches: Vec<String>,
}

#[derive(Parser, Debug)]
struct DiffMessage {
    /// Two branches to compare.
    #[clap(name = "branches", required = true)]
    branches: Vec<String>,

    /// Cherry pick commits that contains the given string.
    /// Multiple strings can be separated by comma.
    /// For example: `ginsp diff-message master develop -c "fix,feat"`
    #[clap(short = 'c', long, num_args = 1)]
    pick_contains: Option<String>,
}

fn get_commits_info(branch: &str) -> Result<Vec<String>> {
    let command = format!("git log --format=%h%s --abbrev=7 {}", branch);
    let output = Command::new("sh").arg("-c").arg(command).output()?;

    if !output.status.success() {
        exit_with_error(&format!(
            "Fail to get commits info for branch '{}'. Error: {}",
            branch,
            String::from_utf8(output.stderr)?
        ));
    }

    let commit_info = String::from_utf8(output.stdout)?;
    let result = commit_info
        .trim()
        .split('\n')
        .map(|commit| {
            let (hash, message) = commit.split_at(7);
            format!("{}::{}", hash, message)
        })
        .collect();

    Ok(result)
}

fn fetch_all() -> Result<()> {
    println!("Fetching all...");
    let output = Command::new("git")
        .arg("fetch")
        .arg("--all")
        .arg("--prune")
        .arg("--tags")
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        exit_with_error(&format!("Fail to fetch all. Error: {}", err));
    }

    println!("{}", String::from_utf8(output.stdout)?);
    Ok(())
}

fn checkout_branch(branch: &str) -> Result<()> {
    let output = Command::new("git").arg("checkout").arg(branch).output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        exit_with_error(&format!(
            "Fail to checkout branch '{}'. Error: {}",
            branch, err
        ));
    }
    println!("{}", String::from_utf8(output.stdout)?);
    Ok(())
}

fn pull_branch(branch: &str) -> Result<()> {
    let output = Command::new("git").arg("pull").output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        exit_with_error(&format!("Fail to pull branch '{}'. Error: {}", branch, err));
    }
    println!("{}", String::from_utf8(output.stdout)?);
    Ok(())
}

fn command_update(branches: Vec<String>) -> Result<()> {
    fetch_all()?;
    for branch in branches.iter() {
        println!("Updating branch '{}'", branch);
        checkout_branch(branch)?;
        pull_branch(branch)?;
    }
    Ok(())
}

fn command_diff(diff_options: &DiffMessage) -> Result<()> {
    let source_branch = &diff_options.branches[0];
    let target_branch = &diff_options.branches[1];

    let source_commits = get_commits_info(source_branch.as_str())?;
    let target_commits = get_commits_info(target_branch.as_str())?;

    let mut source_map = indexmap!();

    for commit in source_commits.iter() {
        let (hash, message) = match commit.split_once("::") {
            Some((hash, message)) => (hash, message),
            None => ("", ""),
        };

        if hash.is_empty() || message.is_empty() {
            exit_with_error(&format!(
                "Fail to parse commit info '{}' of branch {}",
                commit.as_str(),
                source_branch
            ));
        }

        source_map.insert(message.trim(), hash.trim());
    }

    let mut target_map = indexmap!();
    for commit in target_commits.iter() {
        let (hash, message) = match commit.split_once("::") {
            Some((hash, message)) => (hash, message),
            None => ("", ""),
        };

        if hash.is_empty() || message.is_empty() {
            exit_with_error(&format!(
                "Fail to parse commit info '{}' of branch {}",
                commit.as_str(),
                target_branch
            ));
        }

        target_map.insert(message.trim(), hash.trim());
    }

    let unique_to_source = source_map
        .iter()
        .filter(|(message, _)| !target_map.contains_key(*message))
        .map(|(message, hash)| (hash.to_string(), message.to_string()))
        .collect::<Vec<_>>();

    let unique_to_target = target_map
        .iter()
        .filter(|(message, _)| !source_map.contains_key(*message))
        .map(|(message, hash)| (hash.to_string(), message.to_string()))
        .collect::<Vec<_>>();

    let cherry_pick_messages = diff_options
        .pick_contains
        .as_ref()
        .map(|s| s.split(',').collect::<Vec<_>>())
        .unwrap_or_default();

    // HashSet picked commits
    let mut picked_vec = HashSet::new();

    // get last commit hash before cherry pick
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=%h")
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        exit_with_error(&format!("Fail to get last commit hash. Error: {}", err));
    }

    let last_commit_hash = String::from_utf8(output.stdout)?.trim().to_string();

    if !cherry_pick_messages.is_empty() {
        println!(
            "Cherry picking {}...",
            diff_options.pick_contains.as_ref().unwrap()
        );
        // for each unique commit on source branch, if in the cherry pick list, cherry pick it to target branch
        'outer: for (hash, message) in unique_to_source.iter().rev() {
            // for each cherry pick commit message
            // if the cherry pick commit message is a substring of the source commit message
            // cherry pick the source commit to target branch
            'inner: for cherry_pick_message in cherry_pick_messages.iter() {
                if !message.contains(cherry_pick_message) {
                    continue 'inner;
                }

                println!("Cherry picking {} - {}", hash, message);
                let output = Command::new("git").arg("cherry-pick").arg(hash).output()?;
                if output.status.success() {
                    picked_vec.insert(hash);
                    continue 'outer;
                }

                println!("Fail to cherry pick commit {} - {}", hash, message);

                // cherry-pick abort
                println!("Abort cherry pick...");
                let output = Command::new("git")
                    .arg("cherry-pick")
                    .arg("--abort")
                    .output()?;

                if !output.status.success() {
                    let err = String::from_utf8(output.stderr)?;
                    exit_with_error(&format!(
                        "Fail to abort cherry pick commit '{}'. Error: {}",
                        hash, err
                    ));
                }
                // reset to last commit hash
                println!("Reset to commit hash {}...", last_commit_hash);
                let output = Command::new("git")
                    .arg("reset")
                    .arg("--hard")
                    .arg(&last_commit_hash)
                    .output()?;

                if !output.status.success() {
                    exit_with_error(&format!(
                        "Fail to reset to commit hash {}. Error: {}",
                        last_commit_hash,
                        String::from_utf8(output.stderr)?
                    ));
                }

                exit_with_error(&format!(
                    "Error: Fail to cherry pick commit {} - {}",
                    hash, message
                ));
            }
        }
    }

    print_result(source_branch, unique_to_source);
    print_result(target_branch, unique_to_target);

    Ok(())
}

/// Print result as table like this
/// ```
/// Commit messages unique on branch:
/// ------------------------
///     eec4f1c - [ABC-10370] message
///     54912eb - [ABC-10365] message
/// ```
fn print_result(branch: &str, unique_commits: Vec<(String, String)>) {
    println!("\nCommit messages unique on {}:", branch);
    println!("------------------------");
    for (hash, message) in unique_commits {
        // extract ticket number from commit message
        let ticket_number = utils::extract_ticket_number(&message, r"^\[(\w+-\d+)]");

        // get Jira ticket status with reqwest
        let status: String = match ticket_number {
            Some(ticket_number) => utils::get_jira_ticket_status(ticket_number),
            None => "Fail to fetch".to_string(),
        };

        println!("{} - {} - {}", &hash, status, message);
    }
}

fn main() -> Result<()> {
    let options = Cli::parse();

    match &options.subcommand {
        SubCommand::Version => {
            println!("ginsp version {}", env!("CARGO_PKG_VERSION"));
        }
        SubCommand::Update(update) => {
            command_update(update.branches.clone())?;
        }
        SubCommand::DiffMessage(diff) => {
            command_diff(diff)?;
        }
    }

    Ok(())
}
