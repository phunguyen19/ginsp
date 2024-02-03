use crate::error::{CredentialsErrorKind, GinspError};
use toml::Table;

#[derive(Debug)]
pub struct Credentials {
    credentials: Table,
}

impl Credentials {
    pub fn read_credential_file() -> anyhow::Result<Credentials, GinspError> {
        let path_buf = match home::home_dir() {
            Some(p) => p,
            None => return Err(GinspError::System("Cannot find home directory".to_string())),
        };

        let homedir = match path_buf.to_str() {
            Some(p) => p,
            None => return Err(GinspError::System("Cannot read home directory".to_string())),
        };

        let content_string =
            std::fs::read_to_string(format!("{}/.ginsp/credentials.toml", homedir).as_str())
                .map_err(|err| GinspError::Credentials(CredentialsErrorKind::IO(err)))?;

        let content_toml = toml::from_str::<Table>(content_string.as_str())
            .map_err(|err| GinspError::Credentials(CredentialsErrorKind::Toml(err)))?;

        Ok(Credentials {
            credentials: content_toml,
        })
    }

    pub fn find_credential_value(&self, key: &str) -> Option<String> {
        self.credentials.get(key).map(|v| v.to_string())
    }
}
