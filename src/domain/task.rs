//! Task struct and estimate parsing.

use crate::domain::status::Status;
use crate::error::TaskCtlError;
use chrono::{DateTime, Local, NaiveDate};
use serde::{Deserialize, Serialize};

/// Current schema version for task files.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimate: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<u32>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<DateTime<Local>>,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
}

fn default_schema_version() -> u32 {
    SCHEMA_VERSION
}

/// The note (markdown body) stored separately from front matter.
#[derive(Debug, Clone)]
pub struct TaskWithNote {
    pub task: Task,
    pub note: String,
}

impl Task {
    /// Create a new task with default values.
    pub fn new(id: u32, title: String) -> Self {
        let now = Local::now();
        Self {
            id,
            title,
            status: Status::Pending,
            created_at: now,
            updated_at: now,
            due: None,
            tags: Vec::new(),
            estimate: None,
            depends_on: Vec::new(),
            pinned: false,
            pinned_at: None,
            schema_version: SCHEMA_VERSION,
        }
    }
}

/// Parsed estimate value.
#[derive(Debug, Clone, PartialEq)]
pub enum Estimate {
    Minutes(u32),
    Hours(f32),
    Points(f32),
}

impl Estimate {
    /// Parse an estimate string like "30m", "2h", "3p".
    pub fn parse(s: &str) -> Result<Self, TaskCtlError> {
        let s = s.trim().to_lowercase();
        if s.is_empty() {
            return Err(TaskCtlError::InvalidArgument("Empty estimate".to_string()));
        }

        let (num_str, unit) = s.split_at(s.len() - 1);
        let num: f32 = num_str
            .parse()
            .map_err(|_| TaskCtlError::InvalidArgument(format!("Invalid estimate: {s}")))?;

        match unit {
            "m" => Ok(Self::Minutes(num as u32)),
            "h" => Ok(Self::Hours(num)),
            "p" => Ok(Self::Points(num)),
            _ => Err(TaskCtlError::InvalidArgument(format!(
                "Unknown estimate unit: {unit} (expected m/h/p)"
            ))),
        }
    }

    /// Convert to hours using the given point-to-hours ratio.
    pub fn to_hours(&self, point_to_hours: f64) -> f64 {
        match self {
            Self::Minutes(m) => f64::from(*m) / 60.0,
            Self::Hours(h) => f64::from(*h),
            Self::Points(p) => f64::from(*p) * point_to_hours,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_task_defaults() {
        let task = Task::new(1, "Test".to_string());
        assert_eq!(task.id, 1);
        assert_eq!(task.title, "Test");
        assert_eq!(task.status, Status::Pending);
        assert!(task.tags.is_empty());
        assert!(task.depends_on.is_empty());
        assert!(!task.pinned);
        assert_eq!(task.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(Estimate::parse("30m").unwrap(), Estimate::Minutes(30));
    }

    #[test]
    fn parse_hours() {
        assert_eq!(Estimate::parse("2h").unwrap(), Estimate::Hours(2.0));
    }

    #[test]
    fn parse_points() {
        assert_eq!(Estimate::parse("3p").unwrap(), Estimate::Points(3.0));
    }

    #[test]
    fn parse_fractional_hours() {
        assert_eq!(Estimate::parse("1.5h").unwrap(), Estimate::Hours(1.5));
    }

    #[test]
    fn parse_invalid() {
        assert!(Estimate::parse("").is_err());
        assert!(Estimate::parse("abc").is_err());
        assert!(Estimate::parse("3x").is_err());
    }

    #[test]
    fn to_hours_conversion() {
        assert!((Estimate::Minutes(30).to_hours(1.0) - 0.5).abs() < f64::EPSILON);
        assert!((Estimate::Hours(2.0).to_hours(1.0) - 2.0).abs() < f64::EPSILON);
        assert!((Estimate::Points(3.0).to_hours(2.0) - 6.0).abs() < f64::EPSILON);
    }
}
