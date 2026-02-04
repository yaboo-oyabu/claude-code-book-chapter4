//! `task pin` and `task unpin` commands.

use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

pub fn run_pin(repo: &Repository, id: u32) -> Result<String, TaskCtlError> {
    let mut tw = repo.read(id)?;

    if !tw.task.pinned {
        tw.task.pinned = true;
        tw.task.pinned_at = Some(Local::now());
        tw.task.updated_at = Local::now();
        repo.update(&tw)?;
    }
    // Idempotent: if already pinned, do nothing

    Ok(format!("Pinned task #{id}"))
}

pub fn run_unpin(repo: &Repository, id: u32) -> Result<String, TaskCtlError> {
    let mut tw = repo.read(id)?;

    if tw.task.pinned {
        tw.task.pinned = false;
        tw.task.pinned_at = None;
        tw.task.updated_at = Local::now();
        repo.update(&tw)?;
    }
    // Idempotent: if already unpinned, do nothing

    Ok(format!("Unpinned task #{id}"))
}
