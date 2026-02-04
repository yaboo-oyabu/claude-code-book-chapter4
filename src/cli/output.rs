//! Output formatting for tasks (color, plain, JSON).

use crate::config::Config;
use crate::domain::dependency::{self, TreeNode};
use crate::domain::scoring;
use crate::domain::status::Status;
use crate::domain::task::{Task, TaskWithNote};
use chrono::NaiveDate;
use colored::Colorize;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Color,
    Plain,
    Json,
}

impl OutputFormat {
    pub fn from_flags(json: bool, no_color: bool) -> Self {
        if json {
            Self::Json
        } else if no_color || std::env::var("NO_COLOR").is_ok() || !atty_stdout() {
            Self::Plain
        } else {
            Self::Color
        }
    }
}

fn atty_stdout() -> bool {
    // Simple check - colored crate handles this too
    true
}

/// Format a task list for display.
pub fn format_task_list(
    tasks: &[Task],
    all_tasks: &[Task],
    config: &Config,
    format: OutputFormat,
) -> String {
    if tasks.is_empty() {
        return match format {
            OutputFormat::Json => "[]".to_string(),
            _ => "No tasks found. Use 'task add' to create one.".to_string(),
        };
    }

    if format == OutputFormat::Json {
        return format_task_list_json(tasks, all_tasks, config);
    }

    let today = chrono::Local::now().date_naive();
    let mut lines = Vec::new();

    // Header
    let header = format!(
        "{:>4}  {:<12}  {:<36}  {:<10}  {:<5}  {}",
        "#", "Status", "Title", "Due", "Est", "Tags"
    );
    lines.push(header);

    for task in tasks {
        let status_str = format_status_short(task.status, format);
        let blocked = dependency::is_blocked(task, all_tasks);
        let title_display = if blocked {
            format!("{} [blocked]", task.title)
        } else {
            task.title.clone()
        };
        let title_truncated = if title_display.len() > 36 {
            format!("{}...", &title_display[..33])
        } else {
            title_display
        };
        let due_str = task
            .due
            .map(|d| format_due_short(d, today))
            .unwrap_or_default();
        let est_str = task.estimate.as_deref().unwrap_or("");
        let tags_str = task.tags.join(", ");

        let line = format!(
            "{:>4}  {:<12}  {:<36}  {:<10}  {:<5}  {}",
            task.id, status_str, title_truncated, due_str, est_str, tags_str
        );

        if format == OutputFormat::Color {
            let colored_line = match task.status {
                Status::InProgress => line.green().to_string(),
                Status::Done => line.dimmed().to_string(),
                Status::Pending if blocked => line.yellow().to_string(),
                Status::Pending => line,
            };
            lines.push(colored_line);
        } else {
            lines.push(line);
        }
    }

    lines.join("\n")
}

fn format_task_list_json(tasks: &[Task], all_tasks: &[Task], config: &Config) -> String {
    let today = chrono::Local::now().date_naive();
    let items: Vec<serde_json::Value> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let score_result = scoring::calculate_score_with_date(task, all_tasks, config, today);
            serde_json::json!({
                "id": task.id,
                "title": task.title,
                "status": task.status,
                "created_at": task.created_at.to_rfc3339(),
                "updated_at": task.updated_at.to_rfc3339(),
                "due": task.due,
                "tags": task.tags,
                "estimate": task.estimate,
                "depends_on": task.depends_on,
                "pinned": task.pinned,
                "pinned_at": task.pinned_at.map(|d| d.to_rfc3339()),
                "score_info": {
                    "sort_position": i + 1,
                    "primary_factors": score_result.primary_factors,
                }
            })
        })
        .collect();
    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
}

