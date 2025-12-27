use crate::cmd::CommandInput;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("could not read directory {directory:?}: {message}")]
    ReadDirectory {
        directory: PathBuf,
        message: io::Error,
    },
    #[error("could not read directory entry inside {directory:?}: {message}")]
    ReadDirectoryEntry {
        directory: PathBuf,
        message: io::Error,
    },
    #[error("{filename:?} is not a regular file")]
    IsNotARegularFile { filename: PathBuf },
    #[error("could not encode command input {command_input:?} to JSON: {message}")]
    EncodeInputToJSON {
        command_input: CommandInput,
        message: serde_json::Error,
    },
    #[error("could not create new process for command {command:?}: {message}")]
    CreateCommandProcess {
        command: PathBuf,
        message: io::Error,
    },
    // #[error("could write {data:?} to command {command:?} stdin: {message}")]
    // WriteToCommandStdin {
    //     data: String,
    //     command: PathBuf,
    //     message: io::Error,
    // },
    #[error("could not wait for command {command:?} process: {message}")]
    WaitForCommandProcess {
        command: PathBuf,
        message: io::Error,
    },
    #[error("could not read command {command:?} stdout: {message}")]
    ReadCommandStdout {
        command: PathBuf,
        message: io::Error,
    },
    #[error("could not read command {command:?} stderr: {message}")]
    ReadCommandStderr {
        command: PathBuf,
        message: io::Error,
    },
    // #[error("command {command:?} with options {options:?} and stdout {stdout:?} and stderr {stderr:?} exited with exit-code {exit_code:?}")]
    // Crash {
    //     command: PathBuf,
    //     options: String,
    //     stdout: String,
    //     stderr: String,
    //     exit_code: i32,
    // },
    #[error("could not find the command {command_name:?} inside directory {http_path}")]
    FindCommand {
        command_name: String,
        http_path: PathBuf,
    },
    #[error("command part {http_path:?} is not a directory")]
    CommandIsNotDirectory { http_path: PathBuf },
    #[error("command {http_path:?} is a directory and is not runnable")]
    CommandIsDirectory { http_path: PathBuf },
    #[error("command information for command {command:?} is invalid: {message}")]
    InvalidCommandInfo { command: PathBuf, message: String },
    #[error("Could not found command information for command {filename:?}")]
    NoCommandInfo { filename: PathBuf },
    #[error("Could not found command state information for command {filename:?}")]
    NoCommandState { filename: PathBuf },
    #[error("could not load command {name} from {file_path:?} (http path: {http_path:?}) because script --help exited with code {exit_code}: stdout: {stdout}, stderr: {stderr}")]
    CommandHelpFailed {
        file_path: PathBuf,
        http_path: PathBuf,
        exit_code: i32,
        stdout: String,
        stderr: String,
        name: String,
    },
}

impl CommandError {
    pub fn error_code(&self) -> i32 {
        match self {
            CommandError::ReadDirectory { .. } => 3001,
            CommandError::ReadDirectoryEntry { .. } => 3002,
            CommandError::IsNotARegularFile { .. } => 3003,
            CommandError::EncodeInputToJSON { .. } => 3004,
            CommandError::CreateCommandProcess { .. } => 3005,
            CommandError::WaitForCommandProcess { .. } => 3006,
            CommandError::ReadCommandStdout { .. } => 3007,
            CommandError::ReadCommandStderr { .. } => 3008,
            CommandError::FindCommand { .. } => 3009,
            CommandError::CommandIsNotDirectory { .. } => 3010,
            CommandError::CommandIsDirectory { .. } => 3011,
            CommandError::InvalidCommandInfo { .. } => 3013,
            CommandError::NoCommandInfo { .. } => 3014,
            CommandError::NoCommandState { .. } => 3015,
            CommandError::CommandHelpFailed { .. } => 3016,
        }
    }
}
