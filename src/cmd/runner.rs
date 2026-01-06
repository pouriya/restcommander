use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;
use std::{process, process::Stdio};

use super::errors::CommandError;
pub use crate::cmd::tree::CommandOptionValue;

pub type CommandOptionsValue = HashMap<String, CommandOptionValue>;

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub stats: CommandStats,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandStats {
    pub duration: CommandStatsDuration,
    pub size: CommandStatsSize,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandStatsDuration {
    pub total: u64,
    pub start_process: u64,
    pub write_to_stdin: u64,
    pub logging: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandStatsSize {
    pub stdin: usize,
    pub stdout: usize,
    pub stderr: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandInput {
    #[serde(default)]
    pub options: CommandOptionsValue,
    #[serde(default)]
    pub statistics: bool,
}

pub fn run_command(
    command: &PathBuf,
    option_list: Vec<String>,
    input: Option<&CommandInput>,
    parse_stderr_logs: bool,
    env_map: HashMap<String, String>,
) -> Result<CommandOutput, CommandError> {
    let mut input_string = None;
    if input.is_some() {
        input_string = Some(
            serde_json::to_string(&input.unwrap().options).map_err(|reason| {
                CommandError::EncodeInputToJSON {
                    message: reason,
                    command_input: input.unwrap().clone(),
                }
            })?,
        );
        tracing::debug!(
            msg = "Attempting to run command with options",
            command = ?command,
            options = ?option_list,
        )
    } else {
        tracing::debug!(
            msg = "Attempting to run command",
            command = ?command,
        )
    }
    let start = Instant::now();
    let start_process = Instant::now();
    let mut child = process::Command::new(command.clone())
        .args(option_list.clone())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .envs(env_map)
        .spawn()
        .map_err(|reason| CommandError::CreateCommandProcess {
            message: reason,
            command: command.clone(),
        })?;
    let process_duration = start_process.elapsed().as_micros();

    let mut write_to_stdin_duration = 0;
    let start_write_to_stdin = Instant::now();
    if let Some(ref input_str) = input_string {
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin
            .write_all(input_str.as_bytes())
            .map_err(|reason| CommandError::WriteToCommandStdin {
                command: command.clone(),
                message: reason,
            })?;
        child_stdin
            .flush()
            .map_err(|reason| CommandError::WriteToCommandStdin {
                command: command.clone(),
                message: reason,
            })?;
        tracing::trace!(
            msg = "Wrote command options to process stdin",
            command = ?command,
            options = ?input_str,
        );
        write_to_stdin_duration = start_write_to_stdin.elapsed().as_micros();
    };

    let wait_for_child = child
        .wait()
        .map_err(|reason| CommandError::WaitForCommandProcess {
            message: reason,
            command: command.clone(),
        })?;
    let command_duration = start.elapsed().as_micros();
    let child_exit_code = wait_for_child.code().unwrap_or({
        // Process was terminated by a signal
        // Use standard exit codes: 130 for SIGINT, 143 for SIGTERM, 128 for unknown signal
        128
    });

    let mut child_stdout = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut child_stdout)
        .map_err(|reason| CommandError::ReadCommandStdout {
            message: reason,
            command: command.clone(),
        })?;
    let stdout_size = child_stdout.len();
    child_stdout = child_stdout.trim_end().to_string();

    let mut child_stderr = String::new();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut child_stderr)
        .map_err(|reason| CommandError::ReadCommandStderr {
            message: reason,
            command: command.clone(),
        })?;
    let stderr_size = child_stderr.len();
    let start_logging = Instant::now();
    let raw_stderr = child_stderr.clone();
    child_stderr = child_stderr.trim_end().to_string();

    let logging_duration = if parse_stderr_logs {
        let mut child_log_buffer = String::new();
        for line in child_stderr.lines() {
            if line.starts_with("INFO") {
                tracing::info!(
                    msg = line.replacen("INFO", "", 1).trim_start(),
                    command = ?command,
                )
            } else if line.starts_with("ERROR") {
                tracing::error!(
                    msg = line.replacen("ERROR", "", 1).trim_start(),
                    command = ?command,
                )
            } else if line.starts_with("DEBUG") {
                tracing::debug!(
                    msg = line.replacen("DEBUG", "", 1).trim_start(),
                    command = ?command,
                )
            } else if line.starts_with("WARNING") || line.starts_with("WARN") {
                tracing::warn!(
                    msg = line.replacen("WARNING", "", 1)
                        .replacen("WARN", "", 1)
                        .trim_start(),
                    command = ?command,
                )
            } else if line.starts_with("TRACE") {
                tracing::trace!(
                    msg = line.replacen("TRACE", "", 1).trim_start(),
                    command = ?command,
                )
            } else {
                child_log_buffer += line;
            };
        }
        if !child_log_buffer.is_empty() {
            tracing::error!(
                msg = "Unparsed stderr output from command",
                command = ?command,
                stderr = ?child_log_buffer,
            );
        };
        start_logging.elapsed().as_micros()
    } else {
        0
    };
    tracing::trace!(
        msg = "Command execution completed",
        command = ?command,
        stdin = input_string.clone().unwrap_or_default().as_str(),
        stdout = child_stdout.as_str(),
        stderr = child_stderr.as_str(),
        exit_status = child_exit_code,
    );
    tracing::info!(
        msg = "Command finished execution",
        command = ?command,
        exit_status = child_exit_code,
    );
    Ok(CommandOutput {
        stdout: child_stdout,
        stderr: raw_stderr.trim_end().to_string(),
        exit_code: child_exit_code,
        stats: CommandStats {
            duration: CommandStatsDuration {
                total: command_duration as u64,
                start_process: process_duration as u64,
                write_to_stdin: write_to_stdin_duration as u64,
                logging: logging_duration as u64,
            },
            size: CommandStatsSize {
                stdin: input_string.unwrap_or(String::new()).len(),
                stdout: stdout_size,
                stderr: stderr_size,
            },
        },
    })
}
