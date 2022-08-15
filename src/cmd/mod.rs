use crate::cmd::errors::CommandError;
pub use crate::cmd::runner::{CommandInput, CommandOutput, CommandStats};
pub use crate::cmd::tree::{Command, CommandOptionInfo};
use crate::cmd::tree::{CommandOptionInfoValueType, CommandOptionValue};
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

pub fn check_input(command: &Command, input: &CommandInput) -> Result<CommandInput, String> {
    let mut new_input = input.clone();
    if command.info.is_none() {
        return Ok(new_input);
    };
    for (option, definition) in &command.info.as_ref().unwrap().options {
        let new_value = if new_input.options.contains_key(option.as_str()) {
            let input_value = new_input.options.get(option.as_str()).unwrap();
            check_definition(&option, &definition.value_type, input_value)?
        } else {
            if definition.default_value.is_none() {
                // So it is required
                if definition.required {
                    return Err(format!(
                        "required option {} is not given and has no default value",
                        option
                    ));
                };
                unreachable!();
            };
            definition.default_value.clone().unwrap()
        };
        new_input.options.insert(option.clone(), new_value);
    }

    Ok(new_input)
}

fn check_definition(
    option: &str,
    definition: &CommandOptionInfoValueType,
    input: &CommandOptionValue,
) -> Result<CommandOptionValue, String> {
    match (definition, input) {
        (CommandOptionInfoValueType::Any, value) => Ok(value.clone()),
        (CommandOptionInfoValueType::Bool, CommandOptionValue::Bool(flag)) => {
            Ok(CommandOptionValue::Bool(flag.clone()))
        }
        (CommandOptionInfoValueType::String(string_options), CommandOptionValue::String(value)) => {
            if string_options.max_size.is_some()
                && value.len() > string_options.max_size.unwrap() as usize
            {
                return Err(format!(
                    "size of option '{}' is bigger than max size",
                    option
                ));
            };
            if string_options.min_size.is_some()
                && value.len() < string_options.min_size.unwrap() as usize
            {
                return Err(format!(
                    "size of option '{}' is lower than min size",
                    option
                ));
            };
            Ok(CommandOptionValue::String(value.clone()))
        }
        (
            CommandOptionInfoValueType::Integer(integer_options),
            CommandOptionValue::Integer(value),
        ) => {
            if integer_options.max_size.is_some() && value > &integer_options.max_size.unwrap() {
                return Err(format!(
                    "size of option '{}' is bigger than max size",
                    option
                ));
            };
            if integer_options.min_size.is_some() && value < &integer_options.min_size.unwrap() {
                return Err(format!(
                    "size of option '{}' is lower than min size",
                    option
                ));
            };
            Ok(CommandOptionValue::Integer(value.clone()))
        }
        (CommandOptionInfoValueType::Float(float_options), CommandOptionValue::Float(value)) => {
            if float_options.max_size.is_some() && value > &float_options.max_size.unwrap() {
                return Err(format!(
                    "size of option '{}' is bigger than max size",
                    option
                ));
            };
            if float_options.min_size.is_some() && value < &float_options.min_size.unwrap() {
                return Err(format!(
                    "size of option '{}' is lower than min size",
                    option
                ));
            };
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
                CommandOptionInfoValueType::String(_) => "String",
                CommandOptionInfoValueType::Integer(_) => "Integer",
                CommandOptionInfoValueType::Float(_) => "Float",
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
