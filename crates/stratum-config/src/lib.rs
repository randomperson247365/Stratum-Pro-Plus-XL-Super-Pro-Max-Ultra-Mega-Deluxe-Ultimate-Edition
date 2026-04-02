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
    #[error("failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
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

/// Serialize and write a StratumConfig to the given TOML file.
/// Creates parent directories if they don't exist.
pub fn save_config(config: &StratumConfig, path: &Path) -> Result<(), ConfigError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = toml::to_string_pretty(config)?;
    std::fs::write(path, text)?;
    Ok(())
}
