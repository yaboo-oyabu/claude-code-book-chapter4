//! Error types for taskctl.

use thiserror::Error;

/// All error types that can occur in taskctl.
#[derive(Error, Debug)]
pub enum TaskCtlError {
    // Input errors (exit code: 1)
    /// The requested task was not found.
    #[error("Task #{0} does not exist")]
    TaskNotFound(u32),

    /// An invalid argument was provided.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Adding the dependency would create a cycle.
    #[error("Cyclic dependency detected ({0})")]
    CyclicDependency(String),

    /// A task cannot depend on itself.
    #[error("A task cannot depend on itself (#{0})")]
    SelfDependency(u32),

    // Data errors (exit code: 2)
    /// Failed to parse a task file.
    #[error("Failed to parse file: {path}")]
    ParseError {
        path: String,
        #[source]
        source: anyhow::Error,
    },

    /// Schema version mismatch.
    #[error("Schema version mismatch (expected: {expected}, actual: {actual})")]
    #[allow(dead_code)]
    SchemaMismatch { expected: u32, actual: u32 },

    // Lock errors (exit code: 3)
    /// Failed to acquire the lock file.
    #[error("Failed to acquire lock file")]
    LockError(#[source] std::io::Error),

    // Config errors (exit code: 4)
    /// Failed to read or parse the configuration file.
    #[error("Failed to read configuration: {0}")]
    ConfigError(String),

    // IO errors
    /// A generic I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl TaskCtlError {
    /// Returns the process exit code for this error category.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::TaskNotFound(_)
            | Self::InvalidArgument(_)
            | Self::CyclicDependency(_)
            | Self::SelfDependency(_) => 1,

            Self::ParseError { .. } | Self::SchemaMismatch { .. } => 2,

            Self::LockError(_) => 3,

            Self::ConfigError(_) => 4,

            Self::Io(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_input_errors() {
        assert_eq!(TaskCtlError::TaskNotFound(99).exit_code(), 1);
        assert_eq!(TaskCtlError::InvalidArgument("bad".into()).exit_code(), 1);
        assert_eq!(
            TaskCtlError::CyclicDependency("#1 -> #2 -> #1".into()).exit_code(),
            1
        );
        assert_eq!(TaskCtlError::SelfDependency(5).exit_code(), 1);
    }

    #[test]
    fn exit_code_data_errors() {
        let err = TaskCtlError::ParseError {
            path: "test.md".into(),
            source: anyhow::anyhow!("bad yaml"),
        };
        assert_eq!(err.exit_code(), 2);
        assert_eq!(
            TaskCtlError::SchemaMismatch {
                expected: 2,
                actual: 1
            }
            .exit_code(),
            2
        );
    }

    #[test]
    fn exit_code_lock_error() {
        let err = TaskCtlError::LockError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "lock failed",
        ));
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn exit_code_config_error() {
        assert_eq!(TaskCtlError::ConfigError("bad".into()).exit_code(), 4);
    }

    #[test]
    fn exit_code_io_error() {
        let err = TaskCtlError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not found",
        ));
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn display_messages() {
        assert_eq!(
            TaskCtlError::TaskNotFound(42).to_string(),
            "Task #42 does not exist"
        );
        assert_eq!(
            TaskCtlError::SelfDependency(5).to_string(),
            "A task cannot depend on itself (#5)"
        );
    }
}
