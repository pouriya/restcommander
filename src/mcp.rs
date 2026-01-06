use crate::cmd;
use crate::cmd::runner::CommandOptionValue;
use crate::cmd::tree::{Command, CommandOptionInfo, CommandOptionInfoValueType};
use crate::cmd::CommandInput;
use crate::http::HttpResponseType;
use crate::settings::CommandLine;
use http::{Response, StatusCode};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

// === JSON-RPC Types ===
#[derive(Deserialize, Debug)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Value, // null or missing = notification
    method: String,
    #[serde(default)]
    params: Value,
}

impl JsonRpcRequest {
    pub fn is_notification(&self) -> bool {
        self.id.is_null()
    }
}

#[derive(Serialize, Debug)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize, Debug)]
pub struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl JsonRpcError {
    fn parse_error(msg: &str) -> Self {
        Self {
            code: -32700,
            message: msg.into(),
            data: None,
        }
    }
    fn invalid_request(msg: &str) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
            data: None,
        }
    }
    fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {}", method),
            data: None,
        }
    }
    fn invalid_params(msg: &str) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
            data: None,
        }
    }
    fn internal(msg: &str) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
            data: None,
        }
    }
    fn script_failed(msg: &str) -> Self {
        Self {
            code: -32001,
            message: msg.into(),
            data: None,
        }
    }
    fn tool_not_found(name: &str) -> Self {
        Self {
            code: -32003,
            message: format!("Tool not found: {}", name),
            data: None,
        }
    }
    fn resource_not_found(msg: &str) -> Self {
        Self {
            code: -32004,
            message: msg.into(),
            data: None,
        }
    }
}

impl JsonRpcResponse {
    fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }
    fn error(id: Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

// === MCP Types ===
#[derive(Serialize)]
pub struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Serialize)]
pub struct McpResource {
    uri: String,
    name: String,
    description: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
}

// === Translation Functions ===
fn option_to_json_schema_property(info: &CommandOptionInfo) -> Value {
    let mut prop = json!({});

    // Set type and enum
    match &info.value_type {
        CommandOptionInfoValueType::String => {
            prop["type"] = json!("string");
        }
        CommandOptionInfoValueType::Integer => {
            prop["type"] = json!("integer");
        }
        CommandOptionInfoValueType::Float => {
            prop["type"] = json!("number");
        }
        CommandOptionInfoValueType::Boolean => {
            prop["type"] = json!("boolean");
        }
        CommandOptionInfoValueType::Enum(values) => {
            prop["type"] = json!("string");
            prop["enum"] = json!(values);
        }
        CommandOptionInfoValueType::Any => {} // No type constraint
    }

    // Description
    if !info.description.is_empty() {
        prop["description"] = json!(info.description);
    }

    // Default value
    if let Some(ref default) = info.default_value {
        prop["default"] = command_option_value_to_json(default);
    }

    // Size constraints
    if let Some(ref size) = info.size {
        match info.value_type {
            CommandOptionInfoValueType::String => {
                if let Some(min) = size.min {
                    prop["minLength"] = json!(min);
                }
                if let Some(max) = size.max {
                    prop["maxLength"] = json!(max);
                }
            }
            CommandOptionInfoValueType::Integer | CommandOptionInfoValueType::Float => {
                if let Some(min) = size.min {
                    prop["minimum"] = json!(min);
                }
                if let Some(max) = size.max {
                    prop["maximum"] = json!(max);
                }
            }
            _ => {}
        }
    }

    prop
}

fn command_option_value_to_json(value: &CommandOptionValue) -> Value {
    match value {
        CommandOptionValue::None => Value::Null,
        CommandOptionValue::Bool(b) => json!(b),
        CommandOptionValue::Integer(i) => json!(i),
        CommandOptionValue::Float(f) => json!(f),
        CommandOptionValue::String(s) => json!(s),
    }
}