/// Format task detail view.
pub fn format_task_detail(tw: &TaskWithNote, all_tasks: &[Task], format: OutputFormat) -> String {
    if format == OutputFormat::Json {
        return serde_json::to_string_pretty(&tw.task).unwrap_or_default();
    }

    let task = &tw.task;
    let today = chrono::Local::now().date_naive();
    let mut lines = Vec::new();

    let header = format!("Task #{}", task.id);
    if format == OutputFormat::Color {
        lines.push(header.bold().to_string());
        lines.push("━".repeat(40).dimmed().to_string());
    } else {
        lines.push(header);
        lines.push("━".repeat(40));
    }

    lines.push(format!("Title:      {}", task.title));
    lines.push(format!(
        "Status:     {}",
        format_status_long(task.status, format)
    ));

    if let Some(due) = task.due {
        let days = (due - today).num_days();
        let due_info = if days < 0 {
            format!("{due} (overdue)")
        } else if days == 0 {
            format!("{due} (today)")
        } else if days == 1 {
            format!("{due} (tomorrow)")
        } else {
            format!("{due} ({days} days left)")
        };
        lines.push(format!("Due:        {due_info}"));
    }

    if let Some(ref est) = task.estimate {
        lines.push(format!("Estimate:   {est}"));
    }

    if !task.tags.is_empty() {
        lines.push(format!("Tags:       {}", task.tags.join(", ")));
    }

    lines.push(format!(
        "Pinned:     {}",
        if task.pinned { "Yes" } else { "No" }
    ));
    lines.push(format!(
        "Created:    {}",
        task.created_at.format("%Y-%m-%d %H:%M")
    ));
    lines.push(format!(
        "Updated:    {}",
        task.updated_at.format("%Y-%m-%d %H:%M")
    ));

    // Dependencies
    if !task.depends_on.is_empty() || !dependency::get_blocking_tasks(task.id, all_tasks).is_empty()
    {
        lines.push(String::new());
        lines.push("Dependencies:".to_string());
        for &dep_id in &task.depends_on {
            if let Some(dep) = all_tasks.iter().find(|t| t.id == dep_id) {
                let check = if dep.status == Status::Done {
                    " ✓"
                } else {
                    ""
                };
                lines.push(format!(
                    "  depends on: #{} {} [{}]{check}",
                    dep.id, dep.title, dep.status
                ));
            } else {
                lines.push(format!("  depends on: #{dep_id} (deleted)"));
            }
        }
        for blocked_id in dependency::get_blocking_tasks(task.id, all_tasks) {
            if let Some(blocked) = all_tasks.iter().find(|t| t.id == blocked_id) {
                lines.push(format!("  blocks:     #{} {}", blocked.id, blocked.title));
            }
        }
    }

    // Note
    if !tw.note.is_empty() {
        lines.push(String::new());
        lines.push("Note:".to_string());
        for line in tw.note.lines() {
            lines.push(format!("  {line}"));
        }
    }

    lines.join("\n")
}

/// Format the "next" task display.
pub fn format_task_next(
    task: &Task,
    all_tasks: &[Task],
    _config: &Config,
    format: OutputFormat,
) -> String {
    if format == OutputFormat::Json {
        return serde_json::to_string_pretty(task).unwrap_or_default();
    }

    let today = chrono::Local::now().date_naive();
    let factors = scoring::generate_summary(task, all_tasks, today);
    let factors_str = factors.join(" | ");

    let line1 = format!("→ #{} {}", task.id, task.title);
    let line2 = format!("   {factors_str}");

    if format == OutputFormat::Color {
        format!("{}\n{}", line1.bold(), line2.dimmed())
    } else {
        format!("{line1}\n{line2}")
    }
}

/// Format a dependency tree.
pub fn format_tree(node: &TreeNode, format: OutputFormat) -> String {
    let mut lines = Vec::new();
    let _ = format; // reserved for future color support
    format_tree_node(node, "", true, &mut lines);
    lines.join("\n")
}

fn format_tree_node(node: &TreeNode, prefix: &str, is_root: bool, lines: &mut Vec<String>) {
    let status_str = match node.status {
        Status::Done => "[done] ✓",
        Status::InProgress => "[in_progress]",
        Status::Pending => "[pending]",
    };

    let line = format!("#{} {} {status_str}", node.id, node.title);

    if is_root {
        lines.push(line);
    } else {
        lines.push(format!("{prefix}{line}"));
    }

    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == node.children.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };

        let child_line_prefix = if is_root {
            connector.to_string()
        } else {
            format!("{prefix}{connector}")
        };

        format_tree_node(child, &child_prefix, false, lines);
        // Replace the last pushed line's prefix
        if let Some(last) = lines.last_mut() {
            let content = last.trim_start().to_string();
            *last = format!("{child_line_prefix}{content}");
        }
    }
}

fn format_status_short(status: Status, format: OutputFormat) -> String {
    match (status, format) {
        (Status::InProgress, OutputFormat::Color) => "● progress".green().to_string(),
        (Status::InProgress, _) => "● progress".to_string(),
        (Status::Pending, OutputFormat::Color) => "○ pending".to_string(),
        (Status::Pending, _) => "○ pending".to_string(),
        (Status::Done, OutputFormat::Color) => "✓ done".dimmed().to_string(),
        (Status::Done, _) => "✓ done".to_string(),
    }
}

fn format_status_long(status: Status, format: OutputFormat) -> String {
    match (status, format) {
        (Status::InProgress, OutputFormat::Color) => "● in_progress".green().to_string(),
        (Status::InProgress, _) => "● in_progress".to_string(),
        (Status::Pending, _) => "○ pending".to_string(),
        (Status::Done, OutputFormat::Color) => "✓ done".dimmed().to_string(),
        (Status::Done, _) => "✓ done".to_string(),
    }
}

fn format_due_short(due: NaiveDate, today: NaiveDate) -> String {
    let days = (due - today).num_days();
    if days < 0 {
        "overdue".to_string()
    } else if days == 0 {
        "today".to_string()
    } else if days == 1 {
        "tomorrow".to_string()
    } else {
        due.format("%m/%d").to_string()
    }
}
