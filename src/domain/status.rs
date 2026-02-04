//! Task status and transitions.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::TaskCtlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pending,
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl Status {
    pub fn from_str_loose(s: &str) -> Result<Self, TaskCtlError> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "in_progress" | "inprogress" | "in-progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            _ => Err(TaskCtlError::InvalidArgument(format!(
                "Unknown status: {s}"
            ))),
        }
    }
}

/// Transition to a target status. Returns Ok(target) on success.
/// Idempotent: transitioning to the same status returns Ok without changes.
pub fn transition(current: Status, target: Status) -> Result<Status, TaskCtlError> {
    if current == target {
        return Ok(target);
    }

    match (current, target) {
        (Status::Pending, Status::InProgress)
        | (Status::Pending, Status::Done)
        | (Status::InProgress, Status::Done)
        | (Status::InProgress, Status::Pending)
        | (Status::Done, Status::Pending) => Ok(target),
        _ => Err(TaskCtlError::InvalidArgument(format!(
            "Cannot transition from {current} to {target}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions() {
        assert_eq!(
            transition(Status::Pending, Status::InProgress).unwrap(),
            Status::InProgress
        );
        assert_eq!(
            transition(Status::Pending, Status::Done).unwrap(),
            Status::Done
        );
        assert_eq!(
            transition(Status::InProgress, Status::Done).unwrap(),
            Status::Done
        );
        assert_eq!(
            transition(Status::InProgress, Status::Pending).unwrap(),
            Status::Pending
        );
        assert_eq!(
            transition(Status::Done, Status::Pending).unwrap(),
            Status::Pending
        );
    }

    #[test]
    fn idempotent_transitions() {
        assert_eq!(
            transition(Status::Pending, Status::Pending).unwrap(),
            Status::Pending
        );
        assert_eq!(
            transition(Status::InProgress, Status::InProgress).unwrap(),
            Status::InProgress
        );
        assert_eq!(
            transition(Status::Done, Status::Done).unwrap(),
            Status::Done
        );
    }

    #[test]
    fn invalid_transition_done_to_in_progress() {
        assert!(transition(Status::Done, Status::InProgress).is_err());
    }

    #[test]
    fn display() {
        assert_eq!(Status::Pending.to_string(), "pending");
        assert_eq!(Status::InProgress.to_string(), "in_progress");
        assert_eq!(Status::Done.to_string(), "done");
    }

    #[test]
    fn from_str_loose_variants() {
        assert_eq!(Status::from_str_loose("pending").unwrap(), Status::Pending);
        assert_eq!(
            Status::from_str_loose("in_progress").unwrap(),
            Status::InProgress
        );
        assert_eq!(
            Status::from_str_loose("in-progress").unwrap(),
            Status::InProgress
        );
        assert_eq!(Status::from_str_loose("done").unwrap(), Status::Done);
        assert!(Status::from_str_loose("unknown").is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&Status::InProgress).unwrap();
        assert_eq!(json, "\"in_progress\"");
        let parsed: Status = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Status::InProgress);
    }
}
