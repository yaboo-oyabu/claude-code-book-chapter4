//! `.meta.json` management for ID allocation.

use crate::error::TaskCtlError;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub next_id: u32,
}

impl Default for Meta {
    fn default() -> Self {
        Self { next_id: 1 }
    }
}

impl Meta {
    /// Read `.meta.json` from the given directory, or return default if it doesn't exist.
    pub fn load(data_dir: &Path) -> Result<Self, TaskCtlError> {
        let path = data_dir.join(".meta.json");
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).map_err(|e| TaskCtlError::ParseError {
            path: path.to_string_lossy().into_owned(),
            source: anyhow::Error::new(e),
        })
    }

    /// Save `.meta.json` to the given directory.
    pub fn save(&self, data_dir: &Path) -> Result<(), TaskCtlError> {
        let path = data_dir.join(".meta.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| TaskCtlError::ParseError {
            path: path.to_string_lossy().into_owned(),
            source: anyhow::Error::new(e),
        })?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Allocate the next ID and increment the counter.
    pub fn allocate_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_meta() {
        let meta = Meta::default();
        assert_eq!(meta.next_id, 1);
    }

    #[test]
    fn load_nonexistent_returns_default() {
        let dir = TempDir::new().unwrap();
        let meta = Meta::load(dir.path()).unwrap();
        assert_eq!(meta.next_id, 1);
    }

    #[test]
    fn save_and_load() {
        let dir = TempDir::new().unwrap();
        let meta = Meta { next_id: 42 };
        meta.save(dir.path()).unwrap();
        let loaded = Meta::load(dir.path()).unwrap();
        assert_eq!(loaded.next_id, 42);
    }

    #[test]
    fn allocate_id_increments() {
        let mut meta = Meta::default();
        assert_eq!(meta.allocate_id(), 1);
        assert_eq!(meta.allocate_id(), 2);
        assert_eq!(meta.allocate_id(), 3);
        assert_eq!(meta.next_id, 4);
    }
}
