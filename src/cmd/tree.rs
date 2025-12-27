use super::errors::CommandError;
use crate::cmd::MAX_COMMAND_DIRECTORY_DEPTH;
use serde::ser::{SerializeMap, Serializer};
use serde::Serialize as SerializeTrait;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;

// Cloneable error representation for storage in HashMap
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandErrorInfo {
    code: i32,
    message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(skip_deserializing)]
    pub http_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<CommandInfo>,
    #[serde(default)]
    pub is_directory: bool,
    #[serde(
        skip_serializing_if = "HashMap::is_empty",
        serialize_with = "serialize_commands_map",
        default
    )]
    pub commands: HashMap<String, Result<Command, CommandErrorInfo>>,
}

fn serialize_commands_map<S>(
    commands: &HashMap<String, Result<Command, CommandErrorInfo>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeMap;
    let mut map = serializer.serialize_map(Some(commands.len()))?;
    for (k, v) in commands {
        map.serialize_entry(k, &CommandResultWrapper(v))?;
    }
    map.end()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_deserializing)]
    pub support_state: bool,
    #[serde(default)]
    pub options: HashMap<String, CommandOptionInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOptionInfo {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub value_type: CommandOptionInfoValueType,
    #[serde(default)]
    pub default_value: Option<CommandOptionValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<CommandOptionInfoValueSize>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandOptionInfoValueType {
    #[default]
    Any,
    Boolean,
    Integer,
    Float,
    String,
    Enum(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandOptionValue {
    None,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOptionInfoValueSize {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl Command {
    pub fn reload(&mut self) -> Result<(), CommandError> {
        self.commands = Self::detect_commands(
            &self.file_path,
            &self.file_path,
            &self.http_path,
            MAX_COMMAND_DIRECTORY_DEPTH,
        )?;
        Ok(())
    }

    pub fn detect_commands(
        root_directory: &PathBuf,
        directory: &PathBuf,
        http_base_path: &PathBuf,
        recursion_count: usize,
    ) -> Result<HashMap<String, Result<Command, CommandErrorInfo>>, CommandError> {
        if recursion_count == 0 {
            tracing::warn!(
                msg = "Skipped sub-directories: maximum depth reached",
                directory = ?directory,
                max_depth = MAX_COMMAND_DIRECTORY_DEPTH,
            );
            return Ok(HashMap::new());
        };
        let read_directory =
            directory
                .read_dir()
                .map_err(|reason| CommandError::ReadDirectory {
                    message: reason,
                    directory: directory.clone(),
                })?;
        let mut commands = HashMap::new();
        for entry in read_directory {
            let entry = entry
                .map_err(|reason| CommandError::ReadDirectoryEntry {
                    directory: directory.clone(),
                    message: reason,
                })?
                .path();
            if entry.is_file() {
                if !is_executable::is_executable(entry.clone()) {
                    tracing::warn!(
                        msg = "File is not executable and will be discarded",
                        filename = ?entry,
                    );
                    continue;
                };
                let (command_name, result_command) =
                    Self::from_filename(root_directory, &entry, &http_base_path.clone())?;
                match &result_command {
                    Ok(cmd) => {
                        tracing::debug!(
                            msg = "Detected new command",
                            command = command_name.as_str(),
                            filename = ?entry,
                        );
                        tracing::trace!(
                            msg = "Command information details",
                            command = command_name.as_str(),
                            filename = ?entry,
                            info = ?cmd.info,
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            msg = "Failed to detect command information",
                            command = command_name.as_str(),
                            filename = ?entry,
                            error = ?e,
                        );
                    }
                }
                commands.insert(command_name, result_command);
            };
            if entry.is_dir() {
                let sub_commands = Command::detect_commands(
                    root_directory,
                    &entry,
                    http_base_path,
                    recursion_count - 1,
                )?;
                let file_name = entry
                    .file_name()
                    .ok_or_else(|| CommandError::NoFileName {
                        path: entry.clone(),
                    })?
                    .to_str()
                    .ok_or_else(|| CommandError::InvalidPathUtf8 {
                        path: entry.clone(),
                    })?
                    .to_string();
                let http_path =
                    http_base_path
                        .clone()
                        .join(entry.strip_prefix(root_directory).map_err(|_| {
                            CommandError::StripPrefixFailed {
                                path: entry.clone(),
                                prefix: root_directory.clone(),
                            }
                        })?);
                let command = Command {
                    name: file_name.clone(),
                    file_path: entry.clone(),
                    http_path,
                    info: None,
                    is_directory: true,
                    commands: sub_commands,
                };
                commands.insert(file_name, Ok(command));
            }
        }
        Ok(commands)
    }

    pub fn new(
        root_directory: &std::path::Path,
        http_base_path: &std::path::Path,
    ) -> Result<Self, CommandError> {
        if !root_directory.is_dir() {
            return Err(CommandError::CommandIsNotDirectory {
                http_path: root_directory.to_path_buf(),
            });
        };
        let name = root_directory
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .ok_or_else(|| CommandError::InvalidPathUtf8 {
                path: root_directory.to_path_buf(),
            })?
            .to_string();
        let mut command = Self {
            name,
            file_path: root_directory.to_path_buf(),
            http_path: http_base_path.to_path_buf(),
            info: None,
            is_directory: true,
            commands: HashMap::new(),
        };
        command.reload()?;
        Ok(command)
    }

    pub fn from_filename(
        root_directory: &std::path::Path,
        filename: &std::path::Path,
        http_base_path: &std::path::Path,
    ) -> Result<(String, Result<Self, CommandErrorInfo>), CommandError> {
        if !filename.is_file() {
            return Err(CommandError::IsNotARegularFile {
                filename: filename.to_path_buf(),
            });
        };
        let name = filename
            .file_name()
            .ok_or_else(|| CommandError::NoFileName {
                path: filename.to_path_buf(),
            })?
            .to_str()
            .ok_or_else(|| CommandError::InvalidPathUtf8 {
                path: filename.to_path_buf(),
            })?
            .to_string();
        let http_path =
            http_base_path
                .to_path_buf()
                .join(filename.strip_prefix(root_directory).map_err(|_| {
                    CommandError::StripPrefixFailed {
                        path: filename.to_path_buf(),
                        prefix: root_directory.to_path_buf(),
                    }
                })?);

        match Command::detect_command_info(&filename.to_path_buf()) {
            Ok(info) => Ok((
                name.clone(),
                Ok(Command {
                    name,
                    file_path: filename.to_path_buf(),
                    http_path,
                    info: Some(info),
                    is_directory: false,
                    commands: HashMap::new(),
                }),
            )),
            Err(error) => {
                // Convert CommandError to CommandErrorInfo for storage
                let error_info = CommandErrorInfo {
                    code: error.error_code(),
                    message: error.to_string(),
                };
                Ok((name.clone(), Err(error_info)))
            }
        }
    }

    pub fn detect_command_info(command_filename: &PathBuf) -> Result<CommandInfo, CommandError> {
        use crate::cmd::runner;

        let name = command_filename
            .file_name()
            .ok_or_else(|| CommandError::NoFileName {
                path: command_filename.clone(),
            })?
            .to_str()
            .ok_or_else(|| CommandError::InvalidPathUtf8 {
                path: command_filename.clone(),
            })?
            .to_string();

        // Execute script with --help flag (parse_stderr_logs=false since stderr contains JSON)
        let output = runner::run_command(
            command_filename,
            vec!["--help".to_string()],
            None,
            false, // Don't parse stderr logs for --help, it contains JSON
            HashMap::new(),
        )?;

        // Check exit code - must be 0
        if output.exit_code != 0 {
            // We need http_path for the error, but we don't have it here
            // This will be set properly in from_filename
            return Err(CommandError::CommandHelpFailed {
                file_path: command_filename.clone(),
                http_path: PathBuf::new(), // Will be set properly in from_filename
                exit_code: output.exit_code,
                stdout: output.stdout.clone(),
                stderr: output.stderr.clone(),
                name: name.clone(),
            });
        }

        // Parse JSON from stdout
        let help_json: serde_json::Value =
            serde_json::from_str(&output.stdout).map_err(|e| CommandError::InvalidCommandInfo {
                command: command_filename.clone(),
                message: format!("could not parse --help stdout as JSON: {}", e),
            })?;

        // Extract fields with defaults
        let title = help_json
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let description = help_json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());
        let version = help_json
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let support_state = help_json
            .get("state")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Parse JSON from stderr (options definition)
        // For --help, stderr should be pure JSON (no log filtering)
        let options: HashMap<String, CommandOptionInfo> = if output.stderr.trim().is_empty() {
            HashMap::new()
        } else {
            serde_json::from_str(&output.stderr).map_err(|e| CommandError::InvalidCommandInfo {
                command: command_filename.clone(),
                message: format!("could not parse --help stderr as JSON for options: {}", e),
            })?
        };

        // Validate options structure
        for (option_name, option_info) in &options {
            if !option_info.required && option_info.default_value.is_none() {
                return Err(CommandError::InvalidCommandInfo {
                    command: command_filename.clone(),
                    message: format!(
                        "option {:?} is optional and does not have a default value",
                        option_name
                    ),
                });
            }
            if let Some(ref default_value) = option_info.default_value {
                match (&option_info.value_type, default_value) {
                    (CommandOptionInfoValueType::Any, _) => (),
                    (CommandOptionInfoValueType::Enum(_), CommandOptionValue::String(_)) => (),
                    (CommandOptionInfoValueType::String, CommandOptionValue::String(_)) => (),
                    (CommandOptionInfoValueType::Integer, CommandOptionValue::Integer(_)) => (),
                    (CommandOptionInfoValueType::Float, CommandOptionValue::Float(_)) => (),
                    (CommandOptionInfoValueType::Boolean, CommandOptionValue::Bool(_)) => (),
                    _ => {
                        return Err(CommandError::InvalidCommandInfo {
                            command: command_filename.clone(),
                            message: format!("for option '{}' the default value type should be the same as value type", option_name),
                        });
                    }
                }
            }
            if let CommandOptionInfoValueType::Enum(ref list) = option_info.value_type {
                if list.is_empty() {
                    return Err(CommandError::InvalidCommandInfo {
                        command: command_filename.clone(),
                        message: format!(
                            "value of option '{}' is an accepted_value_list which is empty",
                            option_name
                        ),
                    });
                }
                if let Some(CommandOptionValue::String(ref value)) = option_info.default_value {
                    if !list.contains(value) {
                        return Err(CommandError::InvalidCommandInfo {
                            command: command_filename.clone(),
                            message: format!("for option '{}' the default value should be in its default value list", option_name),
                        });
                    }
                }
            }
        }

        tracing::debug!(
            msg = "Detected command information from --help output",
            command_filename = ?command_filename,
        );
        tracing::trace!(
            msg = "Command information details",
            command_filename = ?command_filename,
            description = ?description,
            support_state = ?support_state,
        );

        Ok(CommandInfo {
            title,
            description,
            version,
            support_state,
            options,
        })
    }
}

// Wrapper type for serializing Result<Command, CommandErrorInfo>
struct CommandResultWrapper<'a>(&'a Result<Command, CommandErrorInfo>);

impl<'a> SerializeTrait for CommandResultWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            Ok(command) => {
                let mut map = serializer.serialize_map(None)?;
                if command.is_directory {
                    map.serialize_entry("type", "directory")?;
                } else {
                    map.serialize_entry("type", "command")?;
                }
                map.serialize_entry("name", &command.name)?;
                map.serialize_entry("http_path", &command.http_path)?;
                map.serialize_entry("is_directory", &command.is_directory)?;
                if let Some(ref info) = command.info {
                    map.serialize_entry("info", info)?;
                }
                if !command.commands.is_empty() {
                    map.serialize_entry("commands", &command.commands)?;
                }
                map.end()
            }
            Err(error) => {
                let mut map = serializer.serialize_map(None)?;
                map.serialize_entry("type", "error")?;
                map.serialize_entry("code", &error.code)?;
                map.serialize_entry("message", &error.message)?;
                map.end()
            }
        }
    }
}
