//! Configuration file loading and default values.

use crate::error::TaskCtlError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Top-level configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub priority: PriorityConfig,
    pub estimate: EstimateConfig,
    pub display: DisplayConfig,
    pub data: DataConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PriorityConfig {
    pub weights: Weights,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Weights {
    pub urgency: f64,
    pub blocking: f64,
    pub staleness: f64,
    pub quick_win: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct EstimateConfig {
    pub point_to_hours: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub color: bool,
    pub date_format: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DataConfig {
    pub directory: String,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            urgency: 1.0,
            blocking: 0.8,
            staleness: 0.5,
            quick_win: 0.3,
        }
    }
}

impl Default for EstimateConfig {
    fn default() -> Self {
        Self {
            point_to_hours: 1.0,
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            date_format: "%Y-%m-%d".to_string(),
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("taskctl");
        Self {
            directory: dir.to_string_lossy().into_owned(),
        }
    }
}

impl Config {
    /// Load configuration with the resolution order:
    /// CLI args > env vars > config file > defaults.
    pub fn load(config_path: Option<&Path>, data_dir: Option<&str>) -> Result<Self, TaskCtlError> {
        // Determine config file path
        let path = config_path
            .map(PathBuf::from)
            .or_else(|| std::env::var("TASKCTL_CONFIG").ok().map(PathBuf::from))
            .unwrap_or_else(|| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("~/.config"))
                    .join("taskctl")
                    .join("config.toml")
            });

        let mut config = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| TaskCtlError::ConfigError(format!("{}: {e}", path.display())))?;
            toml::from_str::<Config>(&content)
                .map_err(|e| TaskCtlError::ConfigError(format!("{}: {e}", path.display())))?
        } else {
            Config::default()
        };

        // Override data directory: CLI arg > env var > file/default
        if let Some(dir) = data_dir {
            config.data.directory = dir.to_string();
        } else if let Ok(dir) = std::env::var("TASKCTL_DATA_DIR") {
            config.data.directory = dir;
        }

        Ok(config)
    }

    /// Resolve the data directory path, expanding `~`.
    pub fn data_dir(&self) -> PathBuf {
        expand_tilde(&self.data.directory)
    }

    /// Generate a default config TOML string.
    pub fn default_toml() -> String {
        r#"[priority.weights]
urgency = 1.0
blocking = 0.8
staleness = 0.5
quick_win = 0.3

[estimate]
point_to_hours = 1.0

[display]
color = true
date_format = "%Y-%m-%d"

[data]
directory = "~/.local/share/taskctl"
"#
        .to_string()
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights() {
        let cfg = Config::default();
        assert!((cfg.priority.weights.urgency - 1.0).abs() < f64::EPSILON);
        assert!((cfg.priority.weights.blocking - 0.8).abs() < f64::EPSILON);
        assert!((cfg.priority.weights.staleness - 0.5).abs() < f64::EPSILON);
        assert!((cfg.priority.weights.quick_win - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn default_estimate() {
        let cfg = Config::default();
        assert!((cfg.estimate.point_to_hours - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_display() {
        let cfg = Config::default();
        assert!(cfg.display.color);
        assert_eq!(cfg.display.date_format, "%Y-%m-%d");
    }

    #[test]
    fn parse_partial_toml() {
        let toml_str = r#"
[priority.weights]
urgency = 2.0
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert!((cfg.priority.weights.urgency - 2.0).abs() < f64::EPSILON);
        // Other weights should be defaults
        assert!((cfg.priority.weights.blocking - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_full_toml() {
        let toml_str = r#"
[priority.weights]
urgency = 1.5
blocking = 0.9
staleness = 0.6
quick_win = 0.4

[estimate]
point_to_hours = 2.0

[display]
color = false
date_format = "%m/%d"

[data]
directory = "/tmp/tasks"
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert!((cfg.priority.weights.urgency - 1.5).abs() < f64::EPSILON);
        assert!(!cfg.display.color);
        assert_eq!(cfg.data.directory, "/tmp/tasks");
    }

    #[test]
    fn load_nonexistent_config_returns_default() {
        let cfg = Config::load(Some(Path::new("/nonexistent/config.toml")), None).unwrap();
        assert!((cfg.priority.weights.urgency - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_dir_override() {
        let cfg = Config::load(None, Some("/tmp/my-tasks")).unwrap();
        assert_eq!(cfg.data.directory, "/tmp/my-tasks");
    }

    #[test]
    fn expand_tilde_works() {
        let result = expand_tilde("~/test");
        assert!(!result.to_string_lossy().starts_with("~/"));
    }

    #[test]
    fn expand_absolute_path() {
        let result = expand_tilde("/absolute/path");
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }
}
