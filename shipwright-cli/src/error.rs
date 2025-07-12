use thiserror::Error;

/// Error types for Shipwright CLI operations
#[derive(Error, Debug)]
pub enum ShipwrightError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Workspace error: {0}")]
    WorkspaceError(String),

    #[error("Build error: {0}")]
    BuildError(String),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Hot reload error: {0}")]
    HotReloadError(String),

    #[error("Cargo metadata error: {0}")]
    CargoMetadataError(#[from] cargo_metadata::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Notify error: {0}")]
    NotifyError(#[from] notify::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

impl From<std::io::Error> for ShipwrightError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error.to_string())
    }
}