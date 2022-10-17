use crate::cmd::CommandInput;
use serde_yaml;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("could not read directory {directory:?}: {message:?}")]
    ReadDirectory {
        directory: PathBuf,
        message: io::Error,
    },
    #[error("could not read directory entry inside {directory:?}: {message:?}")]
    ReadDirectoryEntry {
        directory: PathBuf,
        message: io::Error,
    },
    #[error("{filename:?} is not a regular file")]
    IsNotARegularFile { filename: PathBuf },
    #[error("could not encode command input {command_input:?} to JSON: {message:?}")]
    EncodeInputToJSON {
        command_input: CommandInput,
        message: serde_json::Error,
    },
    #[error("could not create new process for command {command:?}: {message:?}")]
    CreateCommandProcess {
        command: PathBuf,
        message: io::Error,
    },
    #[error("could write {data:?} to command {command:?} stdin: {message:?}")]
    WriteToCommandStdin {
        data: String,
        command: PathBuf,
        message: io::Error,
    },
    #[error("could not wait for command {command:?} process: {message:?}")]
    WaitForCommandProcess {
        command: PathBuf,
        message: io::Error,
    },
    #[error("could not read command {command:?} stdout: {message:?}")]
    ReadCommandStdout {
        command: PathBuf,
        message: io::Error,
    },
    #[error("could not read command {command:?} stderr: {message:?}")]
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
    #[error("could not find the command {command_name:?} inside directory {http_path:?}")]
    FindCommand {
        command_name: String,
        http_path: PathBuf,
    },
    #[error("command part {http_path:?} is not a directory")]
    CommandIsNotDirectory { http_path: PathBuf },
    #[error("command {http_path:?} is a directory and is not runnable")]
    CommandIsDirectory { http_path: PathBuf },
    #[error("could not decode command information for command {filename:?}: {message:?}")]
    DecodeCommandInfo {
        filename: PathBuf,
        message: serde_yaml::Error,
    },
    #[error("command information for command {command:?} is invalid: {message:?}")]
    InvalidCommandInfo { command: PathBuf, message: String },
    #[error("Command info file {filename:?} is not a regular file")]
    CommandInfoFileNotIsNotFile { filename: PathBuf },
    #[error("Could not read command info file {filename:?}: {message:?}")]
    ReadCommandInfoFile {
        filename: PathBuf,
        message: io::Error,
    },
}
