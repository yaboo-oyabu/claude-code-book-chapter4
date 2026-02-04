//! Advisory file locking with timeout and stale lock detection.

use crate::error::TaskCtlError;
use fs2::FileExt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(100);

pub struct FileLock {
    _file: File,
    path: PathBuf,
}

impl FileLock {
    /// Acquire an advisory lock on the data directory.
    /// Times out after 5 seconds. Detects and removes stale locks.
    pub fn acquire(data_dir: &Path) -> Result<Self, TaskCtlError> {
        let lock_path = data_dir.join(".lock");

        // Check for stale lock
        if lock_path.exists() {
            if let Ok(content) = fs::read_to_string(&lock_path) {
                if let Ok(pid) = content.trim().parse::<u32>() {
                    if !process_exists(pid) {
                        // Stale lock — remove it
                        let _ = fs::remove_file(&lock_path);
                    }
                }
            }
        }

        let start = Instant::now();
        loop {
            match File::create(&lock_path) {
                Ok(file) => match file.try_lock_exclusive() {
                    Ok(()) => {
                        // Write our PID
                        let mut f = file;
                        let _ = write!(f, "{}", std::process::id());
                        return Ok(Self {
                            _file: f,
                            path: lock_path,
                        });
                    }
                    Err(_) if start.elapsed() < LOCK_TIMEOUT => {
                        std::thread::sleep(LOCK_RETRY_INTERVAL);
                    }
                    Err(e) => return Err(TaskCtlError::LockError(e)),
                },
                Err(e) => return Err(TaskCtlError::LockError(e)),
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn process_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn acquire_and_release() {
        let dir = TempDir::new().unwrap();
        let lock_path = dir.path().join(".lock");

        {
            let _lock = FileLock::acquire(dir.path()).unwrap();
            assert!(lock_path.exists());
        }
        // Lock file should be removed after drop
        assert!(!lock_path.exists());
    }

    #[test]
    fn stale_lock_cleanup() {
        let dir = TempDir::new().unwrap();
        let lock_path = dir.path().join(".lock");

        // Write a stale PID
        fs::write(&lock_path, "999999999").unwrap();

        // Should succeed by cleaning up the stale lock
        let _lock = FileLock::acquire(dir.path()).unwrap();
        assert!(lock_path.exists());
    }
}
