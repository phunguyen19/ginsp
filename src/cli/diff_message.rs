use crate::cli::{Cli, CommandHandler};
use crate::config::{Config, ProjectManagement, ProjectManagementProvider};
use crate::error::{ConfigErrorKind, GinspError};
use crate::utils::exit_with_error;
use crate::{cli, service};
use indexmap::indexmap;
use regex::Regex;
use std::process::Command;

pub struct DiffMessage {}

pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub status: Option<String>,
}

pub struct RawCommitInfo(String, String);
impl From<&RawCommitInfo> for CommitInfo {
    fn from(raw_commit: &RawCommitInfo) -> Self {
        CommitInfo {
            hash: raw_commit.0.to_string(),
            message: raw_commit.1.to_string(),
            status: None,
        }
    }
}

impl DiffMessage {
    pub fn new() -> Self {
        Self {}
    }
}

impl CommandHandler for DiffMessage {
    fn execute(&self, cli: &Cli) -> anyhow::Result<()> {
        // validate git is installed and the current directory is a git repository
        service::git::Git::validate_git_installed()?;
        service::git::Git::validate_git_repo()?;

        // Get and validate command line options
        let diff_options = match cli.subcommand {
            cli::SubCommand::DiffMessage(ref diff_cmd) => diff_cmd,
            _ => return Err(GinspError::Cli("Invalid subcommand".to_string()).into()),
        };

        // Get and validate branches
        let [source_branch, target_branch] = {
            if diff_options.branches.len() != 2 {
                let err_msg = "Provide 2 branches to compare".to_string();
                return Err(GinspError::Cli(err_msg).into());
            }
            [&diff_options.branches[0], &diff_options.branches[1]]
        };

        let is_cherry_pick = diff_options.pick_contains.is_some();

        // validate current branch is the target branch (branches[1])
        if is_cherry_pick {
            let current_branch = service::git::Git::get_current_branch()?;
            if current_branch != diff_options.branches[1] {
                return Err(GinspError::Cli(format!(
                    "Checkout to the target branch '{}' to use cherry-pick option.",
                    target_branch
                ))
                .into());
            }
        }

        let source_map = load_commits_as_map(source_branch)?;
        let target_map = load_commits_as_map(target_branch)?;

        let unique_to_source = unique_by_message(&source_map, &target_map);
        let unique_to_target = unique_by_message(&target_map, &source_map);

        if is_cherry_pick {
            println!(
                "Cherry picking {}...",
                diff_options.pick_contains.as_ref().unwrap()
            );

            // Parse cherry-pick substrings
            let cherry_pick_messages = diff_options
                .pick_contains
                .as_ref()
                .map(|s| s.split(',').collect::<Vec<_>>())
                .unwrap_or_default();

            // This is used to reset to last commit hash if cherry-pick fails
            let last_commit_hash = get_last_commit_hash()?;

            // for each unique commit on source branch, if in the cherry-pick list, cherry-pick it to target branch
            'outer: for RawCommitInfo(hash, message) in unique_to_source.iter().rev() {
                // for each cherry-pick commit message
                // if the cherry-pick commit message is a substring of the source commit message
                // cherry-pick the source commit to target branch
                'inner: for cherry_pick_message in cherry_pick_messages.iter() {
                    if !message.contains(cherry_pick_message) {
                        continue 'inner;
                    }

                    println!("Cherry picking {} - {}", hash, message);
                    let output = Command::new("git")
                        .arg("cherry-pick")
                        .arg(hash)
                        .output()
                        .map_err(|err| GinspError::Git(err.to_string()))?;
                    if output.status.success() {
                        continue 'outer;
                    }

                    println!("Fail to cherry-pick commit {} - {}", hash, message);

                    // cherry-pick abort
                    println!("Abort cherry-pick...");
                    let output = Command::new("git")
                        .arg("cherry-pick")
                        .arg("--abort")
                        .output()
                        .map_err(|err| GinspError::Git(err.to_string()))?;

                    if !output.status.success() {
                        let err = String::from_utf8(output.stderr)
                            .map_err(|err| GinspError::System(err.to_string()))?;
                        exit_with_error(&format!(
                            "Fail to abort cherry-pick commit '{}'. Error: {}",
                            hash, err
                        ));
                    }
                    // reset to last commit hash
                    println!("Reset to commit hash {}...", last_commit_hash);
                    let output = Command::new("git")
                        .arg("reset")
                        .arg("--hard")
                        .arg(&last_commit_hash)
                        .output()
                        .map_err(|err| GinspError::Git(err.to_string()))?;

                    if !output.status.success() {
                        exit_with_error(&format!(
                            "Fail to reset to commit hash {}. Error: {}",
                            last_commit_hash,
                            String::from_utf8(output.stderr)
                                .map_err(|err| GinspError::System(err.to_string()))?
                        ));
                    }

                    exit_with_error(&format!(
                        "Error: Fail to cherry-pick commit {} - {}",
                        hash, message
                    ));
                }
            }
        }

        let mut unique_to_source = unique_to_source
            .iter()
            .map(CommitInfo::from)
            .collect::<Vec<_>>();

        // convert unique_to_target to Vec<CommitInfo>
        let mut unique_to_target = unique_to_target
            .iter()
            .map(CommitInfo::from)
            .collect::<Vec<_>>();

        if diff_options.is_fetch_ticket_status {
            let profile = Config::read_config_file_from_home_dir()?;
            unique_to_source = map_ticket_status(unique_to_source, &profile);
            unique_to_target = map_ticket_status(unique_to_target, &profile);
        }

        print_result(source_branch, unique_to_source);
        print_result(target_branch, unique_to_target);
        println!();

        Ok(())
    }
}