fn options_to_json_schema(options: &HashMap<String, CommandOptionInfo>) -> Value {
    let mut properties = json!({});
    let mut required = Vec::new();

    for (name, info) in options {
        properties[name] = option_to_json_schema_property(info);
        if info.required {
            required.push(name.clone());
        }
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

fn commands_to_tools(cmd: &Command, prefix: &str, root_name: &str) -> Vec<McpTool> {
    let mut tools = Vec::new();

    // Build current path: if prefix is empty and this is the root, skip it
    // Otherwise, append this command's name to the prefix
    let current_path = if prefix.is_empty() && cmd.name == *root_name {
        // This is the root directory - don't include it in paths
        String::new()
    } else if prefix.is_empty() {
        // First level under root - just use the command name
        cmd.name.clone()
    } else {
        // Nested level - append to prefix
        format!("{}/{}", prefix, cmd.name)
    };

    if cmd.is_directory {
        // Recursively collect tools from subcommands
        for (_, result_cmd) in &cmd.commands {
            if let Ok(sub_cmd) = result_cmd {
                tools.extend(commands_to_tools(sub_cmd, &current_path, root_name));
            }
        }
    } else {
        // This is a leaf command - create a tool
        if let Some(ref info) = cmd.info {
            let description = if !info.description.is_empty() {
                info.description.clone()
            } else {
                format!("Command: {}", cmd.name)
            };
            // current_path already excludes root_name
            tools.push(McpTool {
                name: current_path.clone(),
                description,
                input_schema: options_to_json_schema(&info.options),
            });
        }
    }

    tools
}

fn commands_to_resources(cmd: &Command, prefix: &str, root_name: &str) -> Vec<McpResource> {
    let mut resources = Vec::new();

    // Build current path: if prefix is empty and this is the root, skip it
    // Otherwise, append this command's name to the prefix
    let current_path = if prefix.is_empty() && cmd.name == *root_name {
        // This is the root directory - don't include it in paths
        String::new()
    } else if prefix.is_empty() {
        // First level under root - just use the command name
        cmd.name.clone()
    } else {
        // Nested level - append to prefix
        format!("{}/{}", prefix, cmd.name)
    };

    if cmd.is_directory {
        // Recursively collect resources from subcommands
        for (_, result_cmd) in &cmd.commands {
            if let Ok(sub_cmd) = result_cmd {
                resources.extend(commands_to_resources(sub_cmd, &current_path, root_name));
            }
        }
    } else {
        // This is a leaf command - check if it supports state
        if let Some(ref info) = cmd.info {
            if info.support_state {
                let description = if !info.description.is_empty() {
                    info.description.clone()
                } else {
                    format!("State for command: {}", cmd.name)
                };
                // current_path already excludes root_name
                resources.push(McpResource {
                    uri: format!("restcommander://{}/state", current_path),
                    name: cmd.name.clone(),
                    description,
                    mime_type: "application/json".to_string(),
                });
            }
        }
    }

    resources
}

fn resolve_tool_path(tool_name: &str, root: &Command) -> Result<Command, JsonRpcError> {
    let path_parts: Vec<&str> = tool_name.split('/').collect();

    // Build command path list: start with root name, then add path parts
    // Tool names from commands_to_tools exclude the root name (e.g., "basic/current-time")
    // so we always need to prepend root.name for search_for_command
    let mut command_path_list = vec![root.name.clone()];
    command_path_list.extend(path_parts.iter().map(|s| s.to_string()));

    cmd::search_for_command(&command_path_list, root)
        .map_err(|e| JsonRpcError::tool_not_found(&format!("Command not found: {}", e)))
}

fn resolve_resource_uri(uri: &str, root: &Command) -> Result<Command, JsonRpcError> {
    // Strip scheme and /state suffix
    let path = uri
        .strip_prefix("restcommander://")
        .ok_or_else(|| JsonRpcError::resource_not_found("invalid URI scheme"))?
        .strip_suffix("/state")
        .ok_or_else(|| JsonRpcError::resource_not_found("URI must end with /state"))?;

    // Find command and verify it supports state
    let command = resolve_tool_path(path, root)?;
    if !command
        .info
        .as_ref()
        .map(|i| i.support_state)
        .unwrap_or(false)
    {
        return Err(JsonRpcError::resource_not_found(
            "command does not support state",
        ));
    }
    Ok(command)
}

fn json_to_command_options(arguments: &Value) -> HashMap<String, CommandOptionValue> {
    let mut options = HashMap::new();
    if let Some(obj) = arguments.as_object() {
        for (key, value) in obj {
            if let Ok(option_value) = serde_json::from_value::<CommandOptionValue>(value.clone()) {
                options.insert(key.clone(), option_value);
            }
        }
    }
    options
}

// === Request Handlers ===
fn handle_initialize(_commands: &Command) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        Value::Null, // Will be replaced with actual id
        json!({
            "protocolVersion": "2024-11-05",
            "serverInfo": {
                "name": "restcommander",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": {
                "tools": {
                    "listChanged": false
                },
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                }
            }
        }),
    )
}

fn handle_tools_list(commands: &mut Command) -> JsonRpcResponse {
    // Reload commands before listing
    if let Err(e) = commands.reload() {
        return JsonRpcResponse::error(
            Value::Null,
            JsonRpcError::internal(&format!("Failed to reload commands: {}", e)),
        );
    }

    // Pass root name to strip it from tool paths
    let root_name = &commands.name;
    let tools = commands_to_tools(commands, "", root_name);
    JsonRpcResponse::ok(
        Value::Null,
        json!({
            "tools": tools
        }),
    )
}

