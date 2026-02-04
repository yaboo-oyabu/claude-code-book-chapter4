//! `task depends`, `task undepends`, `task tree` commands.

use crate::cli::output::{self, OutputFormat};
use crate::domain::dependency;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;

pub fn run_depends(repo: &Repository, id: u32, on: u32) -> Result<String, TaskCtlError> {
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();

    dependency::add_dependency(id, on, &all_tasks)?;

    let mut tw = repo.read(id)?;
    if !tw.task.depends_on.contains(&on) {
        tw.task.depends_on.push(on);
        tw.task.updated_at = Local::now();
        repo.update(&tw)?;
    }

    Ok(format!("Added dependency: #{id} depends on #{on}"))
}

pub fn run_undepends(repo: &Repository, id: u32, on: u32) -> Result<String, TaskCtlError> {
    let mut tw = repo.read(id)?;
    dependency::remove_dependency(&mut tw.task, on);
    tw.task.updated_at = Local::now();
    repo.update(&tw)?;

    Ok(format!(
        "Removed dependency: #{id} no longer depends on #{on}"
    ))
}

pub fn run_tree(repo: &Repository, id: u32, format: OutputFormat) -> Result<String, TaskCtlError> {
    let all = repo.read_all()?;
    let all_tasks: Vec<_> = all.iter().map(|t| t.task.clone()).collect();

    let tree =
        dependency::get_dependency_tree(id, &all_tasks).ok_or(TaskCtlError::TaskNotFound(id))?;

    Ok(output::format_tree(&tree, format))
}
