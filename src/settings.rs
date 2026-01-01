use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;

use clap::Parser;

use crate::cmd::runner::CommandOptionsValue;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct CommandLine {
    /// HTTP server listen address.
    #[arg(long, default_value = "127.0.0.1", env = "RESTCOMMANDER_SERVER_HOST", value_parser = parse_ip_addr)]
    pub host: String,

    /// HTTP server listen port number.
    #[arg(long, default_value = "1995", env = "RESTCOMMANDER_SERVER_PORT")]
    pub port: u16,

    /// HTTP server base path. Currently not used!
    #[arg(long, default_value = "/", env = "RESTCOMMANDER_SERVER_HTTP_BASE_PATH", value_parser = parse_http_base_path)]
    pub http_base_path: String,

    /// HTTP server basic authentication username.
    ///
    /// You can use this `username` and configured password to get a new bearer token.
    /// If the value is empty and no password is configured, then no authentication
    /// is needed for anything. If the value is empty and password is configured, the
    /// username will be `admin`.
    #[arg(long, default_value = "", env = "RESTCOMMANDER_SERVER_USERNAME")]
    pub username: String,

    /// A file containing sha512 of your user password.
    ///
    /// By configuring this you are able to change the password in runtime via REST API.
    /// Make sure that RestCommander process has appropriate permissions to write to the file.
    /// Empty value means this option should be discarded and if one of server `password_file`
    /// and `password_sha512` is not configured, You can call every REST API endpoint without
    /// authentication.
    #[arg(long, env = "RESTCOMMANDER_SERVER_PASSWORD_FILE", value_parser = parse_password_file)]
    pub password_file: Option<PathBuf>,

    /// sha512 of you user password.
    ///
    /// If server `password_file` is configured, this is discarded.
    /// Note that by configuring this, You can not change the password via REST API or in
    /// web dashboard.
    /// Empty value means this option should be discarded and if one of server `password_file`
    /// and `password_sha512` is not configured, You can call every REST API endpoint without
    /// authentication.
    #[arg(long, env = "RESTCOMMANDER_SERVER_PASSWORD_SHA512")]
    pub password_sha512: Option<String>,

    /// HTTP server TLS certificate file.
    ///
    /// If you configure this along with server `tls_key_file` option, RestCommander
    /// serves everything over HTTPS.
    #[arg(long, env = "RESTCOMMANDER_SERVER_TLS_CERT_FILE", value_parser = parse_tls_file)]
    pub tls_cert_file: Option<PathBuf>,

    /// HTTP server TLS private-key file.
    ///
    /// If you configure this along with server `tls_cert_file` option, RestCommander
    /// serves everything over HTTPS.
    #[arg(long, env = "RESTCOMMANDER_SERVER_TLS_KEY_FILE", value_parser = parse_tls_file)]
    pub tls_key_file: Option<PathBuf>,

    /// Enable/Disable CAPTCHA.
    #[arg(long, env = "RESTCOMMANDER_SERVER_CAPTCHA")]
    pub captcha: bool,

    /// Make CAPTCHA case-sensitive
    #[arg(long, env = "RESTCOMMANDER_SERVER_CAPTCHA_CASE_SENSITIVE")]
    pub captcha_case_sensitive: bool,

    /// hardcoded HTTP bearer token that does not expire.
    ///
    /// You can use this value in your application(s) then you do not have to pass
    /// CAPTCHA each time the previous token has expired to get a new one.
    #[arg(long, env = "RESTCOMMANDER_SERVER_API_TOKEN")]
    pub api_token: Option<String>,

    /// Timeout for dynamically generated HTTP bearer tokens in seconds.
    ///
    /// The default value is 1 week.
    #[arg(
        long,
        default_value = "604800",
        env = "RESTCOMMANDER_SERVER_TOKEN_TIMEOUT"
    )]
    pub token_timeout: usize,

    /// Root directory to load command files and directories and their information files.
    #[arg(long, env = "RESTCOMMANDER_COMMANDS_ROOT_DIRECTORY", value_parser = parse_commands_root_directory)]
    pub root_directory: PathBuf,

    /// Configuration key/values for commands in KEY=VALUE format (can be specified multiple times).
    #[arg(short = 'C', long, value_name = "KEY=VALUE", value_parser = parse_command_key_value)]
    pub commands_configuration: Vec<(String, crate::cmd::tree::CommandOptionValue)>,

    /// Your scripts will receive below configuration key/values directly from env or stdin.
    #[arg(skip)]
    pub configuration: CommandOptionsValue,

    /// Enable trace level logging (shows target and location).
    #[arg(long)]
    pub trace: bool,

    /// Enable debug level logging (shows target).
    #[arg(long)]
    pub debug: bool,

    /// Disable all logging.
    #[arg(long)]
    pub quiet: bool,

    /// A directory to serve your own web files under `/static/*` HTTP path.
    ///
    /// Also you can override RestCommander virtual files inside this folder.
    /// RestCommander virtual files are: index.html, index.js, login.html,
    /// login.js, commands.html, commands.js, restcommander-background-image.jpg,
    /// favicon.ico, bootstrap.bundle.min.js, bootstrap.min.css, api.js, utils.js.
    #[arg(long, env = "RESTCOMMANDER_WWW_STATIC_DIRECTORY", value_parser = parse_static_directory)]
    pub static_directory: Option<PathBuf>,

    /// Enable/Disable the web dashboard.
    #[arg(long, env = "RESTCOMMANDER_WWW_ENABLED")]
    pub enabled: bool,

    /// Configuration key/values for www in KEY=VALUE format (can be specified multiple times).
    #[arg(short = 'W', long, value_name = "KEY=VALUE", value_parser = parse_key_value)]
    pub www_configuration: Vec<(String, String)>,

    /// You can access below configuration key/values from REST-API `/public/configuration` endpoint.
    #[arg(skip)]
    pub www_configuration_map: HashMap<String, String>,
}

