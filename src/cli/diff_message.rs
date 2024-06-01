use crate::cli::{Cli, CommandHandler};
use crate::config::{Config, ProjectManagement, ProjectManagementProvider};
use crate::error::{ConfigErrorKind, GinspError};
use crate::{cli, git, jira};
use indexmap::indexmap;
use regex::Regex;
use std::process::Command;

pub struct DiffMessage {}

pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub status: Option<String>,
    pub is_picked: bool,
}

pub struct RawCommitInfo(String, String);
impl From<&RawCommitInfo> for CommitInfo {
    fn from(raw_commit: &RawCommitInfo) -> Self {
        CommitInfo {
            hash: raw_commit.0.to_string(),
            message: raw_commit.1.to_string(),
            status: None,
            is_picked: false,
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
        git::Git::validate_git_installed()?;
        git::Git::validate_git_repo()?;

        // Get and validate command line options
        let options = match cli.subcommand {
            cli::SubCommand::DiffMessage(ref diff_cmd) => diff_cmd,
            _ => return Err(GinspError::Cli("Invalid subcommand".to_string()).into()),
        };

        // Get and validate branches
        let [source_branch, target_branch] = {
            if options.branches.len() != 2 {
                let err_msg = "Provide 2 branches to compare".to_string();
                return Err(GinspError::Cli(err_msg).into());
            }
            [&options.branches[0], &options.branches[1]]
        };

        let is_cherry_pick = options.pick_contains.is_some();

        // validate current branch is the target branch (branches[1])
        if is_cherry_pick {
            let current_branch = git::Git::get_current_branch()?;
            if current_branch != options.branches[1] {
                return Err(GinspError::Cli(format!(
                    "Checkout to the target branch '{}' to use cherry-pick option.",
                    target_branch
                ))
                .into());
            }
        }

        let cherry_pick_messages = match options
            .pick_contains
            .as_ref()
            .map(|s| s.split(',').collect::<Vec<_>>())
        {
            Some(pick_contains) => pick_contains,
            None => vec![],
        };

        let source_map = load_commits_as_map(source_branch)?;
        let target_map = load_commits_as_map(target_branch)?;

        let unique_to_source = unique_by_message(&source_map, &target_map);
        let unique_to_target = unique_by_message(&target_map, &source_map);

        let mut unique_to_source = unique_to_source
            .iter()
            .map(CommitInfo::from)
            .collect::<Vec<_>>();

        // convert unique_to_target to Vec<CommitInfo>
        let mut unique_to_target = unique_to_target
            .iter()
            .map(CommitInfo::from)
            .collect::<Vec<_>>();

        if options.is_fetch_ticket_status {
            let profile = Config::read_config_file_from_home_dir()?;
            unique_to_source = map_ticket_status(unique_to_source, &profile, options.verbose);
            unique_to_target = map_ticket_status(unique_to_target, &profile, options.verbose);
        }

        if is_cherry_pick && !unique_to_source.is_empty() {
            let last_commit_hash = get_last_commit_hash()?;

            'commit_loop: for commit in unique_to_source.iter_mut().rev() {
                let CommitInfo { hash, message, .. } = commit;

                'contain_msg_loop: for cherry_pick_message in cherry_pick_messages.iter() {
                    if !message.contains(cherry_pick_message) {
                        continue 'contain_msg_loop;
                    }

                    if options.verbose {
                        println!("Doing cherry-pick {} {}", hash, message);
                    }

                    match git::Git::cherry_pick(hash) {
                        Ok(_) => {
                            commit.is_picked = true;
                            continue 'commit_loop;
                        }
                        Err(_) => {
                            eprintln!("Fail to cherry-pick commit. Resetting current branch to the last commit hash {}...", last_commit_hash);

                            eprintln!("Aborting cherry-pick...");
                            git::Git::cherry_pick_abort()
                                .map(git::Git::print_stderr)
                                .map_err(|err| {
                                    GinspError::Git(format!(
                                        "Fail to abort cherry-pick. Error: {}",
                                        err
                                    ))
                                })?;

                            eprintln!(
                                "Resetting to commit hash {} (before doing cherry-pick)...",
                                last_commit_hash
                            );
                            git::Git::reset_hard(&last_commit_hash)
                                .map(git::Git::print_stderr)
                                .map_err(|err| {
                                    GinspError::Git(format!(
                                        "Fail to reset to commit hash {}. Error: {}",
                                        last_commit_hash, err
                                    ))
                                })?;

                            return Err(GinspError::Git(format!(
                                "Fail to cherry-pick commit {} {}",
                                hash, message
                            ))
                            .into());
                        }
                    }
                }
            }
        }

        print_result(source_branch, unique_to_source);
        print_result(target_branch, unique_to_target);
        println!();

        Ok(())
    }
}

fn map_ticket_status(
    commits: Vec<CommitInfo>,
    profile: &Config,
    is_verbose: bool,
) -> Vec<CommitInfo> {
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
                        if is_verbose {
                            println!("Fetching ticket status for {}", ticket_number);
                        }
                        get_ticket_status(&ticket_number, project_management).ok()
                    }
                    None => None,
                },
                is_picked: commit.is_picked,
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

    let status = jira::Jira::get_ticket_status(url, username.to_string(), password.to_string())
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
    let commits_len = commits.len();
    let max_len_index = commits_len.to_string().len();
    let max_status_len = commits
        .iter()
        .map(|commit| commit.status.as_ref().map_or(0, |s| s.len()))
        .max()
        .unwrap_or(0);
    for (index, item) in commits.into_iter().enumerate() {
        let CommitInfo {
            hash,
            message,
            status,
            is_picked,
        } = item;

        let mut string_vec = vec![];

        if is_picked {
            string_vec.push("*".to_string());
        } else {
            string_vec.push(" ".to_string());
        }

        string_vec.push(format!(
            "{:width$}",
            commits_len - index,
            width = max_len_index
        ));

        if let Some(status) = status {
            string_vec.push(format!("{:width$}", status, width = max_status_len));
        } else {
            string_vec.push(format!("{:width$}", "", width = max_status_len));
        }

        string_vec.push(hash);
        string_vec.push(message);

        println!("{}", string_vec.join(" "));
    }
}
