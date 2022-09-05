use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::io::ErrorKind;
use std::net::Ipv4Addr;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use base64;
use chrono::Timelike;

use log::{
    debug, error, info, log_enabled, trace,
    Level::{Debug, Trace},
};

use serde_derive::Deserialize;
use serde_json;
use serde_json::json;

use thiserror::Error;

use tokio::sync::mpsc::Receiver;

use warp;
use warp::http::header::{HeaderMap, AUTHORIZATION, LOCATION};
use warp::http::{Response, StatusCode};
use warp::path::Tail;
use warp::reject::Reject;
use warp::{Filter, Rejection};

use crate::captcha;
use crate::cmd;
use crate::cmd::{Command, CommandInput, CommandStats};
use crate::settings::Cfg;
use crate::utils;
use crate::www;

//  for future use for HTTP "Server" header
// use structopt::clap::crate_name;

pub static API_RUN_BASE_PATH: &str = "/api/run";

#[derive(Error, Debug)]
pub enum HTTPError {
    #[error(transparent)]
    Authentication(#[from] HTTPAuthenticationError),
    #[error(transparent)]
    API(#[from] HTTPAPIError),
    #[error(transparent)]
    Captcha(#[from] std::io::Error),
}

impl Reject for HTTPError {}

#[derive(Error, Debug)]
pub enum HTTPAuthenticationError {
    #[error("could not decode authorization header value {data:?} to base64 ({source:?})")]
    Base64Decode {
        data: String,
        source: base64::DecodeError,
    },
    #[error("username or password is not set in server configuration")]
    UsernameOrPasswordIsNotSet,
    #[error("could not found username or password in {data:?}")]
    UsernameOrPasswordIsNotFound { data: String },
    #[error("unknown authentication method {method:?}")]
    UnknownMethod { method: String },
    #[error("invalid basic authentication with header value {header_value:?}")]
    InvalidBasicAuthentication { header_value: String },
    #[error("invalid username or password")]
    InvalidUsernameOrPassword,
    #[error("invalid CAPTCHA")]
    InvalidCaptcha,
    #[error("invalid CAPTCHA form")]
    InvalidCaptchaForm,
    #[error("Could not found HTTP Cookie header")]
    CookieNotFound,
    #[error("HTTP Cookie is expired")]
    CookieExpired,
    #[error("HTTP Cookie is invalid")]
    InvalidCookie,
}

#[derive(Error, Debug)]
pub enum HTTPAPIError {
    #[error("{message:?}")]
    CommandNotFound { message: String },
    #[error("{message:?}")]
    CheckInput { message: String },
    #[error("{message:?}")]
    InitializeCommand { message: String },
    #[error("{message:?}")]
    ReloadCommands { message: String },
    #[error("{message:?}")]
    ReloadConfig { message: String },
    #[error("Password should not be empty")]
    EmptyPassword,
    #[error("Server configuration does not allow client to change the password")]
    NoPasswordFile,
    #[error("Could not save new password to configured password file ({message:?})")]
    SaveNewPassword { message: String },
}

impl HTTPAPIError {
    fn http_error_code(&self) -> i32 {
        match self {
            // Keep 1001 for Command errors
            Self::CommandNotFound { .. } => 1002,
            Self::CheckInput { .. } => 1003,
            Self::InitializeCommand { .. } => 1004,
            Self::ReloadCommands { .. } => 1005,
            Self::ReloadConfig { .. } => 1006,
            Self::EmptyPassword => 1007,
            Self::NoPasswordFile => 1008,
            Self::SaveNewPassword { .. } => 1010,
        }
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            Self::CommandNotFound { .. } => StatusCode::NOT_FOUND,
            Self::CheckInput { .. } => StatusCode::BAD_REQUEST,
            Self::InitializeCommand { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ReloadCommands { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ReloadConfig { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::EmptyPassword => StatusCode::BAD_REQUEST,
            Self::NoPasswordFile => StatusCode::SERVICE_UNAVAILABLE,
            Self::SaveNewPassword { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl HTTPAuthenticationError {
    fn http_error_code(&self) -> i32 {
        match self {
            // keep 2001 for "Authentication required"
            Self::Base64Decode { .. } => 2002,
            Self::UsernameOrPasswordIsNotFound { .. } => 2003,
            Self::UsernameOrPasswordIsNotSet { .. } => 2004,
            Self::UnknownMethod { .. } => 2005,
            Self::InvalidBasicAuthentication { .. } => 2006,
            Self::InvalidUsernameOrPassword => 2007,
            Self::InvalidCaptcha => 2008,
            Self::InvalidCaptchaForm => 2009,
            Self::CookieNotFound => 2010,
            Self::CookieExpired => 2011,
            Self::InvalidCookie => 2012,
        }
    }

    fn http_status_code(&self) -> StatusCode {
        match self {
            Self::Base64Decode { .. } => StatusCode::BAD_REQUEST,
            Self::UsernameOrPasswordIsNotFound { .. } => StatusCode::UNAUTHORIZED,
            Self::UsernameOrPasswordIsNotSet { .. } => StatusCode::CONFLICT,
            Self::UnknownMethod { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidBasicAuthentication { .. } => StatusCode::BAD_REQUEST,
            Self::InvalidUsernameOrPassword => StatusCode::UNAUTHORIZED,
            Self::InvalidCaptcha => StatusCode::UNAUTHORIZED,
            Self::InvalidCaptchaForm => StatusCode::BAD_REQUEST,
            Self::CookieNotFound => StatusCode::UNAUTHORIZED,
            Self::CookieExpired => StatusCode::UNAUTHORIZED,
            Self::InvalidCookie => StatusCode::UNAUTHORIZED,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SetPassword {
    password: String,
}

#[inline]
fn exit_code_to_status_code(exit_code: i32) -> StatusCode {
    match exit_code {
        0 => StatusCode::OK,                    // 200
        1 => StatusCode::INTERNAL_SERVER_ERROR, // 500
        2 => StatusCode::BAD_REQUEST,           // 400
        3 => StatusCode::FORBIDDEN,             // 403
        4 => StatusCode::NOT_FOUND,             // 404
        5 => StatusCode::SERVICE_UNAVAILABLE,   // 503
        6 => StatusCode::NOT_ACCEPTABLE,        // 406
        7 => StatusCode::NOT_IMPLEMENTED,       // 501
        8 => StatusCode::CONFLICT,              // 409
        9 => StatusCode::REQUEST_TIMEOUT,       // 408
        _ => StatusCode::INTERNAL_SERVER_ERROR, // 500
    }
}

pub async fn setup(
    cfg: Arc<RwLock<Cfg>>,
    commands: Arc<RwLock<Command>>,
) -> Result<
    (
        tokio::sync::oneshot::Sender<()>,
        tokio::sync::mpsc::Receiver<()>,
    ),
    String,
> {
    let (http_start_sender, mut http_start_receiver) = tokio::sync::mpsc::channel::<()>(128);
    let (http_stop_sender, http_stop_receiver) = tokio::sync::oneshot::channel::<()>();
    let initialize_channel = http_start_sender.clone();

    let server_options = cfg.read().unwrap().config_value.server.clone();
    let host = server_options.host.clone();
    let port = server_options.port.clone();

    let api_run_filter =
        warp::path("run").and(api_run_command_filter(cfg.clone(), commands.clone()));
    let api_reload_filter =
        warp::path("reload").and(api_reload_commands_filter(commands.clone()).or(
            api_reload_config_filter(cfg.clone(), http_start_sender.clone()),
        ));
    let maybe_captcha = if cfg
        .read()
        .unwrap()
        .config_value
        .server
        .captcha_file
        .is_some()
    {
        Some(Arc::new(RwLock::new(
            captcha::Captcha::try_from(
                cfg.read()
                    .unwrap()
                    .config_value
                    .server
                    .captcha_file
                    .as_ref()
                    .unwrap()
                    .clone(),
            )
            .map_err(|error| error.to_string())?,
        )))
    } else {
        None
    };
    let tokens = Arc::new(RwLock::new(HashMap::new()));
    let api_filter = warp::path("api")
        .and(
            warp::path("auth").and(
                authentication_filter(cfg.clone(), tokens.clone(), maybe_captcha.clone(), true)
                    .untuple_one()
                    .and(api_auth_filter(tokens.clone()))
            )
                .or(
                    authentication_filter(cfg.clone(), tokens.clone(), maybe_captcha.clone(), false).untuple_one().and(
                        api_run_filter
                            .or(api_reload_filter)
                            .or(api_get_commands_filter(commands.clone()))
                            .or(api_set_password_filter(cfg.clone()))
                            .or(api_test_auth()),
                    )
                )
        );
    let static_filter = warp::path("static").and(
        static_index_html_filter(cfg.clone())
            .or(static_external_filter(cfg.clone()).or(static_internal_filter(cfg.clone()))),
    );
    let dynamic_captcha = warp::path("dynamic").and(captcha_filter(maybe_captcha.clone()));

    let routes = api_filter
        .or(static_filter)
        .or(redirect_root_to_index_html_filter(cfg.clone()))
        .or(dynamic_captcha)
        .recover(handle_rejection)
        .with(warp::log::custom(http_logging));
    if server_options.tls_cert_file.clone().is_some()
        && server_options.tls_key_file.clone().is_some()
    {
        let server = warp::serve(routes)
            .tls()
            .cert_path(server_options.tls_cert_file.clone().unwrap())
            .key_path(server_options.tls_key_file.clone().unwrap());
        tokio::spawn(async move {
            debug!(
                "Attempt to start HTTPS server on {}:{} with cert file {:?} and key file {:?}",
                host,
                port,
                server_options.tls_cert_file.clone().unwrap(),
                server_options.tls_key_file.clone().unwrap()
            );
            let (_, server) = server.bind_with_graceful_shutdown(
                (host.parse::<Ipv4Addr>().unwrap(), port),
                async {
                    http_stop_receiver.await.ok();
                },
            );
            initialize_channel.send(()).await.unwrap();
            server.await;
            info!("stopped HTTPS listener on {}:{}", host, port);
        });
    } else {
        let server = warp::serve(routes);
        tokio::spawn(async move {
            debug!("Attempt to start HTTP server on {}:{}", host, port);
            let (_, server) = server.bind_with_graceful_shutdown(
                (host.parse::<Ipv4Addr>().unwrap(), port),
                async {
                    http_stop_receiver.await.ok();
                },
            );
            initialize_channel.send(()).await.unwrap();
            server.await;
            info!("stopped HTTP listener on {}:{}", host, port);
        });
    };
    match utils::maybe_receive(&mut http_start_receiver, 5, "http-handler".to_string()).await {
        Ok(Some(())) => Ok(()),
        Ok(None) => {
            Err("could not receive HTTP server ack after initialization after 5s".to_string())
        }
        Err(reason) => Err(reason),
    }?;
    info!(
        "Started HTTP server on {}:{} with base path {:?}",
        server_options.host, server_options.port, server_options.http_base_path
    );
    Ok((http_stop_sender, http_start_receiver))
}

pub async fn maybe_handle_message(channel_receiver: &mut Receiver<()>) -> Result<bool, String> {
    match utils::maybe_receive(channel_receiver, 1, "http handler".to_string()).await {
        Ok(None) => Ok(false),
        Ok(_) => Ok(true), // Ok(Some(())
        Err(reason) => Err(reason),
    }
}

fn authentication_filter(
    cfg: Arc<RwLock<Cfg>>,
    tokens: Arc<RwLock<HashMap<String, u32>>>,
    maybe_captcha: Option<Arc<RwLock<captcha::Captcha>>>,
    allow_basic_auth: bool,
) -> impl Filter<Extract = ((),), Error = Rejection> + Clone {
    warp::header::<String>(warp::http::header::AUTHORIZATION.as_str())
        .or(
            warp::any().map(|| String::new())
        )
        .unify()
        .and(
            warp::post()
                .or(warp::any())
                .unify()
                .and(warp::body::form::<HashMap<String, String>>()
                .or(warp::any().map(|| HashMap::new()))
                .unify()),
        )
        .and(
            warp::cookie::cookie("token")
                .or(warp::any().map(|| "".to_string()))
                .unify()
        )
        .and_then(
            move |authorization_value: String, form: HashMap<String, String>, cookie: String| {
                let cfg = cfg.clone();
                let tokens = tokens.clone();
                let maybe_captcha = maybe_captcha.clone();
                async move {
                    match try_authenticate(
                        cfg,
                        tokens,
                        maybe_captcha,
                        authorization_value,
                        form,
                        cookie,
                        allow_basic_auth,
                    ) {
                        Err(reason) => Err(warp::reject::custom(HTTPError::Authentication(reason))),
                        Ok(_) => Ok(()),
                    }
                }
            },
        )
}

fn api_get_commands_filter(
    commands: Arc<RwLock<Command>>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::get().and(warp::path("commands")).then(move || {
        let commands = commands.clone();
        async move {
            make_api_response_ok_with_result(
                serde_json::to_value(commands.read().unwrap().deref()).unwrap(),
            )
        }
    })
}

fn api_run_command_filter(
    cfg: Arc<RwLock<Cfg>>,
    commands: Arc<RwLock<Command>>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::post()
        .map(move || (cfg.clone(), commands.clone()))
        .and(warp::path::tail())
        .and(warp::body::json())
        .and_then(
            move |state: (Arc<RwLock<Cfg>>, Arc<RwLock<Command>>),
                  tail: Tail,
                  command_input: CommandInput| {
                async move {
                    match maybe_run_command(
                        state.0,
                        state.1,
                        tail.as_str().to_string(),
                        command_input,
                    ) {
                        Err(reason) => Err(warp::reject::custom(HTTPError::API(reason))),
                        Ok(response) => Ok(response),
                    }
                }
            },
        )
}

fn api_reload_commands_filter(
    commands: Arc<RwLock<Command>>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::get().and(warp::path("commands")).and_then(move || {
        let commands = commands.clone();
        async move {
            if let Err(reason) = commands.write().unwrap().reload() {
                return Err(warp::reject::custom(HTTPError::API(
                    HTTPAPIError::ReloadCommands {
                        message: reason.to_string(),
                    },
                )));
            };
            Ok(make_api_response_ok())
        }
    })
}

fn api_reload_config_filter(
    cfg: Arc<RwLock<Cfg>>,
    http_notify_channel: tokio::sync::mpsc::Sender<()>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::get().and(warp::path("config")).and_then(move || {
        let cfg = cfg.clone();
        let http_notify_channel = http_notify_channel.clone();
        async move {
            if let Err(reason) = cfg.write().unwrap().try_reload() {
                return Err(warp::reject::custom(HTTPError::API(
                    HTTPAPIError::ReloadConfig {
                        message: reason.to_string(),
                    },
                )));
            };
            http_notify_channel.send(()).await.unwrap();
            Ok(make_api_response_ok())
        }
    })
}

fn api_set_password_filter(
    cfg: Arc<RwLock<Cfg>>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::post()
        .and(warp::path("setPassword"))
        .map(move || cfg.clone())
        .and(warp::body::json())
        .and_then(
            move |cfg: Arc<RwLock<Cfg>>, password: SetPassword| async move {
                match try_set_password(cfg, password) {
                    Err(reason) => Err(warp::reject::custom(HTTPError::API(reason))),
                    Ok(response) => Ok(response),
                }
            },
        )
}

fn api_test_auth() -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::get()
        .and(warp::path("testAuth"))
        .then(|| async move { make_api_response_ok() })
}

fn api_auth_filter(
    tokens: Arc<RwLock<HashMap<String, u32>>>,
) -> impl Filter<Extract = (Response<String>,), Error = Infallible> + Clone {
    warp::any()
        .then(
            move || {
                let token = {
                    let username = uuid::Uuid::new_v4().to_string();
                    let password = uuid::Uuid::new_v4().to_string();
                    base64::encode(format!("{}:{}", username, password))
                };
                let timestamp = chrono::Local::now().second() + 1000;
                tokens.clone().write().unwrap().insert(token.clone(), timestamp);
                async move {
                    make_api_response_with_headers(
                        serde_json::json!({"token": token}),
                        StatusCode::OK,
                        Some(
                            {
                                let mut headers = warp::http::HeaderMap::new();
                                headers.insert(warp::http::header::SET_COOKIE, format!("token={}; Path=/; HttpOnly; Max-Age=1000", token).parse().unwrap());
                                headers
                            }
                        ),
                        None
                    )
                }
            }
        )
}

fn static_index_html_filter(
    cfg: Arc<RwLock<Cfg>>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::get()
        .and(warp::path::path("index.html"))
        .then(move || {
            let cfg = cfg.clone();
            async move { maybe_read_index_html_file(cfg) }
        })
}

fn redirect_root_to_index_html_filter(
    cfg: Arc<RwLock<Cfg>>,
) -> impl Filter<Extract = (Response<String>,), Error = Rejection> + Clone {
    warp::get().and(warp::path::end()).then(move || {
        let cfg = cfg.clone();
        async move {
            let cfg_value = cfg.read().unwrap().config_value.clone();
            if cfg_value.www.enabled {
                Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header(
                        LOCATION,
                        format!("{}static/index.html", cfg_value.server.http_base_path),
                    )
                    .body(String::new())
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body("<html><body>Service Unavailable!</body></html>".into())
                    .unwrap()
            }
        }
    })
}

fn static_external_filter(
    cfg: Arc<RwLock<Cfg>>,
) -> impl Filter<Extract = (warp::fs::File,), Error = Rejection> + Clone {
    let static_directory = cfg
        .clone()
        .read()
        .unwrap()
        .config_value
        .www
        .static_directory
        .clone();
    warp::get()
        .and_then(move || {
            let cfg = cfg.clone();
            async move {
                let www_cfg = cfg.read().unwrap().config_value.www.clone();
                if www_cfg.enabled && www_cfg.static_directory.is_dir() {
                    Ok(())
                } else {
                    Err(warp::reject::not_found())
                }
            }
        })
        .untuple_one()
        .and(warp::fs::dir(static_directory))
}

fn static_internal_filter(
    cfg: Arc<RwLock<Cfg>>,
) -> impl Filter<Extract = (Response<Vec<u8>>,), Error = Rejection> + Clone {
    warp::get()
        .and_then(move || {
            let cfg = cfg.clone();
            async move {
                if cfg.read().unwrap().config_value.www.enabled {
                    Ok(())
                } else {
                    Err(warp::reject::not_found())
                }
            }
        })
        .untuple_one()
        .and(warp::path::tail())
        .and_then(|tail_path: Tail| async move {
            if let Some(bytes) = www::handle_static(tail_path.as_str().to_string()) {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(bytes)
                    .unwrap())
            } else {
                Err(warp::reject::not_found())
            }
        })
}

fn captcha_filter(
    maybe_captcha: Option<Arc<RwLock<captcha::Captcha>>>,
) -> impl Filter<Extract = (Response<Vec<u8>>,), Error = Rejection> + Clone {
    warp::get().and(warp::path("captcha")).and_then(move || {
        let maybe_captcha_tmp = maybe_captcha.clone();
        async move {
            if let Some(captcha) = maybe_captcha_tmp {
                match captcha.write().unwrap().generate(true) {
                    Ok((_, _, png_image)) => Ok(Response::builder()
                        .header(warp::http::header::CONTENT_TYPE, "image/png")
                        .status(StatusCode::OK)
                        .body(png_image)
                        .unwrap()),
                    Err(error) => Err(warp::reject::custom(HTTPError::Captcha(error))),
                }
            } else {
                Err(warp::reject::custom(HTTPError::Captcha(
                    std::io::Error::from(std::io::ErrorKind::Unsupported),
                )))
            }
        }
    })
}

fn try_authenticate(
    cfg: Arc<RwLock<Cfg>>,
    tokens: Arc<RwLock<HashMap<String, u32>>>,
    maybe_captcha: Option<Arc<RwLock<captcha::Captcha>>>,
    authorization_value: String,
    form: HashMap<String, String>,
    cookie: String,
    allow_basic_auth: bool,
) -> Result<(), HTTPAuthenticationError> {
    debug!("allow_basic_auth={}", allow_basic_auth);
    let server_cfg = cfg.read().unwrap().config_value.server.clone();
    if server_cfg.password_sha512.is_empty() && server_cfg.username.is_empty() {
        return Ok(());
    };
    if server_cfg.password_sha512.is_empty() || server_cfg.username.is_empty() {
        return Err(HTTPAuthenticationError::UsernameOrPasswordIsNotSet);
    };
    if !allow_basic_auth {
        if cookie.is_empty() {
            return Err(HTTPAuthenticationError::CookieNotFound)
        };
        return if let Some(expire_time) = tokens.clone().read().unwrap().get(cookie.as_str()) {
            if expire_time > &chrono::Local::now().second() {
                return Ok(())
            };
            Err(HTTPAuthenticationError::CookieExpired)
        } else {
            Err(HTTPAuthenticationError::InvalidCookie)
        }
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
                        debug!(
                            "client password encoded in sha512 for username {:?}: {}",
                            username, password_sha512
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
                                debug!("{}={}", key, value);
                                if maybe_captcha
                                    .unwrap()
                                    .write()
                                    .unwrap()
                                    .compare_and_update(key.to_string(), value)
                                    .map_err(|_error| HTTPAuthenticationError::InvalidCaptcha {})?
                                {
                                    Ok(())
                                } else {
                                    Err(HTTPAuthenticationError::InvalidCaptcha {})
                                }
                            } else {
                                Err(HTTPAuthenticationError::InvalidCaptchaForm {})
                            };
                        };
                    } else {
                        debug!("client authenticates with unknown username {}", username);
                    };
                    return Err(HTTPAuthenticationError::InvalidUsernameOrPassword);
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

fn maybe_run_command(
    cfg: Arc<RwLock<Cfg>>,
    commands: Arc<RwLock<Command>>,
    command_path: String,
    command_input: CommandInput,
) -> Result<Response<String>, HTTPAPIError> {
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
    let env_map = make_environment_variables_map(cfg.clone());
    let command_output = cmd::run_command(&command, &input, env_map).map_err(|reason| {
        HTTPAPIError::InitializeCommand {
            message: reason.to_string(),
        }
    })?;
    let http_status_code = exit_code_to_status_code(command_output.exit_code);
    let http_response_body = if command_output.stdout.is_empty() {
        serde_json::Value::Null
    } else if command_output.decoded_stdout.is_err() {
        serde_json::Value::String(command_output.stdout)
    } else {
        command_output.decoded_stdout.unwrap()
    };
    Ok(make_api_response_with_stats(
        http_response_body,
        http_status_code,
        if command_input.statistics {
            Some(command_output.stats)
        } else {
            None
        },
        if http_status_code == StatusCode::OK {
            None
        } else {
            Some(1001)
        },
    ))
}

fn maybe_read_index_html_file(cfg: Arc<RwLock<Cfg>>) -> Response<String> {
    let cfg_value = cfg.read().unwrap().config_value.clone();
    let (status_code, mut body) = if !cfg_value.www.enabled {
        (
            StatusCode::FORBIDDEN,
            "<html><body>Service Unavailable</body></html>".to_string(),
        )
    } else if cfg_value.www.static_directory.to_str().unwrap().is_empty() {
        (StatusCode::OK, www::get_index_html())
    } else {
        let filename = cfg_value.www.static_directory.join("index.html");
        match fs::read_to_string(filename.clone()) {
            Ok(data) => (StatusCode::OK, data),
            Err(reason) if reason.kind() == ErrorKind::NotFound => {
                (StatusCode::OK, www::get_index_html())
            }
            Err(reason) => {
                error!("could not read file {:?}: {:?}", filename, reason);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "<html><body>Service Unavailable</body></html>".to_string(),
                )
            }
        }
    };
    if status_code == StatusCode::OK {
        body = if !cfg_value.server.http_base_path.is_empty() {
            body.replace("{{BASE_PATH}}", cfg_value.server.http_base_path.as_str())
        } else {
            body.replace("{{BASE_PATH}}", "/")
        };
        body = body.replace("{{TITLE}}", cfg_value.www.html_title.as_str())
    };
    Response::builder().status(status_code).body(body).unwrap()
}

fn make_environment_variables_map(cfg: Arc<RwLock<Cfg>>) -> HashMap<String, String> {
    let cfg_instance = cfg.read().unwrap().config_value.clone();
    HashMap::from(
        [
            (
                "SERVER_HOST".to_string(),
                if cfg_instance.server.host.as_str() == "0.0.0.0" {
                    "127.0.0.1".to_string()
                } else {
                    cfg_instance.server.host.clone()
                },
            ),
            (
                "SERVER_POST".to_string(),
                cfg_instance.server.port.to_string(),
            ),
            (
                "SERVER_HTTP_BASE_PATH".to_string(),
                cfg_instance.server.http_base_path,
            ),
            ("SERVER_USERNAME".to_string(), cfg_instance.server.username),
            (
                "COMMANDS_ROOT_DIRECTORY".to_string(),
                cfg_instance
                    .commands
                    .root_directory
                    .to_str()
                    .unwrap()
                    .to_string(),
            ),
            (
                "LOGGING_LEVEL_NAME".to_string(),
                cfg_instance.logging.level_name.to_log_level().to_string(),
            ),
            // ("FILENAME".to_string(), cfg.read().unwrap().filename.),
        ]
        .map(|item| (format!("RESTCOMMANDER_CONFIG_{}", item.0), item.1)),
    )
}

fn try_set_password(
    cfg: Arc<RwLock<Cfg>>,
    password: SetPassword,
) -> Result<Response<String>, HTTPAPIError> {
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

fn make_api_response_ok() -> Response<String> {
    make_api_response_with_header_and_stats(
        serde_json::Value::Null,
        StatusCode::OK,
        None,
        None,
        None,
    )
}

fn make_api_response_ok_with_result(result: serde_json::Value) -> Response<String> {
    make_api_response_with_header_and_stats(result, StatusCode::OK, None, None, None)
}

fn make_api_response(
    result_or_reason: serde_json::Value,
    status_code: StatusCode,
    maybe_error_code: Option<i32>,
) -> Response<String> {
    make_api_response_with_header_and_stats(
        result_or_reason,
        status_code,
        None,
        None,
        maybe_error_code,
    )
}

fn make_api_response_with_stats(
    result_or_reason: serde_json::Value,
    status_code: StatusCode,
    maybe_statistics: Option<CommandStats>,
    maybe_error_code: Option<i32>,
) -> Response<String> {
    make_api_response_with_header_and_stats(
        result_or_reason,
        status_code,
        None,
        maybe_statistics,
        maybe_error_code,
    )
}

fn make_api_response_with_headers(
    result_or_reason: serde_json::Value,
    status_code: StatusCode,
    maybe_headers: Option<HeaderMap>,
    maybe_error_code: Option<i32>,
) -> Response<String> {
    make_api_response_with_header_and_stats(
        result_or_reason,
        status_code,
        maybe_headers,
        None,
        maybe_error_code,
    )
}

fn make_api_response_with_header_and_stats(
    result_or_reason: serde_json::Value,
    status_code: StatusCode,
    maybe_headers: Option<HeaderMap>,
    maybe_statistics: Option<CommandStats>,
    maybe_error_code: Option<i32>,
) -> Response<String> {
    let mut body = json!(
        {"ok": if status_code == StatusCode::OK {true} else {false}}
    );
    if !result_or_reason.is_null() {
        body.as_object_mut().unwrap().insert(
            if status_code == StatusCode::OK {
                "result"
            } else {
                "reason"
            }
            .to_string(),
            result_or_reason,
        );
    };
    if let Some(statistics) = maybe_statistics {
        body.as_object_mut().unwrap().insert(
            "statistics".to_string(),
            serde_json::to_value(&statistics).unwrap(),
        );
    };
    let mut response = warp::http::Response::builder().status(status_code);
    let headers_mut = response.headers_mut().unwrap();
    if let Some(headers) = maybe_headers {
        for (header, header_value) in headers.iter() {
            headers_mut.insert(header.clone(), header_value.clone());
        }
    };
    if let Some(error_code) = maybe_error_code {
        body.as_object_mut()
            .unwrap()
            .insert("code".to_string(), serde_json::Value::from(error_code));
    };
    headers_mut.insert(
        warp::http::header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    response
        .body(serde_json::to_string(&body).unwrap())
        .unwrap()
}

async fn handle_rejection(rejection: Rejection) -> Result<Response<String>, Rejection> {
    let response = if let Some(HTTPError::API(api_error)) = rejection.find::<HTTPError>() {
        make_api_response(
            serde_json::Value::from(api_error.to_string()),
            api_error.http_status_code(),
            Some(api_error.http_error_code()),
        )
    } else if let Some(HTTPError::Authentication(authentication_error)) =
        rejection.find::<HTTPError>()
    {
        let status_code = authentication_error.http_status_code();
        let mut headers = HeaderMap::new();
        if status_code == StatusCode::UNAUTHORIZED {
            headers.insert(
                "WWW-Authenticate",
                "Basic realm=\"Restricted\", charset=\"UTF-8\""
                    .parse()
                    .unwrap(),
            );
        };
        make_api_response_with_headers(
            serde_json::Value::from(authentication_error.to_string()),
            status_code,
            Some(headers),
            Some(authentication_error.http_error_code()),
        )
    } else if let Some(body_deserialize_error) =
        rejection.find::<warp::filters::body::BodyDeserializeError>()
    {
        make_api_response(
            serde_json::Value::from(body_deserialize_error.to_string()),
            StatusCode::BAD_REQUEST,
            Some(2000),
        )
    } else if let Some(missing_header) = rejection.find::<warp::reject::MissingHeader>() {
        if missing_header.name() == AUTHORIZATION.as_str() {
            let mut headers = HeaderMap::new();
            headers.insert(
                "WWW-Authenticate",
                "Basic realm=\"Restricted\", charset=\"UTF-8\""
                    .parse()
                    .unwrap(),
            );
            make_api_response_with_headers(
                serde_json::Value::from("Authentication required"),
                StatusCode::UNAUTHORIZED,
                Some(headers),
                Some(2001),
            )
        } else {
            make_api_response(
                serde_json::Value::from(missing_header.to_string()),
                StatusCode::BAD_REQUEST,
                Some(2000),
            )
        }
    } else if let Some(method_not_allowed) = rejection.find::<warp::reject::MethodNotAllowed>() {
        make_api_response(
            serde_json::Value::from(method_not_allowed.to_string()),
            StatusCode::METHOD_NOT_ALLOWED,
            Some(2000),
        )
    } else if let Some(HTTPError::Captcha(captcha_error)) = rejection.find::<HTTPError>() {
        Response::builder()
            .status(StatusCode::NOT_ACCEPTABLE)
            .body(captcha_error.to_string())
            .unwrap()
    } else {
        if !rejection.is_not_found() {
            error!("unhandled rejection {:?}", rejection);
        };
        return Err(rejection);
    };
    if log_enabled!(Trace) {
        trace!("Made {:?}", response);
    } else if log_enabled!(Debug) {
        debug!(
            "Made response with status code {:?} and body {:?}",
            response.status(),
            response.body()
        )
    };
    Ok(response)
}

fn http_logging(info: warp::log::Info) {
    let elapsed = info.elapsed().as_micros() as f64 / 1000000.0;
    if log_enabled!(Trace) {
        trace!(
            "New request from {:?} for {:?} with headers {:?} handled in {}s",
            info.remote_addr().unwrap(),
            info.path(),
            info.request_headers(),
            elapsed
        )
    } else if log_enabled!(Debug) {
        debug!(
            "New request from {:?} for {:?} handled in {}s",
            info.remote_addr().unwrap(),
            info.path(),
            elapsed
        )
    } else {
        info!(
            "{:?} | {:?} -> {:?} ({}s)",
            info.remote_addr().unwrap(),
            info.path(),
            info.status(),
            elapsed
        )
    }
}
