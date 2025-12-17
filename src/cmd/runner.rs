use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::{process, process::Stdio};
use tracing::{debug, error, info, trace, warn};

use super::errors::CommandError;
pub use crate::cmd::tree::CommandOptionValue;

pub type CommandOptionsValue = HashMap<String, CommandOptionValue>;

#[derive(Clone, Debug)]
pub struct CommandOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub decoded_stdout: Result<serde_json::Value, String>,
    pub stats: CommandStats,
    pub instruction_list: Vec<CommandInstruction>,
}

impl CommandOutput {
    pub fn new() -> Self {
        Self {
            exit_code: 0,
            stdout: "".to_string(),
            decoded_stdout: Ok(serde_json::Value::String(String::new())),
            stats: CommandStats::new(),
            instruction_list: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandStats {
    pub duration: CommandStatsDuration,
    pub size: CommandStatsSize,
}

impl CommandStats {
    pub fn new() -> Self {
        Self {
            duration: CommandStatsDuration::new(),
            size: CommandStatsSize::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandStatsDuration {
    pub total: u64,
    pub start_process: u64,
    pub write_to_stdin: u64,
    pub logging: u64,
}

impl CommandStatsDuration {
    pub fn new() -> Self {
        Self {
            total: 0,
            start_process: 0,
            write_to_stdin: 0,
            logging: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandStatsSize {
    pub stdin: usize,
    pub stdout: usize,
    pub stderr: usize,
}

impl CommandStatsSize {
    pub fn new() -> Self {
        Self {
            stdin: 0,
            stdout: 0,
            stderr: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandInput {
    #[serde(default)]
    pub options: CommandOptionsValue,
    #[serde(default)]
    pub statistics: bool,
}

#[derive(Clone, Debug)]
pub enum CommandInstruction {
    Reload,
    Report(String),
}

impl FromStr for CommandInstruction {
    type Err = ErrorKind;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("REPORT ") && s.len() > 7 {
            return Ok(Self::Report(s[7..].to_string()));
        }
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
    _capture_stderr: bool,
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
            command = ?command,
            options = ?option_list,
            "Attempt to run command",
        )
    } else {
        debug!(
            command = ?command,
            "Attempt to run command",
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
            .inspect(|_| {
                trace!(command = ?command, options = ?input_string.clone().unwrap(), "Wrote options to process stdin");
            })
            .inspect_err(|error| {
                warn!(
                    command = ?command,
                    error = error.to_string().as_str(),
                    "Could not write options to process stdin"
                );
            }).unwrap_or_default();
        child_stdin.flush().unwrap_or_default();
        write_to_stdin_duration = start_write_to_stdin.elapsed().as_micros();
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
                command = ?command,
                message = line.replacen("INFO", "", 1).trim_start()
            )
        } else if line.starts_with("ERROR") {
            error!(
                command = ?command,
                message = line.replacen("ERROR", "", 1).trim_start()
            )
        } else if line.starts_with("DEBUG") {
            debug!(
                command = ?command,
                message = line.replacen("DEBUG", "", 1).trim_start()
            )
        } else if line.starts_with("WARNING") || line.starts_with("WARN") {
            warn!(
                command = ?command,
                message = line.replacen("WARNING", "", 1)
                    .replacen("WARN", "", 1)
                    .trim_start()
            )
        } else if line.starts_with("TRACE") {
            trace!(
                command = ?command,
                message = line.replacen("TRACE", "", 1).trim_start()
            )
        } else {
            match line.parse::<CommandInstruction>() {
                Ok(instruction) => instruction_list.push(instruction),
                _ => child_log_buffer += line,
            };
        };
    }
    if !child_log_buffer.is_empty() {
        error!(
            command = ?command,
            stderr = ?child_log_buffer
        );
    };
    let logging_duration = start_logging.elapsed().as_micros();
    trace!(
        stdin = input_string.clone().unwrap_or_default().as_str(),
        stdout = child_stdout.as_str(),
        stderr = child_stderr.as_str(),
        exit_status = child_exit_code,
        command = ?command,
    );
    info!(command = ?command, exit_status = child_exit_code);
    let decoded_stdout: Result<serde_json::Value, String> =
        match serde_json::from_str(&child_stdout) {
            Ok(value) => Ok(value),
            Err(reason) => Err(reason.to_string()),
        };
    Ok(CommandOutput {
        instruction_list,
        decoded_stdout,
        stdout: child_stdout,
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
