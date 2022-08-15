use super::errors::CommandError;
use crate::cmd::MAX_COMMAND_DIRECTORY_DEPTH;
use crate::http::API_RUN_BASE_PATH;
use log::{debug, trace, warn};
use serde_derive::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub name: String,
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(skip)]
    pub info_file_path: PathBuf,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_state: Option<String>,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandOptionInfoValueType {
    Any,
    Bool,
    Integer(CommandOptionInfoValueTypeInteger),
    Float(CommandOptionInfoValueTypeFloat),
    String(CommandOptionInfoValueTypeString),
    AcceptedValueList(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOptionValues {
    pub parameters: HashMap<String, CommandOptionValue>,
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
pub struct CommandOptionInfoValueTypeString {
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOptionInfoValueTypeInteger {
    pub min_size: Option<i64>,
    pub max_size: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandOptionInfoValueTypeFloat {
    pub min_size: Option<f64>,
    pub max_size: Option<f64>,
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

    pub fn replace(&mut self, other: Self) {
        self.commands = other.commands;
        self.http_path = other.http_path;
        self.info = other.info;
        self.is_directory = other.is_directory;
        self.name = other.name;
        self.file_path = other.file_path;
    }

    pub fn detect_commands(
        root_directory: &PathBuf,
        directory: &PathBuf,
        http_base_path: &PathBuf,
        recursion_count: usize,
    ) -> Result<HashMap<String, Command>, CommandError> {
        if recursion_count == 0 {
            warn!(
                "maximum depth of command directories is {}. skipped directory {:?} with depth {}",
                MAX_COMMAND_DIRECTORY_DEPTH,
                &directory,
                MAX_COMMAND_DIRECTORY_DEPTH + 1
            );
            return Ok(HashMap::new());
        };
        let read_directory =
            directory
                .read_dir()
                .map_err(|reason| CommandError::ReadDirectory {
                    source: reason,
                    directory: directory.clone(),
                })?;
        let mut commands = HashMap::new();
        for entry in read_directory {
            let entry = entry
                .map_err(|reason| CommandError::ReadDirectoryEntry {
                    directory: directory.clone(),
                    source: reason,
                })?
                .path();
            if entry.is_file() {
                if !is_executable::is_executable(entry.clone()) {
                    if let Some(extension) = entry.extension() {
                        if extension == "yaml" || extension == "yml" {
                            continue;
                        };
                    };
                    warn!("{:?} is not executable and discarded", entry);
                    continue;
                };
                let (command_name, command) =
                    Self::from_filename(root_directory, &entry, &http_base_path.clone())?;
                debug!(
                    "created command {} from command filename {:?}: {:#?}",
                    command_name, &entry, command
                );
                commands.insert(command_name, command);
            };
            if entry.is_dir() {
                let command = Command {
                    name: entry.file_name().unwrap().to_str().unwrap().to_string(),
                    file_path: entry.clone(),
                    info_file_path: Default::default(),
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

    pub fn new(root_directory: &PathBuf, http_base_path: &PathBuf) -> Result<Self, CommandError> {
        if !root_directory.is_dir() {
            return Err(CommandError::CommandIsNotDirectory {
                http_path: root_directory.clone(),
            });
        };
        let mut command = Self {
            name: root_directory
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            file_path: root_directory.clone(),
            info_file_path: Default::default(),
            http_path: http_base_path.clone(),
            info: None,
            is_directory: true,
            commands: HashMap::new(),
        };
        command.reload()?;
        Ok(command)
    }

    pub fn from_filename(
        root_directory: &PathBuf,
        filename: &PathBuf,
        http_base_path: &PathBuf,
    ) -> Result<(String, Self), CommandError> {
        if !filename.is_file() {
            return Err(CommandError::IsNotARegularFile {
                filename: filename.clone(),
            });
        };
        let name = PathBuf::from(filename.file_name().unwrap())
            .to_str()
            .unwrap()
            .to_string();
        return Ok((
            name.clone(),
            Command {
                name,
                file_path: filename.clone(),
                info_file_path: Default::default(),
                http_path: http_base_path
                    .clone()
                    .join(filename.strip_prefix(root_directory).unwrap()),
                info: Some(Command::detect_command_info(filename)?),
                is_directory: false,
                commands: HashMap::new(),
            },
        ));
    }

    pub fn detect_command_info(command_filename: &PathBuf) -> Result<CommandInfo, CommandError> {
        let mut info_filename = command_filename.clone();
        info_filename.set_extension("yaml");
        if !info_filename.exists() {
            info_filename.set_extension("yml");
        };
        if !info_filename.exists() {
            warn!("No .yaml or .yml info file found for {:?}", info_filename);
            return Ok(CommandInfo {
                description: command_filename
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                version: None,
                current_state: None,
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
                source: reason,
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
                current_state: None,
                options: Default::default(),
            });
        };
        let command_info =
            serde_yaml::from_str::<CommandInfo>(&info_file_content).map_err(|reason| {
                CommandError::DecodeCommandInfo {
                    filename: info_filename.clone(),
                    message: reason,
                }
            })?;
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
                    (
                        CommandOptionInfoValueType::AcceptedValueList(_),
                        CommandOptionValue::String(_),
                    ) => (),
                    (CommandOptionInfoValueType::String(_), CommandOptionValue::String(_)) => (),
                    (CommandOptionInfoValueType::Integer(_), CommandOptionValue::Integer(_)) => (),
                    (CommandOptionInfoValueType::Float(_), CommandOptionValue::Float(_)) => (),
                    (CommandOptionInfoValueType::Bool, CommandOptionValue::Bool(_)) => (),
                    _ => {
                        check_options = Err(format!("for option '{}' the default value type should be the same as value type", option));
                        break;
                    }
                }
            };
            match definition.value_type {
                CommandOptionInfoValueType::AcceptedValueList(ref list) => {
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
                _ => (),
            }
        }
        if let Ok(ref command_info) = check_options {
            trace!("{:?} -> {:#?}", command_filename.clone(), command_info);
        };
        check_options.map_err(|reason| CommandError::InvalidCommandInfo {
            command: command_filename.clone(),
            reason: reason,
        })
    }

    pub fn replace_http_base_path(&mut self, base_path: &PathBuf) {
        self.do_replace_http_base_path(&self.http_path.clone(), base_path);
    }

    fn do_replace_http_base_path(&mut self, old_base_path: &PathBuf, new_base_path: &PathBuf) {
        let new_http_path = new_base_path
            .clone()
            .join(PathBuf::from(API_RUN_BASE_PATH).strip_prefix("/").unwrap())
            .join(self.http_path.strip_prefix(old_base_path.clone()).unwrap());
        self.http_path = new_http_path;
        if self.is_directory {
            for (_, command) in self.commands.iter_mut() {
                command.do_replace_http_base_path(old_base_path, new_base_path)
            }
        };
    }
}

impl Default for CommandOptionInfoValueType {
    fn default() -> Self {
        Self::Any
    }
}
