//! `task delete` command.

use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use std::io::{self, Write};

pub fn run(repo: &Repository, id: u32, force: bool) -> Result<String, TaskCtlError> {
    // Verify task exists
    let tw = repo.read(id)?;

    if !force {
        print!("Delete task #{} \"{}\"? [y/N] ", id, tw.task.title);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok("Cancelled.".to_string());
        }
    }

    repo.delete(id)?;
    Ok(format!("Deleted task #{id}"))
}
