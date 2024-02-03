use crate::config::AuthType;
use crate::error::GinspError;
use regex::Regex;
use std::process::Command;

pub fn exit_with_error(error: &str) {
    eprintln!("{}", error);
    std::process::exit(1);
}

pub fn extract_ticket_number(message: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).expect("Invalid ticket regex pattern");
    let caps = re.captures(message);
    caps.map(|caps| caps[1].to_string())
}

// TODO: handle error
pub fn get_jira_ticket_status(
    url: String,
    auth_type: &Option<AuthType>,
    auth_string: Option<String>,
) -> String {
    let client = reqwest::blocking::Client::new();
    let mut builder = client
        .get(url.as_str())
        .header("Accept", "application/json");

    builder = match auth_type {
        Some(AuthType::Basic) => {
            let auth_string = auth_string.unwrap_or_default();
            let (username, password) =
                auth_string.split_at(auth_string.find(':').unwrap_or_default());
            builder.basic_auth(username, Some(&password[1..]))
        }
        Some(AuthType::Bearer) => builder.bearer_auth(auth_string.unwrap_or_default()),
        None => builder,
    };

    let res = builder.send().unwrap();

    let status = res.status();

    return match status {
        reqwest::StatusCode::OK => {
            let body = res.text().unwrap(); // TODO: handle error
            let json: serde_json::Value = serde_json::from_str(&body).unwrap(); // TODO: handle error
            let fields = json["fields"].as_object().unwrap(); // TODO: handle error
            let s = fields["status"]["name"].as_str().unwrap(); // TODO: handle error
            s.to_string()
        }
        _ => {
            // TODO: handle error
            format!("Error: {}", status)
        }
    };
}

pub fn get_commits_info(branch: &str) -> anyhow::Result<Vec<String>> {
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

    anyhow::Ok(result)
}

pub fn fetch_all() -> anyhow::Result<()> {
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
    anyhow::Ok(())
}

pub fn checkout_branch(branch: &str) -> anyhow::Result<()> {
    let output = Command::new("git").arg("checkout").arg(branch).output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        exit_with_error(&format!(
            "Fail to checkout branch '{}'. Error: {}",
            branch, err
        ));
    }
    println!("{}", String::from_utf8(output.stdout)?);
    anyhow::Ok(())
}

pub fn pull_branch(branch: &str) -> anyhow::Result<()> {
    let output = Command::new("git").arg("pull").output()?;
    if !output.status.success() {
        let err = String::from_utf8(output.stderr)?;
        exit_with_error(&format!("Fail to pull branch '{}'. Error: {}", branch, err));
    }
    println!("{}", String::from_utf8(output.stdout)?);
    anyhow::Ok(())
}

/// Validate if git is installed
pub fn validate_git() -> anyhow::Result<(), GinspError> {
    let output = Command::new("git")
        .arg("--version")
        .output()
        .map_err(|err| GinspError::System(err.to_string()))?;

    if !output.status.success() {
        let err =
            String::from_utf8(output.stderr).map_err(|err| GinspError::System(err.to_string()))?;
        Err(GinspError::Git(err))
    } else {
        Ok(())
    }
}

/// Validate if the current repo has git
pub fn validate_git_repo() -> anyhow::Result<(), GinspError> {
    let output = Command::new("git")
        .arg("status")
        .output()
        .map_err(|err| GinspError::System(err.to_string()))?;

    if !output.status.success() {
        let err =
            String::from_utf8(output.stderr).map_err(|err| GinspError::System(err.to_string()))?;
        Err(GinspError::Git(err))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ticket_number() {
        let cases = vec![
            vec![
                "[JIRA-123] This is a test message",
                r"^\[(\w+-\d+)]",
                "JIRA-123",
            ],
            vec![
                "(JIRA-123) This is a test message",
                r"^\((\w+-\d+)\)",
                "JIRA-123",
            ],
            vec!["JIRA-123 This is a test message", r"^(\w+-\d+)", "JIRA-123"],
        ];

        for case in cases {
            let message = case[0];
            let pattern = case[1];
            let expected = case[2];
            let ticket_number = extract_ticket_number(message, pattern);
            assert_eq!(ticket_number, Some(expected.to_string()));
        }
    }
}
