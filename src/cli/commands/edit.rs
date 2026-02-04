//! `task edit` command.

use crate::domain::date_parser;
use crate::domain::dependency;
use crate::domain::task::Estimate;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

#[allow(clippy::too_many_arguments)]
pub fn run(
    repo: &Repository,
    id: u32,
    title: Option<String>,
    due: Option<String>,
    tags: Vec<String>,
    remove_tags: Vec<String>,
    estimate: Option<String>,
    note: Option<String>,
    depends_on: Option<Vec<u32>>,
) -> Result<String, TaskCtlError> {
    let mut tw = repo.read(id)?;
    let today = Local::now().date_naive();

    if let Some(new_title) = title {
        tw.task.title = new_title;
    }

    if let Some(ref due_str) = due {
        if due_str.is_empty() {
            tw.task.due = None;
        } else {
            tw.task.due = Some(date_parser::parse_due(due_str, today)?);
        }
    }

    // Add tags
    for tag in tags {
        for t in tag.split(',').map(|s| s.trim().to_string()) {
            if !t.is_empty() && !tw.task.tags.contains(&t) {
                tw.task.tags.push(t);
            }
        }
    }

    // Remove tags
    for tag in &remove_tags {
        tw.task.tags.retain(|t| !t.eq_ignore_ascii_case(tag));
    }

    if let Some(ref est_str) = estimate {
        if est_str.is_empty() {
            tw.task.estimate = None;
        } else {
            Estimate::parse(est_str)?;
            tw.task.estimate = Some(est_str.clone());
        }
    }

    if let Some(note_text) = note {
        tw.note = note_text;
    }

    if let Some(deps) = depends_on {
        // Validate dependencies: check for cycles
        let all = repo.read_all()?;
        let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();
        for &dep_id in &deps {
            dependency::add_dependency(id, dep_id, &all_tasks)?;
        }
        tw.task.depends_on = deps;
    }

    tw.task.updated_at = Local::now();
    repo.update(&tw)?;

    Ok(format!("Updated task #{id}"))
}
