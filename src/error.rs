use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GinspError {
    #[error("Config error: {0}")]
    Config(ConfigErrorKind),
    #[error("Git error: {0}")]
    Git(String),
    #[error("System error: {0}")]
    System(String),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigErrorKind {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Syntax error: {0}")]
    Syntax(#[from] toml::de::Error),
}
