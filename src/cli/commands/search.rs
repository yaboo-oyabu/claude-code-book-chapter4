//! `task search` command.

use crate::cli::output::{self, OutputFormat};
use crate::config::Config;
use crate::domain::scoring;
use crate::domain::status::Status;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;

pub fn run(
    repo: &Repository,
    config: &Config,
    query: String,
    tag: Option<String>,
    status_filter: Option<String>,
    format: OutputFormat,
) -> Result<String, TaskCtlError> {
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();
    let query_lower = query.to_lowercase();

    let mut results: Vec<_> = all
        .iter()
        .filter(|tw| {
            tw.task.title.to_lowercase().contains(&query_lower)
                || tw.note.to_lowercase().contains(&query_lower)
        })
        .map(|tw| tw.task.clone())
        .collect();

    // Filter by tag
    if let Some(ref tag_filter) = tag {
        results.retain(|t| t.tags.iter().any(|tg| tg.eq_ignore_ascii_case(tag_filter)));
    }

    // Filter by status
    if let Some(ref s) = status_filter {
        let target = Status::from_str_loose(s)?;
        results.retain(|t| t.status == target);
    }

    scoring::sort_tasks(&mut results, &all_tasks, config);

    Ok(output::format_task_list(
        &results, &all_tasks, config, format,
    ))
}
