//! Command-line argument definitions using clap derive.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "task",
    version,
    about = "A CLI task manager with automatic priority adjustment"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output in JSON format.
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Override data directory path.
    #[arg(long, global = true)]
    pub data_dir: Option<String>,

    /// Override config file path.
    #[arg(long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize a default configuration file.
    Init {
        /// Overwrite existing config file.
        #[arg(long)]
        force: bool,
    },

    /// Create a new task.
    Add {
        /// Task title.
        title: String,

        /// Due date (YYYY-MM-DD, today, tomorrow, +3d, +1w, monday..sunday).
        #[arg(long)]
        due: Option<String>,

        /// Tags (comma-separated or repeated).
        #[arg(long, short)]
        tag: Vec<String>,

        /// Estimate (e.g., 30m, 2h, 3p).
        #[arg(long)]
        estimate: Option<String>,

        /// Note to attach.
        #[arg(long)]
        note: Option<String>,

        /// Task IDs this depends on.
        #[arg(long = "depends")]
        depends_on: Vec<u32>,
    },

    /// Show task details.
    Show {
        /// Task ID.
        id: u32,
    },

    /// List tasks.
    List {
        /// Filter by tag.
        #[arg(long, short)]
        tag: Option<String>,

        /// Filter by status.
        #[arg(long)]
        status: Option<String>,

        /// Filter: due on or before this date.
        #[arg(long)]
        due_before: Option<String>,

        /// Filter: due on or after this date.
        #[arg(long)]
        due_after: Option<String>,

        /// Show all tasks including completed.
        #[arg(long)]
        all: bool,
    },

    /// Edit a task.
    Edit {
        /// Task ID.
        id: u32,

        /// New title.
        #[arg(long)]
        title: Option<String>,

        /// New due date (empty to remove).
        #[arg(long)]
        due: Option<String>,

        /// Add tags.
        #[arg(long, short)]
        tag: Vec<String>,

        /// Remove tags.
        #[arg(long)]
        remove_tag: Vec<String>,

        /// New estimate (empty to remove).
        #[arg(long)]
        estimate: Option<String>,

        /// New note.
        #[arg(long)]
        note: Option<String>,

        /// New dependencies (replaces existing).
        #[arg(long = "depends")]
        depends_on: Option<Vec<u32>>,
    },

    /// Delete a task.
    Delete {
        /// Task ID.
        id: u32,

        /// Skip confirmation prompt.
        #[arg(long)]
        force: bool,
    },

    /// Start a task (set status to in_progress).
    Start {
        /// Task ID.
        id: u32,
    },

    /// Complete a task (set status to done).
    Done {
        /// Task ID.
        id: u32,
    },

    /// Reopen a task (set status to pending).
    Pending {
        /// Task ID.
        id: u32,
    },

    /// Pin a task to the top.
    Pin {
        /// Task ID.
        id: u32,
    },

    /// Unpin a task.
    Unpin {
        /// Task ID.
        id: u32,
    },

    /// Add a dependency.
    Depends {
        /// Task ID.
        id: u32,

        /// Depends on this task ID.
        #[arg(long)]
        on: u32,
    },

    /// Remove a dependency.
    Undepends {
        /// Task ID.
        id: u32,

        /// Remove dependency on this task ID.
        #[arg(long)]
        on: u32,
    },

    /// Show dependency tree.
    Tree {
        /// Task ID.
        id: u32,
    },

    /// Show the next recommended task.
    Next,

    /// Show today's tasks.
    Today,

    /// Search tasks by title and note.
    Search {
        /// Search query.
        query: String,

        /// Filter by tag.
        #[arg(long, short)]
        tag: Option<String>,

        /// Filter by status.
        #[arg(long)]
        status: Option<String>,
    },

    /// Run data migration.
    Migrate {
        /// Show what would be migrated without making changes.
        #[arg(long)]
        dry_run: bool,
    },

    /// Generate shell completions.
    Completions {
        /// Shell type (bash, zsh, fish).
        shell: String,
    },
}
