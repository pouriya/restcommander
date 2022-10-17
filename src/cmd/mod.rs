use crate::cmd::errors::CommandError;
pub use crate::cmd::runner::{CommandInput, CommandOutput, CommandStats};
pub use crate::cmd::tree::{Command, CommandInfoGetState, CommandOptionInfo};
use crate::cmd::tree::{
    CommandOptionInfoValueSize, CommandOptionInfoValueType, CommandOptionValue,
};
use std::collections::HashMap;

pub mod errors;
pub mod runner;
pub mod tree;

static MAX_COMMAND_DIRECTORY_DEPTH: usize = 5;

pub fn search_for_command(
    command_path_list: &Vec<String>,
    command: &Command,
) -> Result<Command, CommandError> {
    let first_element = command_path_list[0].clone();
    if first_element == command.name {
        if command_path_list.len() > 1 {
            if command.is_directory {
                let second_element = command_path_list[1].clone();
                // println!("{:?} - {:?} - {:?}", second_element, command_path_list, command.name);
                if command.commands.contains_key(second_element.as_str()) {
                    return search_for_command(
                        &command_path_list[1..].to_owned(),
                        command.commands.get(second_element.as_str()).unwrap(),
                    );
                };
                return Err(CommandError::FindCommand {
                    command_name: command.name.clone(),
                    http_path: command.http_path.clone(),
                });
            };
            return Err(CommandError::CommandIsNotDirectory {
                http_path: command.http_path.clone(),
            });
        };
        if command.is_directory {
            return Err(CommandError::CommandIsDirectory {
                http_path: command.http_path.clone(),
            });
        };
        return Ok(command.clone());
    };
    return Err(CommandError::FindCommand {
        command_name: command.name.clone(),
        http_path: command.http_path.clone(),
    });
}

pub fn run_command(
    command: &Command,
    input: &CommandInput,
    env_map: HashMap<String, String>,
) -> Result<CommandOutput, CommandError> {
    if command.is_directory {
        return Err(CommandError::CommandIsDirectory {
            http_path: command.http_path.clone(),
        });
    };
    runner::run_command(&command.file_path, Vec::new(), Some(input), true, env_map)
}

pub fn get_state(
    command: &Command,
    env_map: HashMap<String, String>,
) -> Result<CommandOutput, CommandError> {
    if let Some(ref info) = command.info {
        if info.support_state && info.state.is_some() {
            let get_state = info.state.as_ref().unwrap();
            match get_state {
                CommandInfoGetState::Constant(value) => {
                    let mut output = CommandOutput::new();
                    output.stdout = value.clone();
                    output.decoded_stdout = Ok(serde_json::Value::String(value.clone()));
                    Ok(output)
                }
                CommandInfoGetState::Options(options) => {
                    runner::run_command(&command.file_path, options.clone(), None, true, env_map)
                }
            }
        } else {
            Err(CommandError::NoCommandState {
                filename: command.file_path.clone(),
            })
        }
    } else {
        Err(CommandError::NoCommandInfo {
            filename: command.file_path.clone(),
        })
    }
}

pub fn check_input(command: &Command, input: &CommandInput) -> Result<CommandInput, String> {
    let mut new_input = input.clone();
    if command.info.is_none() {
        return Ok(new_input);
    };
    for (option, definition) in &command.info.as_ref().unwrap().options {
        let new_value = if new_input.options.contains_key(option.as_str()) {
            let input_value = new_input.options.get(option.as_str()).unwrap();
            check_definition(
                &option,
                &definition.value_type,
                input_value,
                &definition.size,
            )?
        } else {
            if definition.default_value.is_none() {
                match definition.value_type {
                    CommandOptionInfoValueType::Bool => CommandOptionValue::Bool(false),
                    CommandOptionInfoValueType::Any => CommandOptionValue::None,
                    _ => {
                        // So it is required
                        if definition.required {
                            return Err(format!(
                                "required option {} is not given and has no default value",
                                option
                            ));
                        };
                        unreachable!();
                    }
                }
            } else {
                definition.default_value.clone().unwrap()
            }
        };
        new_input.options.insert(option.clone(), new_value);
    }

    Ok(new_input)
}

