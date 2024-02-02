use crate::utils;
use crate::utils::{checkout_branch, exit_with_error, fetch_all, get_commits_info, pull_branch};
use clap::Parser;
use indexmap::indexmap;
use std::collections::HashSet;
use std::process::Command;
use crate::config::{Config, ProjectManagement, ProjectManagementName};

/// Small utils tools to update local git and compare the commits.
#[derive(Parser, Debug)]
#[clap(name = "ginsp")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Version of the tool.
    Version,

    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    Update(Update),

    /// Compare two branches by commit messages.
    DiffMessage(DiffMessage),
}

#[derive(Parser, Debug)]
pub struct Update {
    /// Run `git fetch --all --prune --tags`
    /// and `git pull` on each branch.
    #[clap(name = "branches", required = true)]
    pub branches: Vec<String>,
}

#[derive(Parser, Debug)]
pub struct DiffMessage {
    /// Two branches to compare.
    #[clap(name = "branches", required = true)]
    pub branches: Vec<String>,

    /// Cherry pick commits that contains the given string.
    /// Multiple strings can be separated by comma.
    /// For example: `ginsp diff-message master develop -c "fix,feat"`
    #[clap(short = 'p', long = "pick-contains", num_args = 1)]
    pub pick_contains: Option<String>,

    /// Fetching ticket status from project management tool
    /// and print it in the result table. This option requires a config file.
    /// For example: `ginsp diff-message master develop -p`
    #[clap(short = 't', long = "ticket-status", default_value = "false")]
    pub print_ticket_status: bool,

    /// Config file path.
    /// For example: `ginsp diff-message master develop -c "fix,feat" -p -f ginsp.toml`
    #[clap(short = 'c', long = "config-file")]
    pub config_file: Option<String>,
}

pub fn command_update(branches: Vec<String>) -> anyhow::Result<()> {
    fetch_all()?;
    for branch in branches.iter() {
        println!("Updating branch '{}'", branch);
        checkout_branch(branch)?;
        pull_branch(branch)?;
    }
    anyhow::Ok(())
}

pub struct DiffResult {
    pub source_branch: String,
    pub unique_to_source: Vec<CommitInfo>,
    pub target_branch: String,
    pub unique_to_target: Vec<CommitInfo>,
}

pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub status: Option<String>,
}

pub fn command_diff(diff_options: &DiffMessage) -> anyhow::Result<DiffResult> {
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

    // TODO: better pattern to load the config
    let project_management: Option<ProjectManagement> = if diff_options.print_ticket_status && diff_options.config_file.is_some() {
        let config_file_path = diff_options.config_file.as_ref().unwrap();
        let config = Config::read_toml_file(config_file_path.as_str())?;
        config.project_management
    } else {
        None
    };

    // convert unique_to_source to Vec<CommitInfo>
    let unique_to_source = unique_to_source
        .iter()
        .filter(|(hash, _)| !picked_vec.contains(hash))
        .map(|(hash, message)| CommitInfo {
            hash: hash.to_string(),
            message: message.to_string(),

            // TODO: better pattern
            status: if project_management.is_some() {
                let project_management = project_management.as_ref().unwrap();

                // extract ticket number from commit message
                let ticket_number = utils::extract_ticket_number(&message, project_management.ticket_id_regex.as_str());

                // get Jira ticket status with reqwest
                match ticket_number {
                    Some(ticket_number) => {
                        let url = match project_management.name {
                            ProjectManagementName::Jira => project_management.url.replace(":ticket_id", &ticket_number),
                        };
                        Some(utils::get_jira_ticket_status(url, &project_management.auth_type, project_management.get_auth_string()))
                    },
                    None => Some("Fail to fetch".to_string()),
                }
            } else {
                None
            },
        })
        .collect::<Vec<_>>();

    // convert unique_to_target to Vec<CommitInfo>
    let unique_to_target = unique_to_target
        .iter()
        .map(|(hash, message)| CommitInfo {
            hash: hash.to_string(),
            message: message.to_string(),

            // TODO: better pattern
            status: if project_management.is_some() {
                let project_management = project_management.as_ref().unwrap();

                // extract ticket number from commit message
                let ticket_number = utils::extract_ticket_number(&message, project_management.ticket_id_regex.as_str());

                // get Jira ticket status with reqwest
                match ticket_number {
                    Some(ticket_number) => {
                        let url = match project_management.name {
                            ProjectManagementName::Jira => project_management.url.replace(":ticket_id", &ticket_number),
                        };
                        Some(utils::get_jira_ticket_status(url, &project_management.auth_type, project_management.get_auth_string()))
                    },
                    None => Some("Fail to fetch".to_string()),
                }
            } else {
                None
            },
        })
        .collect::<Vec<_>>();

    Ok(DiffResult {
        source_branch: source_branch.to_string(),
        unique_to_source,
        target_branch: target_branch.to_string(),
        unique_to_target,
    })
}