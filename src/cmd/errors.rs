use crate::cmd::CommandInput;
use std::io;
use std::path::PathBuf;
use serde_yaml;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("could not read directory {directory:?}")]
    ReadDirectory {
        directory: PathBuf,
        source: io::Error,
    },
    #[error("could not read directory entry inside {directory:?}")]
    ReadDirectoryEntry {
        directory: PathBuf,
        source: io::Error,
    },
    #[error("{filename:?} is not a regular file")]
    IsNotARegularFile { filename: PathBuf },
    #[error("could not encode command input {command_input:?} to JSON")]
    EncodeInputToJSON {
        command_input: CommandInput,
        source: serde_json::Error,
    },
    #[error("could not create new process for command {command:?}")]
    CreateCommandProcess { command: PathBuf, source: io::Error },
    #[error("could write {data:?} to command {command:?} stdin")]
    WriteToCommandStdin {
        data: String,
        command: PathBuf,
        source: io::Error,
    },
    #[error("could not wait for command {command:?} process")]
    WaitForCommandProcess { command: PathBuf, source: io::Error },
    #[error("could not read command {command:?} stdout")]
    ReadCommandStdout { command: PathBuf, source: io::Error },
    #[error("could not read command {command:?} stderr")]
    ReadCommandStderr { command: PathBuf, source: io::Error },
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
    #[error("command information for command {command:?} is invalid: {reason:?}")]
    InvalidCommandInfo { command: PathBuf, reason: String },
    #[error("Command info file {filename:?} is not a regular file")]
    CommandInfoFileNotIsNotFile{
        filename: PathBuf
    },
    #[error("Could not read command info file {filename:?}")]
    ReadCommandInfoFile{
        filename: PathBuf,
        source: io::Error
    },
}