fn parse_ip_addr(s: &str) -> Result<String, String> {
    s.parse::<IpAddr>()
        .map_err(|e| format!("Could not parse hostname {:?}: {}", s, e))?;
    Ok(s.to_string())
}

fn parse_http_base_path(s: &str) -> Result<String, String> {
    if !s.starts_with("/") {
        return Err(format!(
            "Invalid HTTP base-path {:?}: HTTP base path must start with '/'",
            s
        ));
    }
    if !s.ends_with("/") {
        return Err(format!(
            "Invalid HTTP base-path {:?}: should contain '/' at the end",
            s
        ));
    }
    Ok(s.to_string())
}

fn parse_password_file(s: &str) -> Result<PathBuf, String> {
    let mut path = PathBuf::from(s);
    if path.is_relative() {
        path = current_dir()
            .map_err(|e| format!("Could not get current directory: {}", e))?
            .join(path);
    }
    // Note: We don't check if file exists here because it might be created later
    Ok(path)
}

fn parse_tls_file(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.is_file() {
        return Err(format!("TLS file {:?} is not found", path));
    }
    Ok(path)
}

fn parse_command_key_value(
    s: &str,
) -> Result<(String, crate::cmd::tree::CommandOptionValue), String> {
    if let Some(equal_pos) = s.find('=') {
        let key = s[..equal_pos].to_string();
        let value_str = s[equal_pos + 1..].to_string();
        let value: crate::cmd::tree::CommandOptionValue = serde_json::from_str(&value_str)
            .map_err(|e| format!("Invalid JSON value for key {}: {}", key, e))?;
        Ok((key, value))
    } else {
        Err(format!("Invalid KEY=VALUE format: {}", s))
    }
}

fn parse_key_value(s: &str) -> Result<(String, String), String> {
    if let Some(equal_pos) = s.find('=') {
        let key = s[..equal_pos].to_string();
        let value = s[equal_pos + 1..].to_string();
        Ok((key, value))
    } else {
        Err(format!("Invalid KEY=VALUE format: {}", s))
    }
}

fn parse_commands_root_directory(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.is_dir() {
        return Err(format!(
            "Commands root directory {:?} is not a directory or could not be found",
            path
        ));
    }
    Ok(path)
}

fn parse_static_directory(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.exists() {
        return Err(format!("Static directory {:?} does not exists", path));
    }
    if !path.is_dir() {
        return Err(format!("Static directory {:?} is not a directory", path));
    }
    Ok(path)
}

impl CommandLine {
    pub fn logging_level(&self) -> LevelFilter {
        if self.quiet {
            LevelFilter::OFF
        } else if self.trace {
            LevelFilter::TRACE
        } else if self.debug {
            LevelFilter::DEBUG
        } else {
            LevelFilter::INFO
        }
    }

    pub fn after_parse(&mut self) -> Result<(), String> {
        // Handle password_file reading
        if let Some(password_file) = &self.password_file {
            let password = fs::read(password_file)
                .map_err(|e| format!("Could not read password file {:?}: {}", password_file, e))?;
            let password = String::from_utf8(password)
                .map_err(|e| {
                    format!(
                        "Could not decode password file {:?} content to UTF-8: {}",
                        password_file, e
                    )
                })?
                .trim()
                .to_string();
            if password.is_empty() {
                return Err(format!("Password file {:?} is empty!", password_file));
            }
            if self.password_sha512.is_some() {
                tracing::warn!(
                    msg = "Both password and password_file fields are set, ignoring password field",
                );
            }
            self.password_sha512 = Some(password);
        }

        // Handle username/password validation
        match (
            !self.username.is_empty(),
            self.password_sha512.is_some(),
            self.password_file.is_some(),
        ) {
            (true, false, false) => {
                return Err("Configuration contains `username` but `password` or `password_file` field is not set".to_string())
            }
            (false, true, _) => {
                tracing::warn!(
                    msg = "Configuration contains password but username field is not set, using 'admin' as default username",
                );
                self.username = "admin".to_string();
            }
            (false, _, true) => {
                tracing::warn!(
                    msg = "Configuration contains password_file but username field is not set, using 'admin' as default username",
                );
                self.username = "admin".to_string();
            }
            _ => (),
        }

        // Handle TLS file validation
        match (self.tls_cert_file.is_some(), self.tls_key_file.is_some()) {
            (true, false) => {
                return Err("TLS cert file is set but TLS key file is not set".to_string())
            }
            (false, true) => {
                return Err("TLS key file is set but TLS cert file is not set".to_string())
            }
            _ => (),
        }

        // Convert parsed command key-value pairs into HashMap
        let mut config = HashMap::new();
        for (key, value) in &self.commands_configuration {
            config.insert(key.clone(), value.clone());
        }
        self.configuration = config;

        // Convert parsed www key-value pairs into HashMap
        let mut www_config = HashMap::new();
        for (key, value) in &self.www_configuration {
            www_config.insert(key.clone(), value.clone());
        }
        self.www_configuration_map = www_config;

        Ok(())
    }
}

pub fn try_setup() -> Result<CommandLine, String> {
    let mut value = CommandLine::parse();
    value.after_parse()?;
    Ok(value)
}
