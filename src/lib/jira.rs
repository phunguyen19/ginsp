use crate::config::profile::AuthType;

pub struct Jira {}

impl Jira {
    pub fn get_ticket_status(
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
}