fn handle_tools_call(
    req: &JsonRpcRequest,
    commands: &mut Command,
    _cfg: &CommandLine,
) -> JsonRpcResponse {
    let tool_name = match req.params["name"].as_str() {
        Some(name) => name,
        None => {
            return JsonRpcResponse::error(
                req.id.clone(),
                JsonRpcError::invalid_params("missing 'name' parameter"),
            );
        }
    };
    let empty_obj = Value::Object(Map::new());
    let arguments = req.params.get("arguments").unwrap_or(&empty_obj);

    // 1. Find command using existing tree search
    let command = match resolve_tool_path(tool_name, commands) {
        Ok(cmd) => cmd,
        Err(e) => return JsonRpcResponse::error(req.id.clone(), e),
    };

    // 2. Convert MCP arguments to CommandInput
    let options = json_to_command_options(arguments);
    let input = CommandInput {
        options,
        statistics: false,
    };

    // 3. Validate using existing validation
    let validated_input = match cmd::check_input(&command, &input) {
        Ok(input) => input,
        Err(e) => {
            return JsonRpcResponse::error(req.id.clone(), JsonRpcError::invalid_params(&e));
        }
    };

    // 4. Execute using existing runner
    let env_map = make_environment_variables_map_from_options(validated_input.options.clone());
    let output = match cmd::run_command(&command, &validated_input, env_map) {
        Ok(output) => output,
        Err(e) => {
            return JsonRpcResponse::error(req.id.clone(), JsonRpcError::internal(&e.to_string()));
        }
    };

    // 5. Wrap in MCP response format
    JsonRpcResponse::ok(
        req.id.clone(),
        json!({
            "content": [{
                "type": "text",
                "text": output.stdout
            }],
            "isError": output.exit_code != 0
        }),
    )
}

fn handle_resources_list(commands: &mut Command) -> JsonRpcResponse {
    // Reload commands before listing
    if let Err(e) = commands.reload() {
        return JsonRpcResponse::error(
            Value::Null,
            JsonRpcError::internal(&format!("Failed to reload commands: {}", e)),
        );
    }

    // Pass root name to strip it from resource URIs
    let root_name = &commands.name;
    let resources = commands_to_resources(commands, "", root_name);
    JsonRpcResponse::ok(
        Value::Null,
        json!({
            "resources": resources
        }),
    )
}

fn handle_resources_read(
    req: &JsonRpcRequest,
    commands: &mut Command,
    cfg: &CommandLine,
) -> JsonRpcResponse {
    let uri = match req.params["uri"].as_str() {
        Some(uri) => uri,
        None => {
            return JsonRpcResponse::error(
                req.id.clone(),
                JsonRpcError::invalid_params("missing 'uri' parameter"),
            );
        }
    };

    // Find command and verify it supports state
    let command = match resolve_resource_uri(uri, commands) {
        Ok(cmd) => cmd,
        Err(e) => return JsonRpcResponse::error(req.id.clone(), e),
    };

    // Get state using existing function
    let env_map = make_environment_variables_map_from_options(add_configuration_to_options(cfg));
    let output = match cmd::get_state(&command, env_map) {
        Ok(output) => output,
        Err(e) => {
            return JsonRpcResponse::error(req.id.clone(), JsonRpcError::internal(&e.to_string()));
        }
    };

    // Parse stdout: if valid JSON return parsed, else return as JSON string
    let content = if output.exit_code == 0 {
        match serde_json::from_str::<Value>(&output.stdout) {
            Ok(value) => value,
            Err(_) => Value::String(output.stdout.clone()),
        }
    } else {
        return JsonRpcResponse::error(
            req.id.clone(),
            JsonRpcError::script_failed(&format!(
                "script --state exited with code {}",
                output.exit_code
            )),
        );
    };

    JsonRpcResponse::ok(
        req.id.clone(),
        json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": serde_json::to_string(&content).unwrap_or_default()
            }]
        }),
    )
}

fn handle_notification(req: &JsonRpcRequest, commands: &mut Command) {
    match req.method.as_str() {
        "notifications/initialized" => {
            tracing::info!("Client initialized, reloading commands");
            let _ = commands.reload();
        }
        "notifications/cancelled" => {
            // Log only - we're synchronous so can't cancel in-flight
            tracing::debug!("Cancel notification received (no-op for sync server)");
        }
        _ => {
            tracing::trace!("Unknown notification: {}", req.method);
        }
    }
    // No response for notifications!
}

// Helper functions from http.rs
fn make_environment_variables_map_from_options(
    options: HashMap<String, CommandOptionValue>,
) -> HashMap<String, String> {
    options
        .into_iter()
        .fold(HashMap::new(), |mut env_map, (key, value)| {
            env_map.insert(
                key,
                match value {
                    CommandOptionValue::Bool(x) => x.to_string(),
                    CommandOptionValue::Integer(x) => x.to_string(),
                    CommandOptionValue::Float(x) => x.to_string(),
                    CommandOptionValue::None => "".to_string(),
                    CommandOptionValue::String(x) => x,
                },
            );
            env_map
        })
}

