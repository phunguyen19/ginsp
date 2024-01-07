use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ErrorKind {
    #[error("Config error: {0}")]
    ConfigError(ConfigErrorKind),
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigErrorKind {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    TOMLError(#[from] toml::de::Error),
}