//! Scoring algorithm for task prioritization.

use crate::config::Config;
use crate::domain::dependency;
use crate::domain::status::Status;
use crate::domain::task::{Estimate, Task};
use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct ScoreResult {
    pub score: f64,
    pub primary_factors: Vec<String>,
}

/// Calculate urgency signal (0.0 - 10.0) based on due date proximity.
pub fn urgency_signal(due: Option<NaiveDate>, today: NaiveDate) -> f64 {
    let Some(due) = due else { return 0.0 };
    let days_remaining = (due - today).num_days();
    if days_remaining <= 0 {
        10.0
    } else if days_remaining >= 30 {
        0.0
    } else {
        10.0 * (1.0 - days_remaining as f64 / 30.0)
    }
}

/// Calculate blocking signal (0.0 - 10.0) based on how many tasks this one blocks.
pub fn blocking_signal(task_id: u32, all_tasks: &[Task]) -> f64 {
    let count = dependency::get_blocking_tasks(task_id, all_tasks).len();
    if count == 0 {
        0.0
    } else if count >= 5 {
        10.0
    } else {
        10.0 * (count as f64 / 5.0)
    }
}

/// Calculate staleness signal (0.0 - 10.0) based on days since last update.
pub fn staleness_signal(updated_at_date: NaiveDate, today: NaiveDate) -> f64 {
    let days = (today - updated_at_date).num_days();
    if days <= 0 {
        0.0
    } else if days >= 14 {
        10.0
    } else {
        10.0 * (days as f64 / 14.0)
    }
}

/// Calculate quick-win signal (0.0 - 10.0) based on estimate.
pub fn quick_win_signal(estimate: Option<&str>, point_to_hours: f64) -> f64 {
    let Some(est_str) = estimate else {
        return 0.0;
    };
    let Ok(est) = Estimate::parse(est_str) else {
        return 0.0;
    };
    let hours = est.to_hours(point_to_hours);
    if hours <= 0.5 {
        10.0
    } else if hours >= 8.0 {
        0.0
    } else {
        10.0 * (1.0 - hours / 8.0)
    }
}

/// Calculate blocked penalty.
pub fn blocked_penalty(task: &Task, all_tasks: &[Task]) -> f64 {
    if dependency::is_blocked(task, all_tasks) {
        -1000.0
    } else {
        0.0
    }
}

/// Calculate the total score for a task using today's date.
#[allow(dead_code)]
pub fn calculate_score(task: &Task, all_tasks: &[Task], config: &Config) -> ScoreResult {
    let today = chrono::Local::now().date_naive();
    calculate_score_with_date(task, all_tasks, config, today)
}

/// Calculate the total score for a task with a specific date (testable).
pub fn calculate_score_with_date(
    task: &Task,
    all_tasks: &[Task],
    config: &Config,
    today: NaiveDate,
) -> ScoreResult {
    let w = &config.priority.weights;

    let urgency = urgency_signal(task.due, today);
    let blocking = blocking_signal(task.id, all_tasks);
    let staleness = staleness_signal(task.updated_at.date_naive(), today);
    let quick_win = quick_win_signal(task.estimate.as_deref(), config.estimate.point_to_hours);
    let penalty = blocked_penalty(task, all_tasks);

    let score = w.urgency * urgency
        + w.blocking * blocking
        + w.staleness * staleness
        + w.quick_win * quick_win
        + penalty;

    let primary_factors = generate_summary(task, all_tasks, today);

    ScoreResult {
        score,
        primary_factors,
    }
}

/// Sort tasks: pinned first (by pinned_at asc), then by score desc, then by created_at asc.
pub fn sort_tasks(tasks: &mut [Task], all_tasks: &[Task], config: &Config) {
    let today = chrono::Local::now().date_naive();
    sort_tasks_with_date(tasks, all_tasks, config, today);
}

pub fn sort_tasks_with_date(
    tasks: &mut [Task],
    all_tasks: &[Task],
    config: &Config,
    today: NaiveDate,
) {
    tasks.sort_by(|a, b| {
        // Pinned first
        match (a.pinned, b.pinned) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            (true, true) => {
                // Both pinned: by pinned_at ascending
                return a.pinned_at.cmp(&b.pinned_at);
            }
            (false, false) => {}
        }

        // By score descending
        let score_a = calculate_score_with_date(a, all_tasks, config, today).score;
        let score_b = calculate_score_with_date(b, all_tasks, config, today).score;
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });
}

