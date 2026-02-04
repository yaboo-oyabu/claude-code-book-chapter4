//! `task today` command.

use crate::cli::output::{self, OutputFormat};
use crate::config::Config;
use crate::domain::scoring;
use crate::domain::status::Status;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

pub fn run(
    repo: &Repository,
    config: &Config,
    format: OutputFormat,
) -> Result<String, TaskCtlError> {
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();
    let today = Local::now().date_naive();

    // Filter: due <= today, in_progress, or pinned (all excluding done)
    let mut candidates: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.status != Status::Done)
        .filter(|t| t.due.is_some_and(|d| d <= today) || t.status == Status::InProgress || t.pinned)
        .cloned()
        .collect();

    // If no candidates, fall back to next
    if candidates.is_empty() {
        return super::next::run(repo, config, format);
    }

    scoring::sort_tasks(&mut candidates, &all_tasks, config);

    Ok(output::format_task_list(
        &candidates,
        &all_tasks,
        config,
        format,
    ))
}
