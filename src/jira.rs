use crate::error::GinspError;

pub struct Jira {}

impl Jira {
    pub fn get_ticket_status(
        url: String,
        username: String,
        password: String,
    ) -> Result<String, GinspError> {
        let client = reqwest::blocking::Client::new();

        let res = client
            .get(url.as_str())
            .header("Accept", "application/json")
            .basic_auth(username, Some(password))
            .send()
            .map_err(|err| GinspError::Http(err.to_string()))?;

        let status = res.status();

        if let reqwest::StatusCode::OK = status {
            let body = res
                .text()
                .map_err(|err| GinspError::System(err.to_string()))?;
            let json: serde_json::Value =
                serde_json::from_str(&body).map_err(|err| GinspError::System(err.to_string()))?;
            let fields = json["fields"]
                .as_object()
                .ok_or(GinspError::Http("Error: fields not found".to_string()))?;
            let s = fields["status"]["name"]
                .as_str()
                .ok_or(GinspError::Http("Error: status not found".to_string()))?;
            Ok(s.to_string())
        } else {
            Err(GinspError::Http(format!("Error: {}", status)))
        }
    }
}
