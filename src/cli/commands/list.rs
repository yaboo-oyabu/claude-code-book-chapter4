//! `task list` command.

use crate::cli::output::{self, OutputFormat};
use crate::config::Config;
use crate::domain::date_parser;
use crate::domain::scoring;
use crate::domain::status::Status;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

#[allow(clippy::too_many_arguments)]
pub fn run(
    repo: &Repository,
    config: &Config,
    tag: Option<String>,
    status_filter: Option<String>,
    due_before: Option<String>,
    due_after: Option<String>,
    all: bool,
    format: OutputFormat,
) -> Result<String, TaskCtlError> {
    let all_tw = repo.read_all()?;
    let all_tasks: Vec<_> = all_tw.iter().map(|t| t.task.clone()).collect();
    let today = Local::now().date_naive();

    let mut tasks: Vec<_> = all_tasks.clone();

    // Filter by status
    if let Some(ref s) = status_filter {
        let target = Status::from_str_loose(s)?;
        tasks.retain(|t| t.status == target);
    } else if !all {
        // Default: hide done tasks
        tasks.retain(|t| t.status != Status::Done);
    }

    // Filter by tag
    if let Some(ref tag_filter) = tag {
        tasks.retain(|t| t.tags.iter().any(|tg| tg.eq_ignore_ascii_case(tag_filter)));
    }

    // Filter by due date
    if let Some(ref before_str) = due_before {
        let before = date_parser::parse_due(before_str, today)?;
        tasks.retain(|t| t.due.is_some_and(|d| d <= before));
    }
    if let Some(ref after_str) = due_after {
        let after = date_parser::parse_due(after_str, today)?;
        tasks.retain(|t| t.due.is_some_and(|d| d >= after));
    }

    // Sort by score
    scoring::sort_tasks(&mut tasks, &all_tasks, config);

    Ok(output::format_task_list(&tasks, &all_tasks, config, format))
}
