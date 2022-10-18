use crate::samples::{maybe_print, CMDSample};
use std::collections::HashMap;
use std::env::current_dir;
use std::fs;
use std::io::Error;
use std::net::{AddrParseError, IpAddr};
use std::path::PathBuf;
use std::string::FromUtf8Error;

use structopt::clap::{crate_authors, crate_description, crate_name, crate_version};
use structopt::StructOpt;

use log::{info, log_enabled, trace, warn, Level, LevelFilter};

use config::{Config, ConfigError, Environment, File};

use warp::http::uri::PathAndQuery;

use serde_derive::{Deserialize, Serialize};

use ttyaskpass::AskPass;

use crate::cmd::runner::CommandOptionsValue;
use thiserror::Error;

use crate::utils;

const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: u16 = 1995;
const DEFAULT_SERVER_HTTP_BASE_PATH: &str = "/";
const DEFAULT_SERVER_USERNAME: &str = "";
const DEFAULT_SERVER_PASSWORD_SHA512: &str = "";
const DEFAULT_SERVER_PASSWORD_FILE: &str = "";
const DEFAULT_SERVER_TOKEN_TIMEOUT: usize = 604800; // 1 week in seconds
const DEFAULT_LOGGING_LEVEL_NAME: &str = "info";
const DEFAULT_WWW_STATIC_DIRECTORY: &str = "";

pub mod defaults {
    use super::*;
    use std::str::FromStr;

    pub mod server {
        use super::*;

        pub fn host_str<'a>() -> &'a str {
            DEFAULT_SERVER_HOST
        }

        pub fn host() -> String {
            host_str().to_string()
        }

        pub fn port_str<'a>() -> &'a str {
            Box::leak(DEFAULT_SERVER_PORT.to_string().into_boxed_str())
        }

        pub fn port() -> u16 {
            u16::from_str(port_str()).unwrap()
        }

        pub fn http_base_path_str<'a>() -> &'a str {
            DEFAULT_SERVER_HTTP_BASE_PATH
        }

        pub fn http_base_path() -> String {
            http_base_path_str().to_string()
        }

        pub fn username_str<'a>() -> &'a str {
            DEFAULT_SERVER_USERNAME
        }

        pub fn username() -> String {
            username_str().to_string()
        }

        pub fn password_file_str<'a>() -> &'a str {
            DEFAULT_SERVER_PASSWORD_FILE
        }

        pub fn password_file() -> PathBuf {
            PathBuf::from(password_file_str())
        }

        pub fn password_sha512_str<'a>() -> &'a str {
            DEFAULT_SERVER_PASSWORD_SHA512
        }

        pub fn password_sha512() -> String {
            password_sha512_str().to_string()
        }

        pub fn tls_cert_file() -> Option<PathBuf> {
            None
        }

        pub fn tls_key_file() -> Option<PathBuf> {
            None
        }

        pub fn captcha_file() -> Option<PathBuf> {
            None
        }

        pub fn captcha_case_sensitive() -> bool {
            false
        }

        pub fn ip_whitelist() -> Vec<String> {
            Vec::new()
        }

        pub fn api_token() -> Option<String> {
            None
        }

        pub fn token_timeout_str<'a>() -> &'a str {
            // 1 week
            Box::leak(DEFAULT_SERVER_TOKEN_TIMEOUT.to_string().into_boxed_str())
        }

        pub fn token_timeout() -> usize {
            usize::from_str(token_timeout_str()).unwrap()
        }
    }

    pub mod commands {
        use super::*;

        pub fn root_directory_str<'a>() -> &'a str {
            Box::leak(
                current_dir()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .into_boxed_str(),
            )
        }

        pub fn root_directory() -> PathBuf {
            PathBuf::from(root_directory_str())
        }

        pub fn configuration() -> CommandOptionsValue {
            HashMap::default()
        }
    }

    pub mod www {
        use super::*;

        pub fn static_directory_str<'a>() -> &'a str {
            DEFAULT_WWW_STATIC_DIRECTORY
        }

        pub fn static_directory() -> PathBuf {
            PathBuf::from(static_directory_str())
        }

        pub fn enabled() -> bool {
            true
        }

        pub fn configuration() -> HashMap<String, String> {
            HashMap::new()
        }
    }

    pub mod logging {
        use super::*;

