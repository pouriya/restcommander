use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time;

use tracing::{debug, info, trace};

use serde_derive::Deserialize;
use serde_json::json;

use thiserror::Error;
use rouille::{Request, Response as RouilleResponse};
use rouille::input;

use wildmatch::WildMatch;

use crate::captcha;
use crate::cmd;
use crate::cmd::runner::CommandOptionValue;
use crate::cmd::runner::CommandOptionsValue;
use crate::cmd::{Command, CommandInput, CommandInstruction, CommandStats};
use crate::report::{ReportContext, ReportError, State as ReportState};
use crate::settings::Cfg;
use crate::utils;
use crate::www;

//  for future use for HTTP "Server" header
// use structopt::clap::crate_name;

pub static API_RUN_BASE_PATH: &str = "/api/run";

#[derive(Error, Debug, Clone)]
pub enum HTTPError {
    #[error(transparent)]
    Authentication(#[from] HTTPAuthenticationError),
    #[error(transparent)]
    #[allow(clippy::upper_case_acronyms)]
    API(#[from] HTTPAPIError),
    #[error("{0}")]
    Deserialize(String),
}

impl HTTPError {
    fn http_error_code(&self) -> i32 {
        match self {
            Self::Authentication(x) => x.http_error_code(),
            Self::API(x) => x.http_error_code(),
            Self::Deserialize(_) => 2000,
        }
    }

    fn http_status_code(&self) -> u16 {
        match self {
            Self::Authentication(x) => x.http_status_code(),
            Self::API(x) => x.http_status_code(),
            Self::Deserialize(_) => 400,
        }
    }
}


#[derive(Error, Debug, Clone)]
pub enum HTTPAuthenticationError {
    #[error("could not decode authorization header value {data:?} to base64 ({source:?})")]
    Base64Decode {
        data: String,
        source: base64::DecodeError,
    },
    #[error("Username or password is not set in server configuration")]
    UsernameOrPasswordIsNotSet,
    #[error("Could not found username or password in {data:?}")]
    UsernameOrPasswordIsNotFound { data: String },
    #[error("Unknown authentication method {method:?}")]
    UnknownMethod { method: String },
    #[error("Invalid basic authentication with header value {header_value:?}")]
    InvalidBasicAuthentication { header_value: String },
    #[error("Invalid username or password")]
    InvalidUsernameOrPassword,
    #[error("invalid CAPTCHA")]
    InvalidCaptcha,
    #[error("Invalid CAPTCHA form")]
    InvalidCaptchaForm,
    #[error("Could not found HTTP Cookie header")]
    TokenNotFound,
    #[error("Token is expired")]
    TokenExpired,
    #[error("Token is invalid")]
    InvalidToken,
    #[error("{0}")]
    Captcha(String),
    #[error("Invalid IP {0}")]
    InvalidIP(String),
}

#[derive(Error, Debug, Clone)]
pub enum HTTPAPIError {
    #[error("{message}")]
    CommandNotFound { message: String },
    #[error("{message}")]
    CheckInput { message: String },
    #[error("{message}")]
    InitializeCommand { message: String },
    #[error("Password should not be empty")]
    EmptyPassword,
    #[error("Server configuration does not allow client to change the password")]
    NoPasswordFile,
    #[error("Could not save new password to configured password file ({message})")]
    SaveNewPassword { message: String },
    #[error("{message}")]
    ReportNotAvailable { message: String },
    #[error("{message}")]
    Report { message: String },
    #[error("No report found")]
    ReportNotFound,
}

impl HTTPAPIError {
    fn http_error_code(&self) -> i32 {
        match self {
            // Keep 1001 for Command errors
            Self::CommandNotFound { .. } => 1002,
            Self::CheckInput { .. } => 1003,
            Self::InitializeCommand { .. } => 1004,
            Self::EmptyPassword => 1007,
            Self::NoPasswordFile => 1008,
            Self::SaveNewPassword { .. } => 1010,
            Self::ReportNotAvailable { .. } => 1011,
            Self::Report { .. } => 1012,
            Self::ReportNotFound => 1013,
        }
    }

    fn http_status_code(&self) -> u16 {
        match self {
            Self::CommandNotFound { .. } => 404,
            Self::CheckInput { .. } => 400,
            Self::InitializeCommand { .. } => 500,
            Self::EmptyPassword => 400,
            Self::NoPasswordFile => 503,
            Self::SaveNewPassword { .. } => 500,
            Self::ReportNotAvailable { .. } => 406,
            Self::Report { .. } => 500,
            Self::ReportNotFound => 404,
        }
    }
}

impl HTTPAuthenticationError {
    fn http_error_code(&self) -> i32 {
        match self {
            Self::Base64Decode { .. } => 2002,
            Self::UsernameOrPasswordIsNotFound { .. } => 2003,
            Self::UsernameOrPasswordIsNotSet { .. } => 2004,
            Self::UnknownMethod { .. } => 2005,
            Self::InvalidBasicAuthentication { .. } => 2006,
            Self::InvalidUsernameOrPassword => 2007,
            Self::InvalidCaptcha => 2008,
            Self::InvalidCaptchaForm => 2009,
            Self::TokenNotFound => 2010,
            Self::TokenExpired => 2011,
            Self::InvalidToken => 2012,
            Self::Captcha(_) => 2012,
            Self::InvalidIP(_) => 2013,
        }
    }

    fn http_status_code(&self) -> u16 {
        match self {
            Self::Base64Decode { .. } => 400,
            Self::UsernameOrPasswordIsNotFound { .. } => 401,
            Self::UsernameOrPasswordIsNotSet { .. } => 409,
            Self::UnknownMethod { .. } => 400,
            Self::InvalidBasicAuthentication { .. } => 400,
            Self::InvalidUsernameOrPassword => 401,
            Self::InvalidCaptcha => 401,
            Self::InvalidCaptchaForm => 400,
            Self::TokenNotFound => 401,
            Self::TokenExpired => 401,
            Self::InvalidToken => 401,
            Self::Captcha(_) => 406,
            Self::InvalidIP(_) => 401,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SetPassword {
    password: String,
}

#[derive(Debug, Deserialize)]
struct Report {
    before_time: Option<String>,
    after_time: Option<String>,
    context: Option<ReportContext>,
    from: Option<String>,
    limit: Option<usize>,
}

#[inline]
fn exit_code_to_status_code(exit_code: i32) -> u16 {
    match exit_code {
        0 => 200,
        1 => 500,
        2 => 400,
        3 => 403,
        4 => 404,
        5 => 503,
        6 => 406,
        7 => 501,
        8 => 409,
        9 => 408,
        _ => 500,
    }
}

pub struct ServerConfig {
    pub handler_state: Arc<HandlerState>,
    pub address: String,
    pub tls_cert: Option<Vec<u8>>,
    pub tls_key: Option<Vec<u8>>,
    pub has_tls: bool,
}

pub fn setup(
    cfg: Arc<RwLock<Cfg>>,
    commands: Arc<RwLock<Command>>,
    report_state: Arc<RwLock<ReportState>>,
) -> Result<ServerConfig, String> {
    let server_options = cfg.read().unwrap().config_value.server.clone();
    let host = server_options.host.clone();
    let port = server_options.port;

    let maybe_captcha = if cfg.read().unwrap().config_value.server.captcha {
        Some(Arc::new(RwLock::new(captcha::Captcha::new())))
    } else {
        None
    };
    let tokens = Arc::new(RwLock::new(HashMap::new()));
    
    // Create shared state for handlers
    let handler_state = Arc::new(HandlerState {
        cfg: cfg.clone(),
        commands: commands.clone(),
        report_state: report_state.clone(),
        tokens: tokens.clone(),
        maybe_captcha: maybe_captcha.clone(),
    });

    let address = format!("{}:{}", host, port);
    
    if server_options.tls_cert_file.clone().is_some()
        && server_options.tls_key_file.clone().is_some()
    {
        // Load TLS certificates as raw bytes (rouille handles PEM parsing)
        let cert_bytes = std::fs::read(server_options.tls_cert_file.clone().unwrap())
            .map_err(|e| format!("Could not read cert file: {}", e))?;
        let key_bytes = std::fs::read(server_options.tls_key_file.clone().unwrap())
            .map_err(|e| format!("Could not read key file: {}", e))?;
        
            debug!(
            "Prepared HTTPS server on {}:{} with cert file {:?} and key file {:?}",
                host,
                port,
                server_options.tls_cert_file.clone().unwrap(),
                server_options.tls_key_file.clone().unwrap()
            );
        
        Ok(ServerConfig {
            handler_state,
            address,
            tls_cert: Some(cert_bytes),
            tls_key: Some(key_bytes),
            has_tls: true,
        })
    } else {
        debug!("Prepared HTTP server on {}:{}", host, port);
        
        Ok(ServerConfig {
            handler_state,
            address,
            tls_cert: None,
            tls_key: None,
            has_tls: false,
        })
    }
}

pub fn start_server(config: ServerConfig) -> Result<(), String> {
    let state = config.handler_state.clone();
    let address = config.address.clone();
    
    info!(
        address = address.as_str(),
        tls = config.has_tls,
        "Starting server"
    );
    
    if let (Some(cert), Some(key)) = (config.tls_cert, config.tls_key) {
        // rouille::Server::new_ssl takes cert and key as Vec<u8>
        let _server = rouille::Server::new_ssl(
            address,
            move |request| handle_request(request, state.clone()),
            cert,
            key,
        ).map_err(|e| format!("Failed to start HTTPS server: {}", e))?;
        // Server blocks here - call run() to start
        _server.run();
    } else {
        rouille::start_server(
            address,
            move |request| handle_request(request, state.clone()),
        );
    }
    
    Ok(())
}

pub struct HandlerState {
    cfg: Arc<RwLock<Cfg>>,
    commands: Arc<RwLock<Command>>,
    report_state: Arc<RwLock<ReportState>>,
    tokens: Arc<RwLock<HashMap<String, usize>>>,
    maybe_captcha: Option<Arc<RwLock<captcha::Captcha>>>,
}

fn handle_request(request: &Request, state: Arc<HandlerState>) -> RouilleResponse {
    http_logging_rouille(request);
    
    let url = request.url();
    let method = request.method();
    
    // Handle root redirect
    if method == "GET" && url == "/" {
        return redirect_root_to_index_html(&state.cfg);
    }
    
    // Handle static files
    if method == "GET" && url.starts_with("/static/") {
        let tail = url.strip_prefix("/static/").unwrap_or("");
        return handle_static(request, &state.cfg, tail);
    }
    
    // Handle API routes
    rouille::router!(request,
        (GET) (/api/public/captcha) => {
            api_captcha(&state.maybe_captcha)
        },
        (GET) (/api/public/configuration) => {
            api_configuration(&state.cfg)
        },
        (GET) (/api/auth/test) => {
            api_auth_test(request, &state.tokens, &state.cfg)
        },
        (POST) (/api/auth/token) => {
            api_auth_token_handler(request, &state.cfg, &state.maybe_captcha, &state.tokens)
        },
        (GET) (/api/commands) => {
            api_get_commands(request, &state.commands, &state.tokens, &state.cfg)
        },
        (POST) (/api/setPassword) => {
            api_set_password(request, &state.cfg, &state.tokens)
        },
        (POST) (/api/report) => {
            api_report(request, &state.report_state)
        },
        _ => {
            // Handle dynamic routes for /api/run/* and /api/state/*
            if method == "POST" && url.starts_with("/api/run/") {
                let tail = url.strip_prefix("/api/run/").unwrap_or("");
                return api_run_command(request, &state.cfg, &state.commands, &state.report_state, &state.tokens, tail);
            }
            if method == "GET" && url.starts_with("/api/state/") {
                let tail = url.strip_prefix("/api/state/").unwrap_or("");
                return api_get_command_state(request, &state.cfg, &state.commands, &state.report_state, &state.tokens, tail);
            }
            RouilleResponse::text("Not Found").with_status_code(404)
        }
    )
}

// Helper functions for rouille handlers
fn http_logging_rouille(_request: &Request) {
    // Logging will be handled per response
}

fn redirect_root_to_index_html(cfg: &Arc<RwLock<Cfg>>) -> RouilleResponse {
    let cfg_value = cfg.read().unwrap().config_value.clone();
    if cfg_value.www.enabled {
        RouilleResponse::redirect_301(format!("{}static/index.html", cfg_value.server.http_base_path))
            } else {
        RouilleResponse::text("<html><body>Service Unavailable!</body></html>")
            .with_status_code(403)
    }
}

fn handle_static(_request: &Request, cfg: &Arc<RwLock<Cfg>>, tail: &str) -> RouilleResponse {
    // Try external static directory first
    let www_cfg = cfg.read().unwrap().config_value.www.clone();
    if www_cfg.enabled && www_cfg.static_directory.is_dir() {
        let file_path = www_cfg.static_directory.join(tail);
        if file_path.exists() && file_path.is_file() {
            if let Ok(data) = std::fs::read(&file_path) {
                // Simple mime type detection
                let mime_type = match file_path.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html",
                    Some("css") => "text/css",
                    Some("js") => "text/javascript",
                    Some("json") => "application/json",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("png") => "image/png",
                    Some("ico") => "image/x-icon",
                    _ => "application/octet-stream",
                };
                return RouilleResponse::from_data(mime_type, data);
            }
        }
    }
    
    // Fall back to internal static files
    if let Some((bytes, maybe_mime_type)) = www::handle_static(tail.to_string()) {
        let mime_type = maybe_mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
        RouilleResponse::from_data(mime_type, bytes)
                        } else {
        RouilleResponse::text("Not Found").with_status_code(404)
    }
}

fn api_captcha(maybe_captcha: &Option<Arc<RwLock<captcha::Captcha>>>) -> RouilleResponse {
    if let Some(captcha) = maybe_captcha {
        let (id, _, png_image) = captcha.write().unwrap().generate(true);
        make_api_response_ok_with_result(serde_json::json!({"id": id, "image": png_image}))
                                    } else {
        make_api_response(Err(HTTPError::Authentication(
            HTTPAuthenticationError::Captcha(
                std::io::Error::from(std::io::ErrorKind::Unsupported).to_string(),
            ),
        )))
    }
}

fn api_configuration(cfg: &Arc<RwLock<Cfg>>) -> RouilleResponse {
    make_api_response_ok_with_result(serde_json::Value::Object(
        cfg.read().unwrap().config_value.www.configuration.clone()
            .into_iter()
            .fold(serde_json::Map::new(), |mut acc, item| {
                acc.insert(item.0, serde_json::Value::String(item.1));
                acc
            }),
    ))
}

fn api_auth_test(request: &Request, tokens: &Arc<RwLock<HashMap<String, usize>>>, cfg: &Arc<RwLock<Cfg>>) -> RouilleResponse {
    match check_authentication(request, tokens, cfg) {
        Ok(_) => make_api_response_ok(),
        Err(e) => make_api_response(Err(HTTPError::Authentication(e))),
    }
}

fn api_auth_token_handler(
    request: &Request,
    cfg: &Arc<RwLock<Cfg>>,
    maybe_captcha: &Option<Arc<RwLock<captcha::Captcha>>>,
    tokens: &Arc<RwLock<HashMap<String, usize>>>,
) -> RouilleResponse {
    let authorization_value = request.header("Authorization").unwrap_or("");
    // Parse form data manually from POST body
    let form: HashMap<String, String> = if request.method() == "POST" {
        match input::post::raw_urlencoded_post_input(request) {
            Ok(data) => {
                // raw_urlencoded_post_input returns Vec<(String, String)>
                data.into_iter().collect()
            }
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };
    
    match authentication_with_basic(
                    cfg.clone(),
                    maybe_captcha.clone(),
        authorization_value.to_string(),
                    form,
    ) {
        Err(error) => make_api_response(Err(HTTPError::Authentication(error))),
        Ok(_) => {
            let token = utils::to_sha512(uuid::Uuid::new_v4().to_string());
            let token_timeout = cfg.read().unwrap().config_value.server.token_timeout;
            let timestamp = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
            .unwrap()
                .as_secs() as usize
                + token_timeout;
            tokens.write().unwrap().insert(token.clone(), timestamp);
            make_api_response_with_headers(
                Ok(serde_json::json!({ "token": token })),
                Some(vec![(
                    std::borrow::Cow::Borrowed("Set-Cookie"),
                    std::borrow::Cow::Owned(format!("token={}; Path=/; Max-Age={}; SameSite=None; Secure;", token, token_timeout)),
                )]),
            )
        }
    }
}

fn api_get_commands(
    request: &Request,
    commands: &Arc<RwLock<Command>>,
    tokens: &Arc<RwLock<HashMap<String, usize>>>,
    cfg: &Arc<RwLock<Cfg>>,
) -> RouilleResponse {
    match check_authentication(request, tokens, cfg) {
        Ok(_) => make_api_response_ok_with_result(
            serde_json::to_value(commands.read().unwrap().deref()).unwrap(),
        ),
        Err(e) => make_api_response(Err(HTTPError::Authentication(e))),
    }
}

fn api_set_password(
    request: &Request,
    cfg: &Arc<RwLock<Cfg>>,
    tokens: &Arc<RwLock<HashMap<String, usize>>>,
) -> RouilleResponse {
    match check_authentication(request, tokens, cfg) {
        Ok(_) => {
        let input: SetPassword = match input::json_input(request) {
            Ok(p) => p,
            Err(_) => return make_api_response(Err(HTTPError::Deserialize("Invalid JSON".to_string()))),
        };
            match try_set_password(cfg.clone(), input) {
                Ok(_) => make_api_response_ok(),
                Err(e) => make_api_response(Err(HTTPError::API(e))),
            }
        }
        Err(e) => make_api_response(Err(HTTPError::Authentication(e))),
    }
}

fn api_report(request: &Request, report_state: &Arc<RwLock<ReportState>>) -> RouilleResponse {
    let input: Report = match input::json_input(request) {
        Ok(r) => r,
        Err(_) => return make_api_response(Err(HTTPError::Deserialize("Invalid JSON".to_string()))),
    };
    
    let report_state_locked = report_state.read().unwrap();
    match crate::report::search(
        input.from,
        input.before_time,
        input.after_time,
        input.context,
        input.limit,
                    report_state_locked.clone(),
    ) {
                    Ok(report_list) => {
                        if report_list.is_empty() {
                            make_api_response(Err(HTTPError::API(HTTPAPIError::ReportNotFound)))
                        } else {
                            let report_list = report_list
                                .into_iter()
                                .map(|report| serde_json::to_value(&report).unwrap())
                                .collect();
                            make_api_response_ok_with_result(report_list)
                        }
                    }
                    Err(error) => make_api_response(Err(HTTPError::API(match error {
                        ReportError::NotAvailable => HTTPAPIError::ReportNotAvailable {
                            message: error.to_string(),
                        },
                        _ => HTTPAPIError::Report {
                            message: error.to_string(),
                        },
                    }))),
    }
}

fn check_authentication(
    request: &Request,
    tokens: &Arc<RwLock<HashMap<String, usize>>>,
    cfg: &Arc<RwLock<Cfg>>,
) -> Result<(), HTTPAuthenticationError> {
            let cfg_value = cfg.read().unwrap().config_value.clone();
    if cfg_value.server.password_sha512.is_empty() {
        return Ok(());
    }
    
    let token = extract_token(request)?;
    authentication_with_token(tokens.clone(), token, cfg.clone())
}

fn extract_token(request: &Request) -> Result<String, HTTPAuthenticationError> {
    // Try cookie first
    if let Some(cookie_header) = request.header("Cookie") {
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if cookie.starts_with("token=") {
                return Ok(cookie.strip_prefix("token=").unwrap_or("").to_string());
            }
        }
    }
    
    // Try Authorization header
    if let Some(auth_header) = request.header("Authorization") {
        let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
        if parts.len() == 2 && parts[0] == "Bearer" {
            return Ok(parts[1].to_string());
        }
        return Err(HTTPAuthenticationError::InvalidBasicAuthentication {
            header_value: auth_header.to_string(),
        });
    }
    
    Err(HTTPAuthenticationError::TokenNotFound)
}

fn check_ip_address_rouille(request: &Request, cfg: &Arc<RwLock<Cfg>>) -> Result<(), HTTPAuthenticationError> {
    let ip_whitelist = cfg.read().unwrap().config_value.server.ip_whitelist.clone();
    if ip_whitelist.is_empty() {
        return Ok(());
    }
    
    let remote_addr = request.remote_addr();
    let ip = remote_addr.ip().to_string();
    for wildcard_ip in ip_whitelist {
        if WildMatch::new(wildcard_ip.as_str()).matches(ip.as_str()) {
            return Ok(());
        }
    }
    Err(HTTPAuthenticationError::InvalidIP(ip))
}

fn api_run_command(
    request: &Request,
    cfg: &Arc<RwLock<Cfg>>,
    commands: &Arc<RwLock<Command>>,
    report_state: &Arc<RwLock<ReportState>>,
    tokens: &Arc<RwLock<HashMap<String, usize>>>,
    command_path: &str,
) -> RouilleResponse {
    // Check IP whitelist
    if let Err(e) = check_ip_address_rouille(request, cfg) {
        return make_api_response(Err(HTTPError::Authentication(e)));
    }
    
    // Check authentication
    if let Err(e) = check_authentication(request, tokens, cfg) {
        return make_api_response(Err(HTTPError::Authentication(e)));
    }
    
    match extract_command_input(request, cfg) {
        Ok(input) => {
            match maybe_run_command(
                commands.clone(),
                command_path.to_string(),
                input,
                report_state.clone(),
                request.remote_addr().to_string(),
            ) {
                Ok(response) => response,
                Err(e) => make_api_response(Err(HTTPError::API(e))),
            }
        }
        Err(e) => make_api_response(Err(e)),
    }
}

fn api_get_command_state(
    request: &Request,
    cfg: &Arc<RwLock<Cfg>>,
    commands: &Arc<RwLock<Command>>,
    report_state: &Arc<RwLock<ReportState>>,
    tokens: &Arc<RwLock<HashMap<String, usize>>>,
    command_path: &str,
) -> RouilleResponse {
    // Check IP whitelist
    if let Err(e) = check_ip_address_rouille(request, cfg) {
        return make_api_response(Err(HTTPError::Authentication(e)));
    }
    
    // Check authentication
    if let Err(e) = check_authentication(request, tokens, cfg) {
        return make_api_response(Err(HTTPError::Authentication(e)));
    }
    
    match maybe_get_command_state(
        cfg.clone(),
        commands.clone(),
        command_path.to_string(),
        report_state.clone(),
        request.remote_addr().to_string(),
    ) {
        Ok(response) => response,
        Err(e) => make_api_response(Err(HTTPError::API(e))),
    }
}

fn extract_command_input(request: &Request, cfg: &Arc<RwLock<Cfg>>) -> Result<CommandInput, HTTPError> {
    let mut input = CommandInput::default();
    
    // Extract from headers
    let mut options = CommandOptionsValue::new();
    for (header_name, header_value) in request.headers() {
        let header_name_upper = header_name.to_uppercase();
        if header_name_upper == "X-RESTCOMMANDER-STATISTICS" {
            input.statistics = true;
            continue;
        }
        if header_name_upper.starts_with("X-") && header_name.len() > 2 {
            let key = header_name[2..].to_string();
            let value_str = header_value;
            let value = serde_json::from_str::<CommandOptionValue>(value_str)
                .unwrap_or_else(|_| CommandOptionValue::String(value_str.to_string()));
            options.insert(key, value);
        } else {
            let key = format!("RESTCOMMANDER_HEADER_{}", header_name_upper.replace("-", "_"));
            let value = CommandOptionValue::String(header_value.to_string());
            options.insert(key, value);
        }
    }
    
    // Add client IP and port
    let remote_addr = request.remote_addr();
    options.insert(
        "RESTCOMMANDER_CLIENT_IP".to_string(),
        CommandOptionValue::String(remote_addr.ip().to_string()),
    );
    options.insert(
        "RESTCOMMANDER_CLIENT_PORT".to_string(),
        CommandOptionValue::Integer(remote_addr.port() as i64),
    );
    
    // Extract from body (JSON or form)
    let content_type = request.header("Content-Type").unwrap_or("");
    let body_options = if content_type.contains("application/json") {
        input::json_input::<CommandOptionsValue>(request).unwrap_or_default()
    } else if content_type.contains("application/x-www-form-urlencoded") {
        // For form data, we need to parse it manually or use post module
        // For now, return empty - can be enhanced later
        CommandOptionsValue::new()
    } else {
        CommandOptionsValue::new()
    };
    
    // Extract from query string
    let query_options = {
        let query = request.raw_query_string();
        if query.is_empty() {
            CommandOptionsValue::new()
        } else {
            serde_urlencoded::from_str::<CommandOptionsValue>(query).unwrap_or_else(|_| CommandOptionsValue::new())
        }
    };
    
    // Unify all options
    input.options = unify_options(vec![
        options,
        query_options,
        body_options,
        add_configuration_to_options(cfg.clone()),
    ]);
    
    Ok(input)
}

// Old filter functions removed - replaced with rouille handlers above
// Utility functions that are still used by handlers are kept below

fn authentication_with_basic(
    cfg: Arc<RwLock<Cfg>>,
    maybe_captcha: Option<Arc<RwLock<captcha::Captcha>>>,
    authorization_value: String,
    form: HashMap<String, String>,
) -> Result<(), HTTPAuthenticationError> {
    let server_cfg = cfg.read().unwrap().config_value.server.clone();
    if server_cfg.password_sha512.is_empty() && server_cfg.username.is_empty() {
        return Ok(());
    };
    if server_cfg.password_sha512.is_empty() || server_cfg.username.is_empty() {
        return Err(HTTPAuthenticationError::UsernameOrPasswordIsNotSet);
    };
    match authorization_value
        .as_str()
        .splitn(2, ' ')
        .collect::<Vec<&str>>()[..]
    {
        ["Basic", username_password] => {
            let decoded_username_password =
                base64::decode(username_password).map_err(|reason| {
                    HTTPAuthenticationError::Base64Decode {
                        data: username_password.to_string(),
                        source: reason,
                    }
                })?;
            match String::from_utf8(decoded_username_password)
                .unwrap()
                .splitn(2, ':')
                .collect::<Vec<&str>>()[..]
            {
                [username, password] => {
                    if username == server_cfg.username {
                        let password_sha512 = utils::to_sha512(password);
                        trace!(
                            username = username,
                            password_sha512 = password_sha512.as_str(),
                            "New client provided credentials.",
                        );
                        if server_cfg.password_sha512 == password_sha512 {
                            if maybe_captcha.is_none() {
                                return Ok(());
                            };
                            return if form.len() == 1 {
                                let (key, value) = form
                                    .into_iter()
                                    .fold(None, |_, key_value| Some(key_value.clone()))
                                    .unwrap()
                                    .clone();
                                if maybe_captcha.unwrap().write().unwrap().compare_and_update(
                                    key.to_string(),
                                    value,
                                    server_cfg.captcha_case_sensitive,
                                ) {
                                    Ok(())
                                } else {
                                    Err(HTTPAuthenticationError::InvalidCaptcha {})
                                }
                            } else {
                                Err(HTTPAuthenticationError::InvalidCaptchaForm {})
                            };
                        };
                    } else {
                        debug!(
                            username = username,
                            "Client authenticated with unknown username."
                        );
                    };
                    Err(HTTPAuthenticationError::InvalidUsernameOrPassword)
                }
                [value] => Err(HTTPAuthenticationError::UsernameOrPasswordIsNotFound {
                    data: value.to_string(),
                }),
                _ => Err(HTTPAuthenticationError::UsernameOrPasswordIsNotFound {
                    data: "".to_string(),
                }),
            }
        }
        [unknown_method, _] => Err(HTTPAuthenticationError::UnknownMethod {
            method: unknown_method.to_string(),
        }),
        _ => Err(HTTPAuthenticationError::InvalidBasicAuthentication {
            header_value: authorization_value,
        }),
    }
}

fn authentication_with_token(
    tokens: Arc<RwLock<HashMap<String, usize>>>,
    token: String,
    cfg: Arc<RwLock<Cfg>>,
) -> Result<(), HTTPAuthenticationError> {
    let cfg = cfg.clone().read().unwrap().config_value.clone();
    if cfg.server.password_sha512.is_empty() {
        return Ok(());
    }
    if token.is_empty() {
        return Err(HTTPAuthenticationError::TokenNotFound);
    };
    if let Some(ref api_token) = cfg.server.api_token {
        if &token == api_token {
            return Ok(());
        }
    }
    return if let Some(expire_time) = tokens.clone().read().unwrap().get(token.as_str()) {
        if expire_time
            > &(time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize)
        {
            return Ok(());
        };
        Err(HTTPAuthenticationError::TokenExpired)
    } else {
        Err(HTTPAuthenticationError::InvalidToken)
    };
}

fn maybe_run_command(
    commands: Arc<RwLock<Command>>,
    command_path: String,
    command_input: CommandInput,
    report_state: Arc<RwLock<ReportState>>,
    address: String,
) -> Result<RouilleResponse, HTTPAPIError> {
    let root_command = commands.read().unwrap().clone();
    let command_path_list: Vec<String> = PathBuf::from(root_command.name.clone())
        .join(PathBuf::from(command_path))
        .components()
        .map(|x| x.as_os_str().to_str().unwrap().to_string())
        .collect();
    let command = cmd::search_for_command(&command_path_list, &root_command).map_err(|reason| {
        HTTPAPIError::CommandNotFound {
            message: reason.to_string(),
        }
    })?;
    let input =
        cmd::check_input(&command, &command_input).map_err(|reason| HTTPAPIError::CheckInput {
            message: reason.to_string(),
        })?;
    let env_map = make_environment_variables_map_from_options(input.options.clone());
    let command_output = cmd::run_command(&command, &input, env_map).map_err(|reason| {
        HTTPAPIError::InitializeCommand {
            message: reason.to_string(),
        }
    })?;
    for instruction in command_output.instruction_list.clone() {
        let report_state_locked = report_state.read().unwrap();
        if let CommandInstruction::Report(report_data) = instruction {
            crate::report::report(
                address.clone(),
                ReportContext::Run,
                report_data.clone(),
                report_state_locked.clone(),
                command.http_path.clone().to_str().unwrap().to_string(),
                None,
            );
        };
        drop(report_state_locked)
    }
    let http_status_code = exit_code_to_status_code(command_output.exit_code) as u16;
    let http_response_body = if command_output.stdout.is_empty() {
        serde_json::Value::Null
    } else if command_output.decoded_stdout.is_err() {
        serde_json::Value::String(command_output.stdout)
    } else {
        command_output.decoded_stdout.unwrap()
    };
    Ok(make_api_response_with_header_and_stats(
        Ok(http_response_body),
        None,
        if command_input.statistics {
            Some(command_output.stats)
        } else {
            None
        },
        Some(http_status_code),
    ))
}

fn maybe_get_command_state(
    cfg: Arc<RwLock<Cfg>>,
    commands: Arc<RwLock<Command>>,
    command_path: String,
    report_state: Arc<RwLock<ReportState>>,
    address: String,
) -> Result<RouilleResponse, HTTPAPIError> {
    let root_command = commands.read().unwrap().clone();
    let command_path_list: Vec<String> = PathBuf::from(root_command.name.clone())
        .join(PathBuf::from(command_path))
        .components()
        .map(|x| x.as_os_str().to_str().unwrap().to_string())
        .collect();
    let command = cmd::search_for_command(&command_path_list, &root_command).map_err(|reason| {
        HTTPAPIError::CommandNotFound {
            message: reason.to_string(),
        }
    })?;
    let command_output = cmd::get_state(
        &command,
        make_environment_variables_map_from_options(add_configuration_to_options(cfg.clone())),
    )
    .map_err(|reason| HTTPAPIError::InitializeCommand {
        message: reason.to_string(),
    })?;
    for instruction in command_output.instruction_list.clone() {
        let report_state_locked = report_state.read().unwrap();
        if let CommandInstruction::Report(report_data) = instruction {
            crate::report::report(
                address.clone(),
                ReportContext::State,
                report_data.clone(),
                report_state_locked.clone(),
                command.http_path.clone().to_str().unwrap().to_string(),
                None,
            );
        };
        drop(report_state_locked)
    }
    let http_status_code = exit_code_to_status_code(command_output.exit_code) as u16;
    let http_response_body = if command_output.stdout.is_empty() {
        serde_json::Value::Null
    } else if command_output.decoded_stdout.is_err() {
        serde_json::Value::String(command_output.stdout)
    } else {
        command_output.decoded_stdout.unwrap()
    };
    Ok(make_api_response_with_header_and_stats(
        Ok(http_response_body),
        None,
        None, // TODO
        Some(http_status_code),
    ))
}

fn make_environment_variables_map_from_options(
    options: CommandOptionsValue,
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

fn add_configuration_to_options(cfg: Arc<RwLock<Cfg>>) -> CommandOptionsValue {
    let cfg_instance = cfg.read().unwrap().config_value.clone();
    let mut options = CommandOptionsValue::from([
        (
            "RESTCOMMANDER_CONFIG_SERVER_HOST".to_string(),
            CommandOptionValue::String(if cfg_instance.server.host.as_str() == "0.0.0.0" {
                "127.0.0.1".to_string()
            } else {
                cfg_instance.server.host.clone()
            }),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_PORT".to_string(),
            CommandOptionValue::Integer(cfg_instance.server.port as i64),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_HTTP_BASE_PATH".to_string(),
            CommandOptionValue::String(cfg_instance.server.http_base_path),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_USERNAME".to_string(),
            CommandOptionValue::String(cfg_instance.server.username),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_API_TOKEN".to_string(),
            CommandOptionValue::String(cfg_instance.server.api_token.unwrap_or_default()),
        ),
        (
            "RESTCOMMANDER_CONFIG_COMMANDS_ROOT_DIRECTORY".to_string(),
            CommandOptionValue::String(
                cfg_instance
                    .commands
                    .root_directory
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
        ),
        (
            "RESTCOMMANDER_CONFIG_SERVER_HTTPS".to_string(),
            CommandOptionValue::Bool(
                cfg_instance
                    .server
                    .tls_key_file
                    .map(|_| true)
                    .unwrap_or(false),
            ),
        ),
        (
            "RESTCOMMANDER_CONFIG_LOGGING_LEVEL_NAME".to_string(),
            CommandOptionValue::String(
                cfg_instance
                    .logging
                    .level_name
                    .to_level_filter()
                    .to_string(),
            ),
        ),
        (
            "RESTCOMMANDER_CONFIGURATION_FILENAME".to_string(),
            CommandOptionValue::String(
                cfg.read()
                    .unwrap()
                    .filename
                    .as_ref()
                    .unwrap_or(&PathBuf::new())
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
        ),
    ]);
    for (key, value) in cfg_instance.commands.configuration {
        options.insert(key, value);
    }
    options
}

fn try_set_password(
    cfg: Arc<RwLock<Cfg>>,
    password: SetPassword,
) -> Result<RouilleResponse, HTTPAPIError> {
    if password.password.is_empty() {
        return Err(HTTPAPIError::EmptyPassword);
    };
    let password_file = cfg
        .read()
        .unwrap()
        .config_value
        .server
        .password_file
        .clone();
    if password_file.to_str().unwrap().is_empty() {
        return Err(HTTPAPIError::NoPasswordFile);
    };
    let password_sha512 = utils::to_sha512(password.password);
    std::fs::write(password_file, password_sha512.clone()).map_err(|reason| {
        HTTPAPIError::SaveNewPassword {
            message: reason.to_string(),
        }
    })?;
    cfg.write().unwrap().config_value.server.password_sha512 = password_sha512;
    Ok(make_api_response_ok())
}

fn unify_options(options_list: Vec<CommandOptionsValue>) -> CommandOptionsValue {
    let mut options = CommandOptionsValue::new();
    for options_list_item in options_list {
        for (option, mut value) in options_list_item {
            if options.contains_key(option.as_str()) {
                trace!(option = option.as_str(), old = ?options.get(option.as_str()).unwrap(), new = ?value, "Replacing value for option.")
            };
            if let CommandOptionValue::String(ref value_string) = value {
                value = serde_json::from_str::<CommandOptionValue>(value_string)
                    .unwrap_or(value)
            }
            options.insert(option, value);
        }
    }
    options
}

fn make_api_response_ok() -> RouilleResponse {
    make_api_response_with_header_and_stats(Ok(serde_json::Value::Null), None, None, None)
}

fn make_api_response_ok_with_result(result: serde_json::Value) -> RouilleResponse {
    make_api_response_with_header_and_stats(Ok(result), None, None, None)
}

fn make_api_response(result: Result<serde_json::Value, HTTPError>) -> RouilleResponse {
    make_api_response_with_header_and_stats(result, None, None, None)
}

fn make_api_response_with_headers(
    result: Result<serde_json::Value, HTTPError>,
    maybe_headers: Option<Vec<(std::borrow::Cow<'static, str>, std::borrow::Cow<'static, str>)>>,
) -> RouilleResponse {
    make_api_response_with_header_and_stats(result, maybe_headers, None, None)
}

fn make_api_response_with_header_and_stats(
    result: Result<serde_json::Value, HTTPError>,
    maybe_headers: Option<Vec<(std::borrow::Cow<'static, str>, std::borrow::Cow<'static, str>)>>,
    maybe_statistics: Option<CommandStats>,
    maybe_status_code: Option<u16>,
) -> RouilleResponse {
    let mut body = json!(
        {
            "ok": if let Some(ref status_code) = maybe_status_code {
                *status_code == 200
            } else {
                result.is_ok()
            }
        }
    );
    body.as_object_mut().unwrap().insert(
        "result".to_string(),
        result
            .clone()
            .or_else::<serde_json::Value, _>(|error| {
                Ok(serde_json::Value::String(error.to_string()))
            })
            .unwrap(),
    );
    if let Some(statistics) = maybe_statistics {
        body.as_object_mut().unwrap().insert(
            "statistics".to_string(),
            serde_json::to_value(&statistics).unwrap(),
        );
    };
    let status_code = if let Some(status_code) = maybe_status_code {
            status_code
        } else if let Err(ref error) = result {
            error.http_status_code()
        } else {
        200
    };
    if let Err(error) = result {
        body.as_object_mut().unwrap().insert(
            "code".to_string(),
            serde_json::Value::Number(serde_json::Number::from(error.http_error_code())),
        );
    };
    let mut response = RouilleResponse::text(serde_json::to_string(&body).unwrap())
        .with_status_code(status_code);
    response.headers.push((
        std::borrow::Cow::Borrowed("Content-Type"),
        std::borrow::Cow::Borrowed("application/json; charset=utf-8"),
    ));
    if let Some(headers) = maybe_headers {
        for (name, value) in headers {
            response.headers.push((name, value));
        }
    };
    response
}

// Old handle_rejection and http_logging functions removed - rouille handles errors differently