fn add_configuration_to_options(cfg: &CommandLine) -> HashMap<String, CommandOptionValue> {
    let mut options = HashMap::from([
        (
            "RESTCOMMANDER_CONFIG_SERVER_HOST".to_string(),
            CommandOptionValue::String(if cfg.host.as_str() == "0.0.0.0" {
                "127.0.0.1".to_string()
            } else {
                cfg.host.clone()
            }),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_PORT".to_string(),
            CommandOptionValue::Integer(cfg.port as i64),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_HTTP_BASE_PATH".to_string(),
            CommandOptionValue::String(cfg.http_base_path.clone()),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_USERNAME".to_string(),
            CommandOptionValue::String(cfg.username.clone()),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_API_TOKEN".to_string(),
            CommandOptionValue::String(cfg.api_token.clone().unwrap_or_default()),
        ),
        (
            "RESTCOMMANDER_CONFIG_COMMANDS_ROOT_DIRECTORY".to_string(),
            CommandOptionValue::String(cfg.root_directory.to_str().unwrap().to_string()),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_HTTPS".to_string(),
            CommandOptionValue::Bool(cfg.tls_key_file.as_ref().map(|_| true).unwrap_or(false)),
        ),
        (
            "RESTCOMMANDER_CONFIG_LOGGING_LEVEL_NAME".to_string(),
            CommandOptionValue::String(cfg.logging_level().to_string()),
        ),
        (
            "RESTCOMMANDER_CONFIGURATION_FILENAME".to_string(),
            CommandOptionValue::String("<COMMANDLINE>".to_string()),
        ),
    ]);
    for (key, value) in &cfg.configuration {
        options.insert(key.clone(), value.clone());
    }
    options
}

// === Main Dispatcher ===
pub fn handle_mcp_request(
    req: &JsonRpcRequest,
    commands: &mut Command,
    cfg: &CommandLine,
) -> JsonRpcResponse {
    // Validate jsonrpc version
    if req.jsonrpc != "2.0" {
        return JsonRpcResponse::error(
            req.id.clone(),
            JsonRpcError::invalid_request("jsonrpc must be '2.0'"),
        );
    }

    // Handle method
    let response = match req.method.as_str() {
        "initialize" => {
            let mut resp = handle_initialize(commands);
            resp.id = req.id.clone();
            resp
        }
        "tools/list" => {
            let mut resp = handle_tools_list(commands);
            resp.id = req.id.clone();
            resp
        }
        "tools/call" => handle_tools_call(req, commands, cfg),
        "resources/list" => {
            let mut resp = handle_resources_list(commands);
            resp.id = req.id.clone();
            resp
        }
        "resources/read" => handle_resources_read(req, commands, cfg),
        _ => JsonRpcResponse::error(req.id.clone(), JsonRpcError::method_not_found(&req.method)),
    };

    response
}

pub fn handle_mcp_body(body: &[u8], commands: &mut Command, cfg: &CommandLine) -> HttpResponseType {
    // Try batch first
    if let Ok(batch) = serde_json::from_slice::<Vec<JsonRpcRequest>>(body) {
        let responses: Vec<JsonRpcResponse> = batch
            .iter()
            .filter_map(|req| {
                if req.is_notification() {
                    handle_notification(req, commands);
                    None // No response for notifications
                } else {
                    Some(handle_mcp_request(req, commands, cfg))
                }
            })
            .collect();
        return json_response(&responses);
    }

    // Single request
    if let Ok(request) = serde_json::from_slice::<JsonRpcRequest>(body) {
        if request.is_notification() {
            handle_notification(&request, commands);
            return response_no_content(); // HTTP 204
        }
        return json_response(&handle_mcp_request(&request, commands, cfg));
    }

    // Parse error
    json_rpc_error_response(Value::Null, -32700, "Parse error", None)
}

pub fn json_rpc_error_response(
    id: Value,
    code: i32,
    message: &str,
    data: Option<Value>,
) -> HttpResponseType {
    let response = JsonRpcResponse::error(
        id,
        JsonRpcError {
            code,
            message: message.to_string(),
            data,
        },
    );
    json_response(&response)
}

fn json_response<T: serde::Serialize>(data: &T) -> HttpResponseType {
    let body = serde_json::to_string(data).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Connection", "close")
        .body(body.into_bytes())
        .unwrap()
}

fn response_no_content() -> HttpResponseType {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("Connection", "close")
        .body(Vec::new())
        .unwrap()
}