        pub fn level_name_str<'a>() -> &'a str {
            DEFAULT_LOGGING_LEVEL_NAME
        }

        pub fn level_name() -> CfgLoggingLevelName {
            CfgLoggingLevelName::from_str(level_name_str()).unwrap()
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[
    structopt(
        name = crate_name!(),
        about = crate_description!(),
        version = crate_version!(),
        author = crate_authors!()
    )
]
pub enum CMDOpt {
    Config(CMDOptCfg),
    Playground(CfgValue),
    Sample(CMDSample),
    Sha512(CMDSha512),
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Starts from a .toml configuration file.")]
pub struct CMDOptCfg {
    #[structopt(
        parse(from_os_str),
        about = "A .toml configuration file. To generate a new one, use `sample config` subcommand."
    )]
    config_file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Cfg {
    pub config_value: CfgValue,
    pub filename: Option<PathBuf>,
}

#[derive(Debug, Error)]
pub enum CfgError {
    #[error("Server is started via command-line options and no configuration file is given to reload from.")]
    NoConfigFileGiven,
    #[error("Could not read configuration file {filename:?}: {message:?}")]
    ReadFile {
        filename: PathBuf,
        message: ConfigError,
    },
    #[error("Could not deserialize configuration file {filename:?}: {message:?}")]
    Deserialize {
        filename: PathBuf,
        message: ConfigError,
    },
    #[error("{0}")]
    Check(String),
}

trait CheckValue {
    type Error;
    fn check_value(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Deserialize, Serialize, StructOpt)]
pub struct CfgValue {
    #[serde(default)]
    #[structopt(flatten)]
    pub server: CfgServer,
    #[serde(default)]
    #[structopt(flatten)]
    pub commands: CfgCommands,
    #[serde(default)]
    #[structopt(flatten)]
    pub logging: CfgLogging,
    #[serde(default)]
    #[structopt(flatten)]
    pub www: CfgWWW,
}

impl TryFrom<PathBuf> for CfgValue {
    type Error = CfgError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let mut config_value = Config::builder()
            .add_source(File::from(path.clone()).required(true))
            .add_source(Environment::with_prefix(crate_name!()))
            .build()
            .map_err(|reason| CfgError::ReadFile {
                filename: path.clone(),
                message: reason,
            })?
            .try_deserialize::<CfgValue>()
            .map_err(|reason| CfgError::Deserialize {
                filename: path.clone(),
                message: reason,
            })?;
        config_value
            .check_value()
            .map_err(|reason| CfgError::Check(reason.to_string()))?;
        if log_enabled!(Level::Trace) {
            trace!("{:?} -> {:?}", path.clone(), config_value.clone())
        } else {
            info!("loaded configuration from {:?}", path.clone())
        };
        Ok(config_value)
    }
}

impl CheckValue for CfgValue {
    type Error = CfgError;
    fn check_value(&mut self) -> Result<(), Self::Error> {
        self.server
            .check_value()
            .map_err(|reason| CfgError::Check(reason.to_string()))?;
        // self.commands.check_value()
        //     .map_err(
        //         |reason| {
        //             CfgError::Check(reason.to_string())
        //         }
        //     )?;
        // self.logging.check_value()
        //     .map_err(
        //         |reason| {
        //             CfgError::Check(reason.to_string())
        //         }
        //     )?;
        self.www
            .check_value()
            .map_err(|reason| CfgError::Check(reason.to_string()))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, StructOpt)]
pub struct CfgServer {
    /// HTTP server listen address.
    #[serde(default = "defaults::server::host")]
    #[structopt(
        name = "server-host",
        long,
        default_value = defaults::server::host_str(),
        env = "RESTCOMMANDER_SERVER_HOST",
    )]
    pub host: String,

