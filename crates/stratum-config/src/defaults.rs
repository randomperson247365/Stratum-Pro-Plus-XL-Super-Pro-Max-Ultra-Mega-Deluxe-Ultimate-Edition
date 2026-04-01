use std::path::PathBuf;

/// Returns the default config file path: $XDG_CONFIG_HOME/stratum/config.toml
/// or ~/.config/stratum/config.toml if XDG_CONFIG_HOME is unset.
pub fn default_config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
            PathBuf::from(home).join(".config")
        });
    base.join("stratum").join("config.toml")
}
