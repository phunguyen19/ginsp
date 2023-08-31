use clap::{command, Parser, Subcommand};
use indexmap::indexmap;
use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

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

fn get_commits_info(branch: &str) -> Vec<String> {
    let command = format!("git log --format=%h%s {}", branch);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("Failed to run git command");

    let commit_info = String::from_utf8(output.stdout).unwrap();
    return commit_info
        .trim()
        .split('\n')
        .map(|commit| {
            let (hash, message) = commit.split_at(7);
            format!("{}{}", hash, message)
        })
        .collect();
}

fn update_command(branches: Vec<String>) {
    println!("Updating all branches");
    let fetching_output = Command::new("git")
        .arg("fetch")
        .arg("--all")
        .arg("--prune")
        .arg("--tags")
        .output()
        .map(|v| String::from_utf8(v.stdout))
        .expect("Failed to run git command")
        .expect("Failed to run git command");

    println!("{}", fetching_output);

    for branch in branches.iter() {
        println!("Updating branch: {}", branch);

        let mut o = Command::new("git")
            .arg("checkout")
            .arg(branch)
            .output()
            .map(|v| String::from_utf8(v.stdout))
            .expect("Failed to run git command")
            .expect("Failed to run git command");

        println!("{}", o);

        o = Command::new("git")
            .arg("pull")
            .arg(branch)
            .output()
            .map(|v| String::from_utf8(v.stdout))
            .expect("Failed to run git command")
            .expect("Failed to run git command");

        println!("{}", o);
    }
}

fn diff_command((source_branch, target_branch): (String, String)) {
    let source_commits = get_commits_info(source_branch.as_str());
    let target_commits = get_commits_info(target_branch.as_str());

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
}

fn main() {
    let _ = Command::new("git").arg("fetch").spawn();

    let options = Cli::parse();

    // if sub command is update
    if let SubCommand::Update(update) = &options.subcommand {
        update_command(update.branches.clone());
    }

    // if sub command is diff
    if let SubCommand::Diff(diff) = &options.subcommand {
        diff_command((diff.branches[0].clone(), diff.branches[1].clone()));
    }
}