    /// HTTP server listen port number.
    #[serde(default = "defaults::server::port")]
    #[structopt(
        name = "server-port",
        long,
        default_value = defaults::server::port_str(),
        env = "RESTCOMMANDER_SERVER_PORT",
    )]
    pub port: u16,

    /// HTTP server base path. Currently not used!
    #[serde(default = "defaults::server::http_base_path")]
    #[structopt(
        name = "server-http-base-path",
        long,
        default_value =  defaults::server::http_base_path_str(),
        env = "RESTCOMMANDER_SERVER_HTTP_BASE_PATH",
    )]
    pub http_base_path: String,

    /// HTTP server basic authentication username.
    ///
    /// You can use this `username` and configured password to get a new bearer token.
    /// If the value is empty and no password is configured, then no authentication
    /// is needed for anything. If the value is empty and password is configured, the
    /// username will be `admin`.
    #[serde(default = "defaults::server::username")]
    #[structopt(
        name = "server-username",
        long,
        default_value = defaults::server::username_str(),
        env = "RESTCOMMANDER_SERVER_USERNAME",
    )]
    pub username: String,

    /// A file containing sha512 of your user password.
    ///
    /// By configuring this you are able to change the password in runtime via REST API.
    /// Make sure that RestCommander process has appropriate permissions to write to the file.
    /// Empty value means this option should be discarded and if one of server `password_file`
    /// and `password_sha512` is not configured, You can call every REST API endpoint without
    /// authentication.
    #[serde(default = "defaults::server::password_file")]
    #[structopt(
        name = "server-password-file",
        long,
        default_value = defaults::server::password_file_str(),
        env = "RESTCOMMANDER_SERVER_PASSWORD_FILE"
    )]
    pub password_file: PathBuf,

    /// sha512 of you user password.
    ///
    /// If server `password_file` is configured, this is discarded.
    /// Note that by configuring this, You can not change the password via REST API.
    /// Empty value means this option should be discarded and if one of server `password_file`
    /// and `password_sha512` is not configured, You can call every REST API endpoint without
    /// authentication.
    #[serde(default = "defaults::server::password_sha512")]
    #[structopt(
        name = "server-password-sha512",
        long,
        default_value = defaults::server::password_sha512_str(),
        env = "RESTCOMMANDER_SERVER_PASSWORD_SHA512"
    )]
    pub password_sha512: String,

    /// HTTP server TLS certificate file.
    ///
    /// If you configure this along with server `tls_key_file` option, RestCommander
    /// serves everything over HTTPS.
    /// You can get a test certificate via `sample self-signed-cert` subcommand.
    #[serde(default = "defaults::server::tls_cert_file")]
    #[structopt(
        name = "server-tls-cert-file",
        long,
        parse(from_os_str),
        env = "RESTCOMMANDER_SERVER_TLS_CERT_FILE"
    )]
    pub tls_cert_file: Option<PathBuf>,

    /// HTTP server TLS private-key file.
    ///
    /// If you configure this along with server `tls_cert_file` option, RestCommander
    /// serves everything over HTTPS.
    /// You can get a test private-key via `sample self-signed-key` subcommand.
    #[serde(default = "defaults::server::tls_key_file")]
    #[structopt(
        name = "server-tls-key-file",
        long,
        parse(from_os_str),
        env = "RESTCOMMANDER_SERVER_TLS_KEY_FILE"
    )]
    pub tls_key_file: Option<PathBuf>,

    /// A file for saving captcha id/values.
    ///
    /// Empty value means that no CAPTCHA is required to get a new REST-API bearer token.
    #[structopt(
        name = "server-captcha-file",
        long,
        parse(from_os_str),
        env = "RESTCOMMANDER_SERVER_CAPTCHA_FILE"
    )]
    pub captcha_file: Option<PathBuf>,

    /// Make CAPTCHA case-sensitive
    #[serde(default = "defaults::server::captcha_case_sensitive")]
    #[structopt(
        name = "server-captcha-case-sensitive",
        long,
        env = "RESTCOMMANDER_CAPTCHA_CASE_SENSITIVE"
    )]
    pub captcha_case_sensitive: bool,

    /// List of IP addresses that can interact with REST-API. Wildcard characters like *
    /// are allowed.
    ///
    /// No value means everyone can interact with REST-API.
    /// RestCommander currently does not support HTTP IP headers, So this IP address
    /// is the connected client IP address and not the IP address that upstream webserver
    /// forwards in the request header.
    #[serde(default = "defaults::server::ip_whitelist")]
    #[structopt(
        name = "server-ip-whitelist",
        long,
        env = "RESTCOMMANDER_CAPTCHA_CASE_SENSITIVE"
    )]
    pub ip_whitelist: Vec<String>,

    /// hardcoded HTTP bearer token that does not expire.
    ///
    /// You can use this value in your application(s) then you do not have to pass
    /// CAPTCHA each time the previous token has expired to get a new one.
    #[serde(default = "defaults::server::api_token")]
    #[structopt(
        name = "server-api-token",
        long,
        env = "RESTCOMMANDER_SERVER_API_TOKEN"
    )]
    pub api_token: Option<String>,

    /// Timeout for dynamically generated HTTP bearer tokens in seconds.
    ///
    /// The default value is 1 week.
    #[serde(default = "defaults::server::token_timeout")]
    #[structopt(
        name = "server-token-timeout",
        long,
        default_value = defaults::server::token_timeout_str(),
        env = "RESTCOMMANDER_SERVER_TOKEN_TIMEOUT",
    )]
    pub token_timeout: usize,
}

