use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GinspError {
    #[error("Cli error: {0}")]
    Cli(String),
    #[error("Config error: {0}")]
    Config(ConfigErrorKind),
    #[error("Git error: {0}")]
    Git(String),
    #[error("Jira error: {0}")]
    Http(String),
    #[error("System error: {0}")]
    System(String),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigErrorKind {
    #[error("Unable to read config file: IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Unable to read config file: Syntax error: {0}")]
    Syntax(#[from] toml::de::Error),
    #[error("Invalid credential key")]
    InvalidCredentialKey,
}
