use crate::error::{ConfigErrorKind, GinspError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project_management: Option<ProjectManagement>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectManagement {
    pub name: ProjectManagementName,
    pub url: String,
    pub credential_env_var_name: String,
    pub ticket_id_regex: String,
    pub auth_type: Option<AuthType>,
    auth_string: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum AuthType {
    Basic,
    Bearer,
}

#[derive(Debug, Deserialize)]
pub enum ProjectManagementName {
    Jira,
}

impl Config {
    pub fn read_toml_file(path: &str) -> anyhow::Result<Config, GinspError> {
        let toml = std::fs::read_to_string(path)
            .map_err(|err| GinspError::ConfigError(ConfigErrorKind::IOError(err)))?;

        let mut config: Config = toml::from_str(toml.as_str())
            .map_err(|err| GinspError::ConfigError(ConfigErrorKind::TOMLError(err)))?;

        // read auth string from env var
        match &mut config.project_management {
            Some(project_management) => {
                let env_var_name = project_management.credential_env_var_name.as_str();
                match std::env::var(env_var_name) {
                    Ok(auth_string) => {
                        project_management.auth_string = Some(auth_string);
                    }
                    Err(_) => {} // TODO: error handling
                };
            }
            None => {}
        };

        // return config
        Ok(config)
    }
}

impl ProjectManagement {
    pub fn get_auth_string(&self) -> Option<String> {
        self.auth_string.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_toml_file_not_exist() {
        let config = Config::read_toml_file("tests/fixtures/test-config.not-exist.toml");
        assert_eq!(config.is_err(), true);
        assert!(config
            .unwrap_err()
            .to_string()
            .starts_with("Config error: IO error: No such file or directory"));
    }

    #[test]
    fn test_wrong_toml_format() {
        let config = Config::read_toml_file("tests/fixtures/test-config.wrong-format.toml");
        assert_eq!(config.is_err(), true);
        assert!(config
            .unwrap_err()
            .to_string()
            .starts_with("Config error: TOML error: "));
    }

    #[test]
    fn test_read_toml_file() {
        let config = Config::read_toml_file("tests/fixtures/test-config.toml");
        assert_eq!(config.is_ok(), true);
        assert_eq!(config.unwrap().project_management.is_some(), true);
    }
}
