use log::{debug, error, info, log_enabled, trace, warn, Level};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
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
    pub decoded_stdout: Result<serde_json::Value, String>,
    pub stats: CommandStats,
    pub instruction_list: Vec<CommandInstruction>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandInput {
    #[serde(default)]
    pub options: CommandOptionsValue,
    #[serde(default)]
    pub statistics: bool,
}

impl Default for CommandInput {
    fn default() -> Self {
        Self {
            options: Default::default(),
            statistics: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CommandInstruction {
    Reload,
}

impl FromStr for CommandInstruction {
    type Err = ErrorKind;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "reload" => Ok(Self::Reload),
            _ => Err(Self::Err::Unsupported),
        }
    }
}

pub fn run_command(
    command: &PathBuf,
    option_list: Vec<String>,
    input: Option<&CommandInput>,
    capture_stderr: bool,
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
        debug!(
            "attempt to run command {:?} with options {:?} and input {:?}",
            command,
            option_list,
            input_string.clone().unwrap()
        )
    } else {
        debug!(
            "attempt to run command {:?} with options {:?} and without input",
            command, option_list
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
    if input_string.is_some() {
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin
            .write_all(input_string.clone().unwrap().as_bytes())
            .map_err(|reason| CommandError::WriteToCommandStdin {
                message: reason,
                data: input_string.clone().unwrap().clone(),
                command: command.clone(),
            })?;
        child_stdin.flush().unwrap();
        write_to_stdin_duration = start_write_to_stdin.elapsed().as_micros();
        debug!("wrote input to child's stdin");
    };

    let wait_for_child = child
        .wait()
        .map_err(|reason| CommandError::WaitForCommandProcess {
            message: reason,
            command: command.clone(),
        })?;
    let command_duration = start.elapsed().as_micros();
    let child_exit_code = wait_for_child.code().unwrap();

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
    child_stderr = child_stderr.trim_end().to_string();
    let mut child_log_buffer = String::new();
    let mut instruction_list = Vec::new();
    for line in child_stderr.lines() {
        if line.starts_with("INFO") {
            info!(
                "{:?} -> {}",
                command,
                line.replacen("INFO", "", 1).trim_start()
            )
        } else if line.starts_with("ERROR") {
            error!(
                "{:?} -> {}",
                command,
                line.replacen("ERROR", "", 1).trim_start()
            )
        } else if line.starts_with("DEBUG") {
            debug!(
                "{:?} -> {}",
                command,
                line.replacen("DEBUG", "", 1).trim_start()
            )
        } else if line.starts_with("WARNING") || line.starts_with("WARN") {
            warn!(
                "{:?} -> {}",
                command,
                line.replacen("WARNING", "", 1)
                    .replacen("WARN", "", 1)
                    .trim_start()
            )
        } else if line.starts_with("TRACE") {
            trace!(
                "{:?} -> {}",
                command,
                line.replacen("TRACE", "", 1).trim_start()
            )
        } else {
            match line.parse::<CommandInstruction>() {
                Ok(instruction) => instruction_list.push(instruction),
                _ => child_log_buffer += line,
            };
        };
    }
    if !child_log_buffer.is_empty() {
        error!("{:?} stderr -> {}", command, child_log_buffer)
    };
    let logging_duration = start_logging.elapsed().as_micros();

    if log_enabled!(Level::Debug) || log_enabled!(Level::Trace) {
        let mut log_text = format!("command {:?} statistics:", command);
        if !option_list.is_empty() {
            log_text += format!("\noptions: {:?}", option_list).as_str();
        };
        if input_string.clone().is_some() {
            log_text += format!("\nstdin data: {}", input_string.clone().unwrap()).as_str()
        };
        if !child_stdout.is_empty() {
            log_text += format!("\nstdout data: {}", child_stdout).as_str()
        };
        if !child_stderr.is_empty() {
            log_text += format!("\nstderr data: {}", child_stderr).as_str()
        };
        debug!("{}", log_text);
    } else {
        info!(
            "command {:?} exited with {} exit-code",
            command, child_exit_code
        );
    }
    let decoded_stdout: Result<serde_json::Value, String> =
        match serde_json::from_str(&child_stdout) {
            Ok(value) => Ok(value),
            Err(reason) => Err(reason.to_string()),
        };
    if !capture_stderr {
        child_stderr = String::new()
    };
    Ok(CommandOutput {
        stdout: child_stdout,
        stderr: child_stderr,
        exit_code: child_exit_code,
        decoded_stdout: decoded_stdout,
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
        instruction_list: instruction_list,
    })
}
