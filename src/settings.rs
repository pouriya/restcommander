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

use thiserror::Error;

use crate::samples;
use crate::utils;

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
    Run(CMDOptRun),
    Sample(CMDSample),
    #[structopt(about = "Prints hex-encoded sha512 of input")]
    Sha512(CMDSha512),
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Reads a .toml configuration file and starts a server")]
pub struct CMDOptCfg {
    #[structopt(parse(from_os_str))]
    config_file: PathBuf,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(about = "Runs a server with commandline options")]
pub struct CMDOptRun {
    #[structopt(long, default_value = "127.0.0.1", env = "RESTCOMMANDER_HOST")]
    pub host: String,
    #[structopt(long, default_value = "1995", env = "RESTCOMMANDER_PORT")]
    pub port: u16,
    #[structopt(long, default_value = "/", env = "RESTCOMMANDER_HTTP_BASE_PATH")]
    pub http_base_path: String,
    #[structopt(long, default_value = "info", env = "RESTCOMMANDER_LOG_LEVEL")]
    pub log_level: CfgLoggingLevelName,
    #[structopt(long, parse(from_os_str), env="RESTCOMMANDER_ROOT_DIRECTORY", default_value=CfgCommandsDefault::root_directory_str())]
    pub root_directory: PathBuf,
    #[structopt(long, env = "RESTCOMMANDER_USERNAME")]
    pub username: Option<String>,
    #[structopt(long, env = "RESTCOMMANDER_PASSWORD")]
    pub password_sha512: Option<String>,
    #[structopt(long, env = "RESTCOMMANDER_PASSWORD_FILE")]
    pub password_file: Option<PathBuf>,
    #[structopt(long, env = "RESTCOMMANDER_STATIC_DIRECTORY")]
    pub static_directory: Option<PathBuf>,
    #[structopt(long, env = "RESTCOMMANDER_ENABLE_PANEL")]
    pub disable_panel: bool,
    #[structopt(long, parse(from_os_str), env = "RESTCOMMANDER_TLS_CERT_PATH")]
    pub tls_cert_file: Option<PathBuf>,
    #[structopt(long, parse(from_os_str), env = "RESTCOMMANDER_TLS_KEY_PATH")]
    pub tls_key_file: Option<PathBuf>,
    #[structopt(long, parse(from_os_str), env = "RESTCOMMANDER_CAPTCHA_FILE")]
    pub captcha_file: Option<PathBuf>,
    #[structopt(long, env = "RESTCOMMANDER_CAPTCHA_CASE_SENSITIVE")]
    pub captcha_case_sensitive: bool,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CfgValue {
    #[serde(default)]
    pub server: CfgServer,
    #[serde(default)]
    pub commands: CfgCommands,
    #[serde(default)]
    pub logging: CfgLogging,
    #[serde(default)]
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
            trace!("{:?} -> {:#?}", path.clone(), config_value.clone())
        } else {
            info!("loaded configuration from {:?}", path.clone())
        };
        trace!("{:#?}", config_value.clone());
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CfgServer {
    #[serde(default = "CfgServerDefault::host")]
    pub host: String,
    #[serde(default = "CfgServerDefault::port")]
    pub port: u16,
    #[serde(default = "CfgServerDefault::http_base_path")]
    pub http_base_path: String,
    #[serde(default = "CfgServerDefault::username")]
    pub username: String,
    #[serde(default = "CfgServerDefault::password_file")]
    pub password_file: PathBuf,
    #[serde(default = "CfgServerDefault::password_sha512")]
    pub password_sha512: String,
    #[serde(default = "CfgServerDefault::tls_cert_file")]
    pub tls_cert_file: Option<PathBuf>,
    #[serde(default = "CfgServerDefault::tls_key_file")]
    pub tls_key_file: Option<PathBuf>,
    pub captcha_file: Option<PathBuf>,
    #[serde(default = "CfgServerDefault::captcha_case_sensitive")]
    pub captcha_case_sensitive: bool,
    #[serde(default = "CfgServerDefault::ip_whitelist")]
    pub ip_whitelist: Vec<String>,
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
            (true, false, false) => return Err(CfgServerCheckError::PasswordOrPasswordFileIsNotSet),
            (false, true, _) => warn!("configuration contains `password` but `username` field is not set. Using `admin` as default username."),
            (false, _, true) => warn!("configuration contains `password_file` but `username` field is not set. Using `admin` as default username."),
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

struct CfgServerDefault {}
impl CfgServerDefault {
    fn host() -> String {
        "127.0.0.1".to_string()
    }
    fn port() -> u16 {
        1995
    }
    fn http_base_path() -> String {
        "/".to_string()
    }
    fn username() -> String {
        String::new()
    }
    fn password_sha512() -> String {
        String::new()
    }
    fn password_file() -> PathBuf {
        PathBuf::new()
    }
    fn tls_cert_file() -> Option<PathBuf> {
        None
    }
    fn tls_key_file() -> Option<PathBuf> {
        None
    }
    fn captcha_file() -> Option<PathBuf> {
        None
    }
    fn captcha_case_sensitive() -> bool {
        false
    }
    fn ip_whitelist() -> Vec<String> {
        Vec::new()
    }
}

impl Default for CfgServer {
    fn default() -> Self {
        Self {
            host: CfgServerDefault::host(),
            port: CfgServerDefault::port(),
            http_base_path: CfgServerDefault::http_base_path(),
            username: CfgServerDefault::username(),
            password_file: CfgServerDefault::password_file(),
            password_sha512: CfgServerDefault::password_sha512(),
            tls_cert_file: CfgServerDefault::tls_cert_file(),
            tls_key_file: CfgServerDefault::tls_key_file(),
            captcha_file: CfgServerDefault::captcha_file(),
            captcha_case_sensitive: CfgServerDefault::captcha_case_sensitive(),
            ip_whitelist: CfgServerDefault::ip_whitelist(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CfgCommands {
    #[serde(default = "CfgCommandsDefault::root_directory")]
    pub root_directory: PathBuf,
}

struct CfgCommandsDefault {}
impl CfgCommandsDefault {
    fn root_directory() -> PathBuf {
        current_dir().unwrap()
    }
    fn root_directory_str<'a>() -> &'a str {
        Box::leak(
            current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .into_boxed_str(),
        )
    }
}

impl Default for CfgCommands {
    fn default() -> Self {
        Self {
            root_directory: CfgCommandsDefault::root_directory(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CfgLogging {
    #[serde(default)]
    pub level_name: CfgLoggingLevelName,
}

impl Default for CfgLogging {
    fn default() -> Self {
        Self {
            level_name: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct CfgWWW {
    #[serde(default = "CfgWWWDefault::static_directory")]
    pub static_directory: PathBuf,
    #[serde(default = "CfgWWWDefault::enabled")]
    pub enabled: bool,
    #[serde(default = "CfgWWWDefault::configuration")]
    pub configuration: HashMap<String, String>,
}

#[derive(Debug, Error)]
enum CfgWWWCheckError {
    #[error("Static directory {directory:?} {message}")]
    StaticDirectory { directory: PathBuf, message: String },
}

struct CfgWWWDefault {}
impl CfgWWWDefault {
    fn static_directory() -> PathBuf {
        PathBuf::new()
    }
    fn enabled() -> bool {
        true
    }
    fn configuration() -> HashMap<String, String> {
        HashMap::new()
    }
}

impl Default for CfgWWW {
    fn default() -> Self {
        Self {
            static_directory: CfgWWWDefault::static_directory(),
            enabled: CfgWWWDefault::enabled(),
            configuration: CfgWWWDefault::configuration(),
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
#[structopt(about = "Script and configuration samples")]
pub enum CMDSample {
    #[structopt(about = "Configuration with default values set.")]
    Config,
    #[structopt(about = "Simple Python sample.")]
    Python,
    #[structopt(about = "Simple Unix shell sample.")]
    Shell,
    #[structopt(about = "Simple Perl sample.")]
    Perl,
    #[structopt(about = "A self-signed private key.")]
    SelfSignedKey,
    #[structopt(about = "A self-signed certificate.")]
    SelfSignedCert,
    #[structopt(about = "A Systemd service file.")]
    SystemdService,
    #[structopt(about = "A script to test service HTTP API status-code and body.")]
    TestScript,
    #[structopt(about = "YAML info of test-script sample.")]
    TestScriptInfo,
}

#[derive(Debug, Clone, StructOpt)]
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

impl TryFrom<CMDOptRun> for Cfg {
    type Error = String;

    fn try_from(command_line_options: CMDOptRun) -> Result<Self, Self::Error> {
        Ok(Cfg {
            config_value: CfgValue {
                server: CfgServer {
                    host: command_line_options.host,
                    port: command_line_options.port,
                    http_base_path: command_line_options.http_base_path,
                    username: command_line_options.username.unwrap_or("".to_string()),
                    password_sha512: command_line_options
                        .password_sha512
                        .unwrap_or("".to_string()),
                    password_file: command_line_options.password_file.unwrap_or(PathBuf::new()),
                    tls_cert_file: command_line_options.tls_cert_file,
                    tls_key_file: command_line_options.tls_key_file,
                    captcha_file: command_line_options.captcha_file,
                    captcha_case_sensitive: command_line_options.captcha_case_sensitive,
                    ip_whitelist: Vec::new(),
                },
                commands: CfgCommands {
                    root_directory: command_line_options.root_directory,
                },
                logging: CfgLogging {
                    level_name: command_line_options.log_level,
                },
                www: CfgWWW {
                    static_directory: command_line_options
                        .static_directory
                        .unwrap_or(PathBuf::new()),
                    enabled: !command_line_options.disable_panel,
                    configuration: CfgWWWDefault::configuration(),
                },
            },
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
        CMDOpt::Sample(sample_name) => Err(samples::maybe_print(sample_name)),
        CMDOpt::Run(options) => Ok(Cfg::try_from(options).map_err(|reason| Some(reason))?),
        CMDOpt::Config(config_file) => Ok(
            Cfg::try_from(config_file.config_file).map_err(|reason| Some(reason.to_string()))?
        ),
    }
}