fn map_ticket_status(commits: Vec<CommitInfo>, profile: &Config) -> Vec<CommitInfo> {
    commits
        .iter()
        .map(|commit| {
            let project_management = profile.project_management.as_ref().unwrap();
            let ticket_number =
                extract_ticket_number(&commit.message, project_management.ticket_id_regex.as_str());
            CommitInfo {
                hash: commit.hash.to_string(),
                message: commit.message.to_string(),
                status: match ticket_number {
                    Some(ticket_number) => {
                        get_ticket_status(&ticket_number, project_management).ok()
                    }
                    None => None,
                },
            }
        })
        .collect::<Vec<_>>()
}

fn load_commits_as_map(branch: &str) -> Result<indexmap::IndexMap<String, String>, GinspError> {
    let commits = get_commits_info(branch).map_err(|err| {
        GinspError::Git(format!(
            "Fail to get commits info for branch '{}'. Error: {}",
            branch, err
        ))
    })?;

    let mut map = indexmap!();

    for commit in commits.iter() {
        let (hash, message) = match commit.split_once("::") {
            Some((hash, message)) => (hash, message),
            None => ("", ""),
        };

        if hash.is_empty() || message.is_empty() {
            return Err(GinspError::Git(format!(
                "Fail to parse commit info '{}' of branch {}",
                commit.as_str(),
                branch
            )));
        }

        map.insert(message.trim().to_string(), hash.trim().to_string());
    }

    Ok(map)
}

fn get_commits_info(branch: &str) -> Result<Vec<String>, GinspError> {
    let command = format!("git log --format=%h%s --abbrev=7 {}", branch);
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|err| GinspError::Git(err.to_string()))?;

    if !output.status.success() {
        let err =
            String::from_utf8(output.stderr).map_err(|err| GinspError::System(err.to_string()))?;
        return Err(GinspError::Git(err));
    }

    let commit_info =
        String::from_utf8(output.stdout).map_err(|err| GinspError::System(err.to_string()))?;
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

fn unique_by_message(
    from: &indexmap::IndexMap<String, String>,
    to: &indexmap::IndexMap<String, String>,
) -> Vec<RawCommitInfo> {
    from.iter()
        .filter(|(message, _)| !to.contains_key(*message))
        .map(|(message, hash)| RawCommitInfo(hash.to_string(), message.to_string()))
        .collect::<Vec<_>>()
}

fn get_last_commit_hash() -> anyhow::Result<String, GinspError> {
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=%h")
        .output()
        .map_err(|err| GinspError::Git(err.to_string()))?;

    if !output.status.success() {
        let err =
            String::from_utf8(output.stderr).map_err(|err| GinspError::System(err.to_string()))?;
        return Err(GinspError::Git(err));
    }

    Ok(String::from_utf8(output.stdout)
        .map_err(|err| GinspError::System(err.to_string()))?
        .trim()
        .to_string())
}

fn extract_ticket_number(message: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).expect("Invalid ticket regex pattern");
    let caps = re.captures(message);
    caps.map(|caps| caps[1].to_string())
}

fn get_ticket_status(
    ticket_number: &str,
    project_management: &ProjectManagement,
) -> Result<String, GinspError> {
    let split_at = project_management
        .credential_key
        .find(':')
        .ok_or(GinspError::Config(ConfigErrorKind::InvalidCredentialKey))?;

    let url = match project_management.provider {
        ProjectManagementProvider::Jira => {
            project_management.url.replace(":ticket_id", ticket_number)
        }
    };

    let (username, password) = project_management.credential_key.split_at(split_at);

    let status =
        service::jira::Jira::get_ticket_status(url, username.to_string(), password.to_string())
            .map_err(|err| GinspError::Http(err.to_string()))?;

    Ok(status)
}

/// Print result as table like this
/// ```
/// Commit messages unique on branch:
/// ------------------------
///     eec4f1c - [ABC-10370] message
///     54912eb - [ABC-10365] message
/// ```
fn print_result(branch: &str, commits: Vec<CommitInfo>) {
    println!("\nCommit messages unique on {}:", branch);
    println!("------------------------");
    let len = commits.len();
    for (index, item) in commits.into_iter().enumerate() {
        let CommitInfo {
            hash,
            message,
            status,
        } = item;

        if status.is_none() {
            println!("   {:>3} - {} - {}", len - index, hash, message);
        } else {
            println!(
                "   {:3} - {} - {} - {}",
                len - index,
                hash,
                status.unwrap_or_default(),
                message
            );
        }
    }
}
