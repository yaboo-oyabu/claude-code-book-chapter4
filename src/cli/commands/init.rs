//! `task init` command.

use crate::config::Config;
use crate::error::TaskCtlError;
use std::path::PathBuf;

pub fn run(force: bool) -> Result<String, TaskCtlError> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("taskctl");

    let config_path = config_dir.join("config.toml");

    if config_path.exists() && !force {
        return Err(TaskCtlError::ConfigError(format!(
            "Configuration file already exists: {}. Use --force to overwrite.",
            config_path.display()
        )));
    }

    std::fs::create_dir_all(&config_dir)?;
    std::fs::write(&config_path, Config::default_toml())?;

    Ok(format!(
        "Created configuration file: {}",
        config_path.display()
    ))
}
