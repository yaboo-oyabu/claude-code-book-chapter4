//! `task completions` command.

use crate::cli::args::Cli;
use crate::error::TaskCtlError;
use clap::CommandFactory;
use clap_complete::{generate, Shell};

pub fn run(shell_name: &str) -> Result<String, TaskCtlError> {
    let shell = match shell_name.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        _ => {
            return Err(TaskCtlError::InvalidArgument(format!(
                "Unsupported shell: {shell_name}. Supported: bash, zsh, fish"
            )));
        }
    };

    let mut cmd = Cli::command();
    let mut buf = Vec::new();
    generate(shell, &mut cmd, "task", &mut buf);
    String::from_utf8(buf)
        .map_err(|e| TaskCtlError::InvalidArgument(format!("Failed to generate completions: {e}")))
}
