use crate::error::{ConfigErrorKind, GinspError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project_management: Option<ProjectManagement>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectManagement {
    pub provider: ProjectManagementProvider,
    pub url: String,
    pub credential_key: String,
    pub ticket_id_regex: String,
}

#[derive(Debug, Deserialize)]
pub enum AuthType {
    Basic,
}

#[derive(Debug, Deserialize)]
pub enum ProjectManagementProvider {
    Jira,
}

impl Config {
    pub fn read_toml_file(path: &str) -> anyhow::Result<Config, GinspError> {
        let toml = std::fs::read_to_string(path)
            .map_err(|err| GinspError::Config(ConfigErrorKind::IO(err)))?;

        let config: Config = toml::from_str(toml.as_str())
            .map_err(|err| GinspError::Config(ConfigErrorKind::Syntax(err)))?;

        Ok(config)
    }

    pub fn read_config_file_from_home_dir() -> anyhow::Result<Config, GinspError> {
        let path_buf = home::home_dir().ok_or(GinspError::Config(ConfigErrorKind::IO(
            std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found"),
        )))?;

        let homedir = path_buf
            .to_str()
            .ok_or(GinspError::Config(ConfigErrorKind::IO(
                std::io::Error::new(std::io::ErrorKind::InvalidData, "Home directory not found"),
            )))?;

        let file_path = format!("{}/.ginsp/config.toml", homedir);

        Self::read_toml_file(file_path.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_toml_file_not_exist() {
        let config = Config::read_toml_file("tests/fixtures/test-config.not-exist.toml");
        assert!(config.is_err());
        assert!(config
            .unwrap_err()
            .to_string()
            .starts_with("Config error: IO error: No such file or directory"));
    }

    #[test]
    fn test_wrong_toml_format() {
        let config = Config::read_toml_file("tests/fixtures/test-config.wrong-format.toml");
        assert!(config.is_err());
        assert!(config
            .unwrap_err()
            .to_string()
            .starts_with("Config error: Syntax error:"));
    }

    #[test]
    fn test_read_toml_file() {
        let config = Config::read_toml_file("tests/fixtures/test-config.toml");
        assert!(config.is_ok());
    }
}
