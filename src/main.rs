mod cli;
mod config;
mod domain;
mod error;
mod storage;

use crate::cli::args::{Cli, Command};
use crate::cli::output::OutputFormat;
use crate::config::Config;
use crate::error::TaskCtlError;
use crate::storage::repository::Repository;
use clap::Parser;
use std::path::Path;
use std::process;

fn main() {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    let result = run(cli);

    match result {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            if let TaskCtlError::LockError(_) = e {
                eprintln!(
                    "Hint: If the problem persists, manually delete the .lock file in your data directory."
                );
            }
            process::exit(e.exit_code());
        }
    }
}

fn run(cli: Cli) -> Result<String, TaskCtlError> {
    let format = OutputFormat::from_flags(cli.json, cli.no_color);

    // Handle init and completions before loading config/repo
    match &cli.command {
        Command::Init { force } => return cli::commands::init::run(*force),
        Command::Completions { shell } => return cli::commands::completions::run(shell),
        _ => {}
    }

    let config = Config::load(
        cli.config.as_deref().map(Path::new),
        cli.data_dir.as_deref(),
    )?;
    let repo = Repository::new(config.data_dir());

    match cli.command {
        Command::Init { .. } | Command::Completions { .. } => unreachable!(),

        Command::Add {
            title,
            due,
            tag,
            estimate,
            note,
            depends_on,
        } => cli::commands::add::run(&repo, title, due, tag, estimate, note, depends_on),

        Command::Show { id } => cli::commands::show::run(&repo, id, format),

        Command::List {
            tag,
            status,
            due_before,
            due_after,
            all,
        } => cli::commands::list::run(
            &repo, &config, tag, status, due_before, due_after, all, format,
        ),

        Command::Edit {
            id,
            title,
            due,
            tag,
            remove_tag,
            estimate,
            note,
            depends_on,
        } => cli::commands::edit::run(
            &repo, id, title, due, tag, remove_tag, estimate, note, depends_on,
        ),

        Command::Delete { id, force } => cli::commands::delete::run(&repo, id, force),

        Command::Start { id } => cli::commands::status::run_start(&repo, id),
        Command::Done { id } => cli::commands::status::run_done(&repo, id),
        Command::Pending { id } => cli::commands::status::run_pending(&repo, id),

        Command::Pin { id } => cli::commands::pin::run_pin(&repo, id),
        Command::Unpin { id } => cli::commands::pin::run_unpin(&repo, id),

        Command::Depends { id, on } => cli::commands::depends::run_depends(&repo, id, on),
        Command::Undepends { id, on } => cli::commands::depends::run_undepends(&repo, id, on),
        Command::Tree { id } => cli::commands::depends::run_tree(&repo, id, format),

        Command::Next => cli::commands::next::run(&repo, &config, format),
        Command::Today => cli::commands::today::run(&repo, &config, format),

        Command::Search { query, tag, status } => {
            cli::commands::search::run(&repo, &config, query, tag, status, format)
        }

        Command::Migrate { dry_run } => {
            cli::commands::migrate::run(&repo, &config.data_dir(), dry_run)
        }
    }
}
