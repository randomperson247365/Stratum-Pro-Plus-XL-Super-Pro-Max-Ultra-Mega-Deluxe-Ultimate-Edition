pub mod defaults;
pub mod schema;
pub mod watcher;

pub use defaults::default_config_path;
pub use schema::*;
pub use watcher::watch_config;

use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

/// Load a StratumConfig from the given TOML file.
/// If the file doesn't exist, returns the default config.
pub fn load_config(path: &Path) -> Result<StratumConfig, ConfigError> {
    if !path.exists() {
        return Ok(StratumConfig::default());
    }
    let text = std::fs::read_to_string(path)?;
    let config: StratumConfig = toml::from_str(&text)?;
    Ok(config)
}
