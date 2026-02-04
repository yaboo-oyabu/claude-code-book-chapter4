//! `task start`, `task done`, `task pending` commands.

use std::fmt::Write;

use crate::domain::dependency;
use crate::domain::status::{self, Status};
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

pub fn run_start(repo: &Repository, id: u32) -> Result<String, TaskCtlError> {
    transition(repo, id, Status::InProgress, "Started")
}

pub fn run_done(repo: &Repository, id: u32) -> Result<String, TaskCtlError> {
    let mut tw = repo.read(id)?;
    let new_status = status::transition(tw.task.status, Status::Done)?;

    let mut msg = if tw.task.status == new_status {
        format!("Completed task #{id}")
    } else {
        tw.task.status = new_status;
        tw.task.updated_at = Local::now();
        repo.update(&tw)?;
        format!("Completed task #{id}")
    };

    // Check for unblocked tasks
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();
    let unblocked: Vec<u32> = dependency::get_blocking_tasks(id, &all_tasks)
        .into_iter()
        .filter(|&blocked_id| {
            // A task is unblocked if ALL of its dependencies are now done
            if let Some(blocked) = all_tasks.iter().find(|t| t.id == blocked_id) {
                !dependency::is_blocked(blocked, &all_tasks)
            } else {
                false
            }
        })
        .collect();

    if !unblocked.is_empty() {
        let ids: Vec<String> = unblocked.iter().map(|id| format!("#{id}")).collect();
        let _ = write!(msg, "\n  Unblocked: {}", ids.join(", "));
    }

    Ok(msg)
}

pub fn run_pending(repo: &Repository, id: u32) -> Result<String, TaskCtlError> {
    transition(repo, id, Status::Pending, "Reopened")
}

fn transition(
    repo: &Repository,
    id: u32,
    target: Status,
    verb: &str,
) -> Result<String, TaskCtlError> {
    let mut tw = repo.read(id)?;
    let new_status = status::transition(tw.task.status, target)?;

    if tw.task.status != new_status {
        tw.task.status = new_status;
        tw.task.updated_at = Local::now();
        repo.update(&tw)?;
    }

    Ok(format!("{verb} task #{id}"))
}