#[derive(Debug, Error)]
pub enum CfgServerCheckError {
    #[error("Could not parse hostname {host:?}: {message:?}")]
    Host {
        host: String,
        message: AddrParseError,
    },
    #[error("Invalid HTTP base-path {http_base_path:?}: {message:?}")]
    HTTPBasePATH {
        http_base_path: String,
        message: String,
    },
    #[error(
        "Configuration contains `username` but `password` or `password_file` field is not set"
    )]
    PasswordOrPasswordFileIsNotSet,
    #[error("Could not read password file {filename:?}: {message:?}")]
    ReadPasswordFile { filename: PathBuf, message: Error },
    #[error("Could not decode password file {filename:?} content to UTF-8: {message:?}")]
    DecodePasswordFileContent {
        filename: PathBuf,
        message: FromUtf8Error,
    },
    #[error("Password file {filename:?} is empty!")]
    PasswordFileEmpty { filename: PathBuf },
    #[error("TLS cert file {filename:?} is not found")]
    TLSCertFileNotFound { filename: PathBuf },
    #[error("TLS key file {filename:?} is not found")]
    TLSKeyFileNotFound { filename: PathBuf },
    #[error("TLS key file is set but TLS cert file is not set")]
    TLSCertFileISNotSet,
    #[error("TLS cert file is set but TLS key file is not set")]
    TLSKeyFileISNotSet,
}

impl CheckValue for CfgServer {
    type Error = CfgServerCheckError;
    fn check_value(&mut self) -> Result<(), Self::Error> {
        self.host
            .clone()
            .parse::<IpAddr>()
            .map_err(|reason| CfgServerCheckError::Host {
                host: self.host.clone(),
                message: reason,
            })?;
        if let Err(reason) = PathAndQuery::try_from(self.http_base_path.clone()) {
            return Err(CfgServerCheckError::HTTPBasePATH {
                http_base_path: self.http_base_path.clone(),
                message: reason.to_string(),
            });
        };
        if !self.http_base_path.clone().ends_with("/") {
            return Err(CfgServerCheckError::HTTPBasePATH {
                http_base_path: self.http_base_path.clone(),
                message: "should contain '/' at the end".to_string(),
            });
        };
        if !self.http_base_path.clone().starts_with("/") {
            return Err(CfgServerCheckError::HTTPBasePATH {
                http_base_path: self.http_base_path.clone(),
                message: "should contain '/' at the start".to_string(),
            });
        };
        match (
            !self.username.is_empty(),
            !self.password_sha512.is_empty(),
            !self.password_file.to_str().unwrap().is_empty(),
        ) {
            (true, false, false) => {
                return Err(CfgServerCheckError::PasswordOrPasswordFileIsNotSet)
            }
            (false, true, _) => {
                warn!("configuration contains `password` but `username` field is not set. Using `admin` as default username.");
                self.username = "admin".to_string();
            }
            (false, _, true) => {
                warn!("configuration contains `password_file` but `username` field is not set. Using `admin` as default username.");
                self.username = "admin".to_string();
            }
            _ => (),
        };
        if !self.password_file.to_str().unwrap().is_empty() {
            if self.password_file.is_relative() {
                self.password_file = current_dir().unwrap().join(self.password_file.clone())
            }
            let password = fs::read(self.password_file.clone()).map_err(|reason| {
                CfgServerCheckError::ReadPasswordFile {
                    filename: self.password_file.clone(),
                    message: reason,
                }
            })?;
            let password = String::from_utf8(password)
                .map_err(|reason| CfgServerCheckError::DecodePasswordFileContent {
                    filename: self.password_file.clone(),
                    message: reason,
                })?
                .trim()
                .to_string();
            if password.is_empty() {
                return Err(CfgServerCheckError::PasswordFileEmpty {
                    filename: self.password_file.clone(),
                });
            };
            if !self.password_sha512.is_empty() {
                warn!(
                    "both `password` and `password_file` fields are set. Ignoring `password` field"
                );
            };
            self.password_sha512 = password;
        };
        if self.tls_cert_file.clone().is_some() && self.tls_key_file.is_some() {
            if !self.tls_cert_file.clone().unwrap().is_file() {
                return Err(CfgServerCheckError::TLSCertFileNotFound {
                    filename: self.tls_cert_file.clone().unwrap(),
                });
            };
            if !self.tls_key_file.clone().unwrap().is_file() {
                return Err(CfgServerCheckError::TLSKeyFileNotFound {
                    filename: self.tls_key_file.clone().unwrap(),
                });
            };
        } else if self.tls_cert_file.clone().is_none() && self.tls_key_file.is_some() {
            return Err(CfgServerCheckError::TLSCertFileISNotSet);
        } else if self.tls_key_file.is_none() && self.tls_cert_file.clone().is_some() {
            return Err(CfgServerCheckError::TLSKeyFileISNotSet);
        };
        Ok(())
    }
}

