//! `task migrate` command.

use crate::domain::task::SCHEMA_VERSION;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use chrono::Local;
use std::path::Path;

pub fn run(repo: &Repository, data_dir: &Path, dry_run: bool) -> Result<String, TaskCtlError> {
    let all = repo.read_all()?;
    let outdated: Vec<_> = all
        .iter()
        .filter(|tw| tw.task.schema_version < SCHEMA_VERSION)
        .collect();

    if outdated.is_empty() {
        return Ok("All tasks are up to date. No migration needed.".to_string());
    }

    let mut lines = vec![format!(
        "Found {} task(s) requiring migration to schema version {SCHEMA_VERSION}:",
        outdated.len()
    )];

    for tw in &outdated {
        lines.push(format!(
            "  #{} \"{}\" (v{} → v{SCHEMA_VERSION})",
            tw.task.id, tw.task.title, tw.task.schema_version
        ));
    }

    if dry_run {
        lines.push("\n(dry run — no changes made)".to_string());
        return Ok(lines.join("\n"));
    }

    // Create backup
    let backup_name = format!(".backup-{}", Local::now().format("%Y%m%d-%H%M%S"));
    let backup_dir = data_dir.join(&backup_name);
    std::fs::create_dir_all(&backup_dir)?;

    for tw in &outdated {
        let src = data_dir.join(format!("{}.md", tw.task.id));
        let dst = backup_dir.join(format!("{}.md", tw.task.id));
        std::fs::copy(&src, &dst)?;
    }

    lines.push(format!("Backup created: {}", backup_dir.display()));

    // Migrate
    for tw in &outdated {
        let mut tw = (*tw).clone();
        tw.task.schema_version = SCHEMA_VERSION;
        repo.update(&tw)?;
    }

    lines.push("Migration complete.".to_string());
    Ok(lines.join("\n"))
}
