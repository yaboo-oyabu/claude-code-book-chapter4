//! `task add` command.

use crate::domain::date_parser;
use crate::domain::task::Estimate;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

pub fn run(
    repo: &Repository,
    title: String,
    due: Option<String>,
    tags: Vec<String>,
    estimate: Option<String>,
    note: Option<String>,
    depends_on: Vec<u32>,
) -> Result<String, TaskCtlError> {
    // Validate estimate if provided
    if let Some(ref est) = estimate {
        Estimate::parse(est)?;
    }

    // Parse due date if provided
    let due_date = if let Some(ref due_str) = due {
        let today = Local::now().date_naive();
        Some(date_parser::parse_due(due_str, today)?)
    } else {
        None
    };

    // Flatten tags (handle comma-separated)
    let all_tags: Vec<String> = tags
        .into_iter()
        .flat_map(|t| {
            t.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
        })
        .filter(|t| !t.is_empty())
        .collect();

    let mut tw = repo.create(title, |task| {
        task.due = due_date;
        task.tags.clone_from(&all_tags);
        task.estimate.clone_from(&estimate);
        task.depends_on.clone_from(&depends_on);
    })?;

    let id = tw.task.id;
    let task_title = tw.task.title.clone();

    // Write note if provided
    if let Some(note_text) = note {
        tw.note = note_text;
        repo.update(&tw)?;
    }

    Ok(format!("Created task #{id}: {task_title}"))
}
