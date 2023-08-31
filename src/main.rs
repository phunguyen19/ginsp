use anyhow::{Ok, Result};
use clap::Parser;
use indexmap::indexmap;

use std::process::Command;

/// Small utils tools to update local git and compare the commits.
#[derive(Parser, Debug)]
#[clap(name = "ginsp")]
struct Cli {
    #[clap(subcommand)]
    subcommand: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    Update(Update),

    /// Compare two branches by git commit messages.
    /// Show the commits unique to each branch.
    Diff(Diff),
}

#[derive(Parser, Debug)]
struct Update {
    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    #[clap(name = "branches", required = true)]
    branches: Vec<String>,
}

#[derive(Parser, Debug)]
struct Diff {
    /// Two branches to compare.
    #[clap(name = "branches", required = true)]
    branches: Vec<String>,

    /// Cherry pick the commits from the source branch to the target branch
    #[clap(long, default_value = "false")]
    cherry_pick: bool,
}

fn exit_with_error(error: &str) {
    eprintln!("{}", error);
    std::process::exit(1);
}

fn get_commits_info(branch: &str) -> Result<Vec<String>> {
    let command = format!("git log --format=%h%s {}", branch);
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
            format!("{}{}", hash, message)
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
        checkout_branch(&branch)?;
        pull_branch(&branch)?;
    }
    Ok(())
}

fn command_diff((source_branch, target_branch): (String, String)) -> Result<()> {
    let source_commits = get_commits_info(source_branch.as_str())?;
    let target_commits = get_commits_info(target_branch.as_str())?;

    let mut source_map = indexmap!();
    for commit in source_commits.iter() {
        let (hash, message) = commit.split_at(9);
        source_map.insert(message.trim(), hash.trim());
    }

    let mut target_map = indexmap!();
    for commit in target_commits.iter() {
        let (hash, message) = commit.split_at(9);
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

    println!("Commit messages unique on {}:", source_branch);
    println!("--------");
    for (hash, message) in unique_to_source {
        println!("{:>7} - {}", hash, message);
    }

    println!("\nCommit messages unique on {}:", target_branch);
    println!("--------");
    for (hash, message) in unique_to_target {
        println!("{:>7} - {}", hash, message);
    }

    Ok(())
}

fn main() -> Result<()> {
    let _ = Command::new("git").arg("fetch").spawn();

    let options = Cli::parse();

    match &options.subcommand {
        SubCommand::Update(update) => {
            command_update(update.branches.clone())?;
        }
        SubCommand::Diff(diff) => {
            command_diff((diff.branches[0].clone(), diff.branches[1].clone()))?;
        }
    }

    Ok(())
}