/// Generate summary factors for display (max 3 items).
pub fn generate_summary(task: &Task, all_tasks: &[Task], today: NaiveDate) -> Vec<String> {
    let mut factors = Vec::new();

    // Due date
    if let Some(due) = task.due {
        let days = (due - today).num_days();
        let due_str = if days < 0 {
            "due: overdue".to_string()
        } else if days == 0 {
            "due: today".to_string()
        } else if days == 1 {
            "due: tomorrow".to_string()
        } else {
            format!("due: {}", due.format("%m/%d"))
        };
        factors.push(due_str);
    }

    // Blocking count
    let blocking = dependency::get_blocking_tasks(task.id, all_tasks);
    if !blocking.is_empty() {
        factors.push(format!("blocks: {}", blocking.len()));
    }

    // Estimate
    if let Some(ref est) = task.estimate {
        factors.push(format!("est: {est}"));
    }

    // Pinned
    if task.pinned {
        factors.push("pinned".to_string());
    }

    // Blocked by
    if dependency::is_blocked(task, all_tasks) {
        let blockers: Vec<String> = task
            .depends_on
            .iter()
            .filter(|&&dep_id| {
                all_tasks
                    .iter()
                    .find(|t| t.id == dep_id)
                    .is_some_and(|t| t.status != Status::Done)
            })
            .map(|id| format!("#{id}"))
            .collect();
        factors.push(format!("blocked by: {}", blockers.join(", ")));
    }

    factors.truncate(3);
    factors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::task::Task;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 2, 5).unwrap()
    }

    #[test]
    fn urgency_no_due() {
        assert!((urgency_signal(None, today())).abs() < f64::EPSILON);
    }

    #[test]
    fn urgency_overdue() {
        let due = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
        assert!((urgency_signal(Some(due), today()) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn urgency_today() {
        assert!((urgency_signal(Some(today()), today()) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn urgency_far_future() {
        let due = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
        assert!((urgency_signal(Some(due), today())).abs() < f64::EPSILON);
    }

    #[test]
    fn urgency_15_days() {
        let due = NaiveDate::from_ymd_opt(2025, 2, 20).unwrap();
        let signal = urgency_signal(Some(due), today());
        assert!((signal - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn blocking_no_tasks() {
        let tasks = vec![Task::new(1, "T".into())];
        assert!((blocking_signal(1, &tasks)).abs() < f64::EPSILON);
    }

    #[test]
    fn blocking_five_or_more() {
        let mut tasks = vec![Task::new(1, "T".into())];
        for i in 2..=6 {
            let mut t = Task::new(i, format!("T{i}"));
            t.depends_on = vec![1];
            tasks.push(t);
        }
        assert!((blocking_signal(1, &tasks) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn staleness_today() {
        assert!((staleness_signal(today(), today())).abs() < f64::EPSILON);
    }

    #[test]
    fn staleness_14_days() {
        let updated = today() - chrono::Duration::days(14);
        assert!((staleness_signal(updated, today()) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn staleness_7_days() {
        let updated = today() - chrono::Duration::days(7);
        assert!((staleness_signal(updated, today()) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn quick_win_30m() {
        assert!((quick_win_signal(Some("30m"), 1.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn quick_win_8h() {
        assert!((quick_win_signal(Some("8h"), 1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn quick_win_none() {
        assert!((quick_win_signal(None, 1.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn blocked_penalty_not_blocked() {
        let tasks = vec![Task::new(1, "T".into())];
        assert!((blocked_penalty(&tasks[0], &tasks)).abs() < f64::EPSILON);
    }

    #[test]
    fn blocked_penalty_is_blocked() {
        let tasks = vec![Task::new(1, "T".into()), {
            let mut t = Task::new(2, "T2".into());
            t.depends_on = vec![1];
            t
        }];
        assert!((blocked_penalty(&tasks[1], &tasks) - (-1000.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn sort_pinned_first() {
        let config = Config::default();
        let mut tasks = vec![Task::new(1, "Normal".into()), {
            let mut t = Task::new(2, "Pinned".into());
            t.pinned = true;
            t.pinned_at = Some(chrono::Local::now());
            t
        }];
        let all = tasks.clone();
        sort_tasks_with_date(&mut tasks, &all, &config, today());
        assert_eq!(tasks[0].id, 2); // Pinned first
    }

    #[test]
    fn summary_with_due() {
        let mut task = Task::new(1, "T".into());
        task.due = Some(today());
        let factors = generate_summary(&task, &[], today());
        assert!(factors.iter().any(|f| f.contains("due: today")));
    }

    #[test]
    fn summary_max_three() {
        let mut task = Task::new(1, "T".into());
        task.due = Some(today());
        task.estimate = Some("2h".into());
        task.pinned = true;
        task.depends_on = vec![99]; // blocked
        let all = vec![task.clone(), Task::new(99, "Dep".into())];
        let factors = generate_summary(&task, &all, today());
        assert!(factors.len() <= 3);
    }
}