fn check_definition(
    option: &str,
    definition: &CommandOptionInfoValueType,
    input: &CommandOptionValue,
    maybe_size: &Option<CommandOptionInfoValueSize>,
) -> Result<CommandOptionValue, String> {
    if let Some(size_definition) = maybe_size {
        check_size(option, input, size_definition)?
    }
    match (definition, input) {
        (CommandOptionInfoValueType::Any, value) => Ok(value.clone()),
        (CommandOptionInfoValueType::Bool, CommandOptionValue::Bool(flag)) => {
            Ok(CommandOptionValue::Bool(flag.clone()))
        }
        (CommandOptionInfoValueType::String, CommandOptionValue::String(value)) => {
            Ok(CommandOptionValue::String(value.clone()))
        }
        (CommandOptionInfoValueType::Integer, CommandOptionValue::Integer(value)) => {
            Ok(CommandOptionValue::Integer(value.clone()))
        }
        (CommandOptionInfoValueType::Float, CommandOptionValue::Float(value)) => {
            Ok(CommandOptionValue::Float(value.clone()))
        }
        (CommandOptionInfoValueType::AcceptedValueList(accepted_value_list), value) => {
            match value {
                CommandOptionValue::String(string_value) => {
                    if accepted_value_list.contains(string_value) {
                        Ok(value.clone())
                    } else {
                        Err(format!(
                            "accepted values for option '{}' are {}",
                            option,
                            accepted_value_list
                                .iter()
                                .map(|x| { format!("'{}'", x) })
                                .collect::<Vec<String>>()
                                .join(", ")
                        ))
                    }
                }
                _ => Err(format!(
                    "option '{}' should be 'String' and one of {}",
                    option,
                    accepted_value_list
                        .iter()
                        .map(|x| { format!("'{}'", x) })
                        .collect::<Vec<String>>()
                        .join(", ")
                )),
            }
        }
        // Type Errors:
        (x, y) => {
            let x_type = match x {
                CommandOptionInfoValueType::Bool => "Boolean",
                CommandOptionInfoValueType::String => "String",
                CommandOptionInfoValueType::Integer => "Integer",
                CommandOptionInfoValueType::Float => "Float",
                CommandOptionInfoValueType::Any => "None",
                _ => unreachable!(),
            };
            let y_type = match y {
                CommandOptionValue::None => "None",
                CommandOptionValue::Float(_) => "Float",
                CommandOptionValue::Integer(_) => "Integer",
                CommandOptionValue::String(_) => "String",
                CommandOptionValue::Bool(_) => "Boolean",
            };
            Err(format!(
                "option '{}' takes '{}' type but we got '{}' type",
                option, x_type, y_type
            ))
        }
    }
}

fn check_size(
    option: &str,
    input: &CommandOptionValue,
    size_definition: &CommandOptionInfoValueSize,
) -> Result<(), String> {
    let maybe_input_size = match input {
        &CommandOptionValue::String(ref x) => Some((x.len() as f64).clone()),
        &CommandOptionValue::Integer(ref x) => Some((*x as f64).clone()),
        &CommandOptionValue::Float(ref x) => Some(x.clone()),
        &CommandOptionValue::Bool(_) => None,
        &CommandOptionValue::None => None,
    };
    if let Some(input_size) = maybe_input_size {
        if let Some(min_size) = size_definition.min {
            if input_size < (min_size as f64) {
                return Err(format!(
                    "input size {} for option '{}' is lower than configured minimum size {}",
                    input_size, option, min_size
                ));
            }
        }
        if let Some(max_size) = size_definition.max {
            if input_size > (max_size as f64) {
                return Err(format!(
                    "input size {} for option '{}' is bigger than configured maximum size {}",
                    input_size, option, max_size
                ));
            }
        }
    }
    Ok(())
}
