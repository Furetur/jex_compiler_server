use crate::run_command::CommandError::{StdioDecodingFailed, UnsuccessfulExitCode};
use std::io::Error;
use std::str::from_utf8;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub enum CommandError {
    /// The command failed to run
    RunFailure(Error),
    /// The command was executed but returned an unsuccessful exit code.
    /// The passed [String] is stderr
    UnsuccessfulExitCode(String),
    /// The command ran successfully but stdout could not be parsed as a utf-8 string
    StdioDecodingFailed(String),
    /// Failed to execute on time
    Timeout,
}

/// Runs command and returns its output.
/// Returns stdout if the command exited successfully, otherwise returns CommandError.
pub async fn run_command(
    command: &mut Command,
    timeout_duration: Duration,
) -> Result<String, CommandError> {
    // TODO: this does not kill the process
    let output = timeout(timeout_duration, command.output())
        .await
        .map_err(|_e| CommandError::Timeout)?
        .map_err(|e| CommandError::RunFailure(e))?;
    if output.status.success() {
        from_utf8(&output.stdout)
            .map(|s| s.to_string())
            .map_err(|e| StdioDecodingFailed(e.to_string()))
    } else {
        let stderr = from_utf8(&output.stderr)
            .unwrap_or("Failed to decode stderr as utf-8.")
            .to_string();
        Err(UnsuccessfulExitCode(stderr))
    }
}