impl Default for CfgServer {
    fn default() -> Self {
        Self {
            host: defaults::server::host(),
            port: defaults::server::port(),
            http_base_path: defaults::server::http_base_path(),
            username: defaults::server::username(),
            password_file: defaults::server::password_file(),
            password_sha512: defaults::server::password_sha512(),
            tls_cert_file: defaults::server::tls_cert_file(),
            tls_key_file: defaults::server::tls_key_file(),
            captcha_file: defaults::server::captcha_file(),
            captcha_case_sensitive: defaults::server::captcha_case_sensitive(),
            ip_whitelist: defaults::server::ip_whitelist(),
            api_token: defaults::server::api_token(),
            token_timeout: defaults::server::token_timeout(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, StructOpt)]
pub struct CfgCommands {
    /// Root directory to load command files and directories and their information files.
    ///
    /// The default value is your current working directory.
    #[serde(default = "defaults::commands::root_directory")]
    #[structopt(
        name = "commands-root-directory",
        long,
        parse(from_os_str),
        default_value=defaults::commands::root_directory_str(),
        env="RESTCOMMANDER_COMMANDS_ROOT_DIRECTORY",
    )]
    pub root_directory: PathBuf,

    /// Your scripts will receive below configuration key/values directly from env or stdin.
    #[serde(default = "defaults::commands::configuration")]
    #[structopt(skip)]
    pub configuration: CommandOptionsValue,
}

impl Default for CfgCommands {
    fn default() -> Self {
        Self {
            root_directory: defaults::commands::root_directory(),
            configuration: defaults::commands::configuration(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, StructOpt)]
pub struct CfgLogging {
    /// Logging level name.
    ///
    /// Possible values: off | error | warning | info | debug | trace
    #[serde(default = "defaults::logging::level_name")]
    #[structopt(
        name = "logging-level-name",
        long,
        default_value = defaults::logging::level_name_str(),
        env = "RESTCOMMANDER_LOGGING_LEVEL_NAME",
    )]
    pub level_name: CfgLoggingLevelName,
}

impl Default for CfgLogging {
    fn default() -> Self {
        Self {
            level_name: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, StructOpt)]
pub struct CfgWWW {
    /// A directory to serve your own web files under `/static/*` HTTP path.
    ///
    /// Also you can override RestCommander virtual files inside this folder.
    /// RestCommander virtual files are: index.html, index.js, login.html,
    /// login.js, commands.html, commands.js, restcommander-background-image.jpg,
    /// favicon.ico, bootstrap.bundle.min.js, bootstrap.min.css, api.js, utils.js.
    #[serde(default = "defaults::www::static_directory")]
    #[structopt(
        name = "www-static-directory",
        long,
        default_value = defaults::www::static_directory_str(),
        env = "RESTCOMMANDER_WWW_STATIC_DIRECTORY",
    )]
    pub static_directory: PathBuf,

    /// Enable/Disable the web dashboard.
    #[serde(default = "defaults::www::enabled")]
    #[structopt(name = "www-enabled", long, env = "RESTCOMMANDER_WWW_ENABLED")]
    pub enabled: bool,

    /// You can access below configuration key/values from REST-API `/public/configuration` endpoint.
    #[serde(default = "defaults::www::configuration")]
    #[structopt(skip)]
    pub configuration: HashMap<String, String>,
}

