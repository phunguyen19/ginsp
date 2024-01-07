use crate::error::{ConfigErrorKind, ErrorKind};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    project_management: Option<ProjectManagement>,
}

#[derive(Debug, Deserialize)]
struct ProjectManagement {
    name: ProjectManagementName,
    url: String,
    credential_env_var_name: String,
    ticket_id_regex: String,
}

#[derive(Debug, Deserialize)]
enum ProjectManagementName {
    Jira,
}

impl Config {
    pub fn read_toml_file(path: &str) -> anyhow::Result<Config, ErrorKind> {
        let toml = std::fs::read_to_string(path)
            .map_err(|err| ErrorKind::ConfigError(ConfigErrorKind::IOError(err)))?;
        let config: Config = toml::from_str(toml.as_str())
            .map_err(|err| ErrorKind::ConfigError(ConfigErrorKind::TOMLError(err)))?;
        Ok(config)
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
