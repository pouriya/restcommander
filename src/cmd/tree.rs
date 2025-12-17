use super::errors::CommandError;
use crate::cmd::MAX_COMMAND_DIRECTORY_DEPTH;
use serde_derive::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, trace, warn};

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
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub commands: HashMap<String, Command>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandInfo {
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing)]
    pub state: Option<CommandInfoGetState>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandInfoGetState {
    Options(Vec<String>),
    Constant(String),
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
    ) -> Result<HashMap<String, Command>, CommandError> {
        if recursion_count == 0 {
            warn!(
                directory = ?directory,
                hint = format!("Maximum supported depth for sub-directories is {}", MAX_COMMAND_DIRECTORY_DEPTH).as_str(),
                "Skipped sub-directories.",
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
                    if let Some(extension) = entry.extension() {
                        if extension == "yaml" || extension == "yml" {
                            continue;
                        };
                    };
                    warn!(filename = ?entry, "It is not executable and will be discarded.");
                    continue;
                };
                let (command_name, command) =
                    Self::from_filename(root_directory, &entry, &http_base_path.clone())?;
                debug!(
                    command = command_name.as_str(),
                    filename = ?entry,
                    "Detected new command.",
                );
                trace!(
                    command = command_name.as_str(),
                    filename = ?entry,
                    info = ?command,
                );
                commands.insert(command_name, command);
            };
            if entry.is_dir() {
                let command = Command {
                    name: entry.file_name().unwrap().to_str().unwrap().to_string(),
                    file_path: entry.clone(),
                    http_path: http_base_path
                        .clone()
                        .join(entry.strip_prefix(root_directory).unwrap()),
                    info: None,
                    is_directory: true,
                    commands: Command::detect_commands(
                        root_directory,
                        &entry,
                        http_base_path,
                        recursion_count - 1,
                    )?,
                };
                commands.insert(
                    entry.file_name().unwrap().to_str().unwrap().to_string(),
                    command,
                );
            }
        }
        Ok(commands)
    }

    pub fn new(root_directory: &std::path::Path, http_base_path: &std::path::Path) -> Result<Self, CommandError> {
        if !root_directory.is_dir() {
            return Err(CommandError::CommandIsNotDirectory {
                http_path: root_directory.to_path_buf(),
            });
        };
        let mut command = Self {
            name: root_directory
                .file_name()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap()
                .to_string(),
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
    ) -> Result<(String, Self), CommandError> {
        if !filename.is_file() {
            return Err(CommandError::IsNotARegularFile {
                filename: filename.to_path_buf(),
            });
        };
        let name = PathBuf::from(filename.file_name().unwrap())
            .to_str()
            .unwrap()
            .to_string();
        Ok((
            name.clone(),
            Command {
                name,
                file_path: filename.to_path_buf(),
                http_path: http_base_path
                    .to_path_buf()
                    .join(filename.strip_prefix(root_directory).unwrap()),
                info: Some(Command::detect_command_info(&filename.to_path_buf())?),
                is_directory: false,
                commands: HashMap::new(),
            },
        ))
    }

    pub fn detect_command_info(command_filename: &PathBuf) -> Result<CommandInfo, CommandError> {
        let mut info_filename = PathBuf::from(format!(
            "{}.yaml",
            command_filename.clone().to_str().unwrap()
        ));
        if !info_filename.exists() {
            info_filename.set_extension("yml");
        };
        if !info_filename.exists() {
            warn!(command_filename = ?command_filename, "No .yaml or .yml information file found");
            return Ok(CommandInfo {
                description: command_filename
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                version: None,
                state: None,
                support_state: false,
                options: Default::default(),
            });
        };
        if !info_filename.is_file() {
            return Err(CommandError::CommandInfoFileNotIsNotFile {
                filename: command_filename.clone(),
            });
        };
        let info_file_content = fs::read_to_string(info_filename.clone()).map_err(|reason| {
            CommandError::ReadCommandInfoFile {
                filename: command_filename.clone(),
                message: reason,
            }
        })?;
        if info_file_content.trim().is_empty() {
            return Ok(CommandInfo {
                description: command_filename
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                version: None,
                state: None,
                support_state: false,
                options: Default::default(),
            });
        };
        let mut command_info =
            serde_yaml::from_str::<CommandInfo>(&info_file_content).map_err(|reason| {
                CommandError::DecodeCommandInfo {
                    filename: info_filename.clone(),
                    message: reason,
                }
            })?;
        if command_info.state.is_some() {
            command_info.support_state = true;
        }
        let mut check_options = Ok(command_info.clone());
        for (option, definition) in command_info.options {
            if !definition.required && definition.default_value.is_none() {
                check_options = Err(format!(
                    "option {:?} is optional and does not have a default value",
                    option
                ));
                break;
            };
            if let Some(ref default_value) = definition.default_value {
                match (definition.value_type.clone(), default_value) {
                    (CommandOptionInfoValueType::Any, _) => (),
                    (CommandOptionInfoValueType::Enum(_), CommandOptionValue::String(_)) => (),
                    (CommandOptionInfoValueType::String, CommandOptionValue::String(_)) => (),
                    (CommandOptionInfoValueType::Integer, CommandOptionValue::Integer(_)) => (),
                    (CommandOptionInfoValueType::Float, CommandOptionValue::Float(_)) => (),
                    (CommandOptionInfoValueType::Boolean, CommandOptionValue::Bool(_)) => (),
                    _ => {
                        check_options = Err(format!("for option '{}' the default value type should be the same as value type", option));
                        break;
                    }
                }
            };
            if let CommandOptionInfoValueType::Enum(ref list) = definition.value_type {
                if list.is_empty() {
                    check_options = Err(format!(
                        "value of option '{}' is an accepted_value_list which is empty",
                        option
                    ));
                    break;
                };
                if let Some(CommandOptionValue::String(ref value)) = definition.default_value {
                    if !list.contains(value) {
                        check_options = Err(format!("for option '{}' the default value should be in its default value list", option));
                        break;
                    }
                }
            }
        }
        if let Ok(ref command_info) = check_options {
            debug!(command_filename = ?command_filename, info_filename = ?info_filename, "Detected command information.");
            trace!(command_filename = ?command_filename, info_filename = ?info_filename, info = ?command_info);
        };
        check_options.map_err(|reason| CommandError::InvalidCommandInfo {
            command: command_filename.clone(),
            message: reason,
        })
    }
}