#[derive(Debug, Error)]
enum CfgWWWCheckError {
    #[error("Static directory {directory:?} {message}")]
    StaticDirectory { directory: PathBuf, message: String },
}

impl Default for CfgWWW {
    fn default() -> Self {
        Self {
            static_directory: defaults::www::static_directory(),
            enabled: defaults::www::enabled(),
            configuration: defaults::www::configuration(),
        }
    }
}

impl CheckValue for CfgWWW {
    type Error = CfgWWWCheckError;
    fn check_value(&mut self) -> Result<(), Self::Error> {
        let static_directory = self.static_directory.clone();
        if static_directory.to_str().unwrap().is_empty() {
            return Ok(());
        };
        if static_directory.exists() {
            if static_directory.is_dir() {
                Ok(())
            } else {
                Err(CfgWWWCheckError::StaticDirectory {
                    directory: static_directory,
                    message: "is not a directory".to_string(),
                })
            }
        } else {
            Err(CfgWWWCheckError::StaticDirectory {
                directory: static_directory,
                message: "does not exists".to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CfgLoggingLevelName {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Off,
}

impl Default for CfgLoggingLevelName {
    fn default() -> Self {
        Self::Info
    }
}

impl std::str::FromStr for CfgLoggingLevelName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str().trim() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "error" => Ok(Self::Error),
            "warning" | "warn" => Ok(Self::Warning),
            "off" => Ok(Self::Off),
            unknown => Err(format!("Unknown log level name {:?}", unknown)),
        }
    }
}

impl CfgLoggingLevelName {
    pub fn to_log_level(&self) -> log::LevelFilter {
        match self {
            Self::Trace => LevelFilter::Trace,
            Self::Debug => LevelFilter::Debug,
            Self::Info => LevelFilter::Info,
            Self::Error => LevelFilter::Error,
            Self::Warning => LevelFilter::Warn,
            Self::Off => LevelFilter::Off,
        }
    }
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Prints hex-encoded sha512 of input")]
pub struct CMDSha512 {
    #[structopt(about = "input to be encoded. If empty, It prompts to ask input.")]
    input: Option<String>,
}

impl Cfg {
    pub fn try_reload(&mut self) -> Result<(), CfgError> {
        let config_value = match self.filename.clone() {
            Some(filename) => CfgValue::try_from(filename),
            None => Err(CfgError::NoConfigFileGiven),
        }?;
        self.config_value = config_value;
        Ok(())
    }
}

impl TryFrom<PathBuf> for Cfg {
    type Error = CfgError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Ok(Cfg {
            config_value: CfgValue::try_from(path.clone())?,
            filename: Some(if path.is_relative() {
                current_dir().unwrap().join(path)
            } else {
                path
            }),
        })
    }
}

impl TryFrom<CfgValue> for Cfg {
    type Error = String;

    fn try_from(value: CfgValue) -> Result<Self, Self::Error> {
        let mut value = value.clone();
        value
            .check_value()
            .map_err(|reason| CfgError::Check(reason.to_string()).to_string())?;
        Ok(Cfg {
            config_value: value,
            filename: None,
        })
    }
}

pub fn try_setup() -> Result<Cfg, Option<String>> {
    match CMDOpt::from_args() {
        CMDOpt::Sha512(CMDSha512 { input: maybe_input }) => {
            let input = if let Some(input) = maybe_input {
                input
            } else {
                AskPass::new([0; 10240])
                    .with_star('*')
                    .askpass("Enter input text: ")
                    .map(|x| String::from_utf8(x.into()).unwrap())
                    .map_err(|reason| {
                        Some(format!("Could not read password: {}", reason.to_string()))
                    })?
            }
            .trim()
            .to_string();
            if input.is_empty() {
                return Err(Some("input is empty!".to_string()));
            };
            println!("{}", utils::to_sha512(input));
            Err(None)
        }
        CMDOpt::Sample(sample_name) => {
            maybe_print(sample_name);
            Err(None)
        }
        CMDOpt::Playground(options) => Ok(Cfg::try_from(options).map_err(|reason| Some(reason))?),
        CMDOpt::Config(config_file) => Ok(
            Cfg::try_from(config_file.config_file).map_err(|reason| Some(reason.to_string()))?
        ),
    }
}
