//! `task next` command.

use crate::cli::output::{self, OutputFormat};
use crate::config::Config;
use crate::domain::dependency;
use crate::domain::scoring;
use crate::domain::status::Status;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;

pub fn run(
    repo: &Repository,
    config: &Config,
    format: OutputFormat,
) -> Result<String, TaskCtlError> {
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();

    // Filter: pending/in_progress, not blocked
    let mut candidates: Vec<_> = all_tasks
        .iter()
        .filter(|t| t.status != Status::Done)
        .filter(|t| !dependency::is_blocked(t, &all_tasks))
        .cloned()
        .collect();

    if candidates.is_empty() {
        return Ok(if format == OutputFormat::Json {
            "null".to_string()
        } else {
            "No actionable tasks found.".to_string()
        });
    }

    scoring::sort_tasks(&mut candidates, &all_tasks, config);

    let best = &candidates[0];
    Ok(output::format_task_next(best, &all_tasks, config, format))
}
