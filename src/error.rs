use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GinspError {
    #[error("Config error: {0}")]
    ConfigError(ConfigErrorKind),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("System error: {0}")]
    SystemError(String),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigErrorKind {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    TOMLError(#[from] toml::de::Error),
    #[error("ENV error: {0}")]
    ENVError(#[from] std::env::VarError),
}
