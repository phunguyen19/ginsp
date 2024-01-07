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

// TODO: handle error, should return Result
pub fn get_jira_ticket_status(url: String) -> String {
    let client = reqwest::blocking::Client::new();
    let res = client
        .get(url)
        .basic_auth("email", Some("key"))
        .header("Accept", "application/json")
        // TODO: handle error
        .send()
        .unwrap();

    let status = res.status();

    return match status {
        reqwest::StatusCode::OK => {
            // parse ticket status fields.status.name from res body
            // TODO: handle error
            let body = res.text().unwrap();
            // TODO: handle error
            let json: serde_json::Value = serde_json::from_str(&body).unwrap();
            // TODO: handle error
            let fields = json["fields"].as_object().unwrap();
            // TODO: handle error
            let s = fields["status"]["name"].as_str().unwrap();

            // TODO: handle error, should return Result
            s.to_string()
        }
        _ => {
            // TODO: handle error, should return Result
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
