use regex::Regex;

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
pub fn get_jira_ticket_status(ticket_number: String) -> String {
    let url = format!(
        "https://inspectorio.atlassian.net/rest/api/3/issue/{}",
        ticket_number
    );
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
