pub mod build;
pub mod dev;
pub mod new;
pub mod serve;

use crate::{config::Config, error::ShipwrightError};

/// Common functionality shared across commands
pub struct CommandContext {
    pub config: Config,
    pub workspace: crate::workspace::Workspace,
}

impl CommandContext {
    pub fn new(config: Config) -> Result<Self, ShipwrightError> {
        let workspace = crate::workspace::Workspace::detect(None)?;
        Ok(Self { config, workspace })
    }
}