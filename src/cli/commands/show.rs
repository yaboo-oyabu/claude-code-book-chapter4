//! `task show` command.

use crate::cli::output::{self, OutputFormat};
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;

pub fn run(repo: &Repository, id: u32, format: OutputFormat) -> Result<String, TaskCtlError> {
    let tw = repo.read(id)?;
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();
    Ok(output::format_task_detail(&tw, &all_tasks, format))
}
