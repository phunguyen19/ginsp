use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum GinspError {
    #[error("Config error: {0}")]
    Config(ConfigErrorKind),
    #[error("Credentials error: {0}")]
    Credentials(CredentialsErrorKind),
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
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("ENV error: {0}")]
    Env(#[from] std::env::VarError),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CredentialsErrorKind {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}
