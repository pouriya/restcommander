use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::RwLock as AsyncRwLock;

use tracing::error;

mod captcha;
mod cmd;
mod http;
mod logging;
mod report;
mod samples;
mod settings;
mod utils;
mod www;

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut logging_state = logging::setup(settings::CfgLogging::default());
    let cfg = match settings::try_setup() {
        Ok(cfg) => Arc::new(RwLock::new(cfg)),
        Err(maybe_error) => {
            if maybe_error.is_none() {
                exit(0)
            } else {
                eprintln!("{}", maybe_error.unwrap());
                exit(1)
            }
        }
    };
    logging::update(
        cfg.read().unwrap().config_value.clone().logging,
        &mut logging_state,
    );
    let cfg = match settings::try_setup() {
        Ok(cfg) => {
            cfg.trace_log();
            Arc::new(RwLock::new(cfg))
        }
        Err(maybe_error) => {
            if maybe_error.is_none() {
                exit(0)
            } else {
                eprintln!("{}", maybe_error.unwrap());
                exit(1)
            }
        }
    };
    let mut cfg_instance = cfg.read().unwrap().config_value.clone();
    let root_directory = cfg_instance.commands.root_directory.clone();
    let commands = Arc::new(RwLock::new(
        cmd::tree::Command::new(
            &root_directory,
            &PathBuf::from(cfg_instance.server.http_base_path.clone()).join(
                PathBuf::from(http::API_RUN_BASE_PATH)
                    .strip_prefix("/")
                    .unwrap(),
            ),
        )
        .map_err(|reason| reason.to_string())?,
    ));
    let report_state = Arc::new(AsyncRwLock::new(
        report::maybe_setup(cfg_instance.logging.clone(), None).await?,
    ));
    let (mut _http_server_sender, mut _http_server_receiver) =
        http::setup(cfg.clone(), commands.clone(), report_state.clone()).await?;
    if cfg_instance.server.print_banner {
        samples::maybe_print(samples::CMDSample::Banner)
    }
    loop {
        match http::maybe_handle_message(&mut _http_server_receiver).await {
            Ok(true) => {
                // Update logging:
                let new_cfg_instance = cfg.write().unwrap().config_value.clone();
                let new_cfg_logging = new_cfg_instance.logging.clone();
                if cfg_instance.logging.level_name != new_cfg_logging.level_name
                    || cfg_instance.logging.output != new_cfg_logging.output
                {
                    logging::update(new_cfg_logging.clone(), &mut logging_state);
                };
                let last_report_state = Some(report_state.read().await.clone());
                if cfg_instance.logging.report != new_cfg_logging.report {
                    match report::maybe_setup(new_cfg_logging.clone(), last_report_state).await {
                        Ok(new_report_state) => {
                            *report_state.write().await = new_report_state;
                        }
                        Err(error) => error!("{:?}", error),
                    }
                }
                let new_commands_root_directory = new_cfg_instance.commands.root_directory.clone();
                if cfg_instance.commands.root_directory != new_commands_root_directory {
                    let load_new_commands = cmd::tree::Command::new(
                        &new_commands_root_directory,
                        &PathBuf::from(new_cfg_instance.server.http_base_path.clone()).join(
                            PathBuf::from(http::API_RUN_BASE_PATH)
                                .strip_prefix("/")
                                .unwrap(),
                        ),
                    );
                    match load_new_commands {
                        Ok(new_commands) => commands.write().unwrap().replace(new_commands),
                        Err(reason) => {
                            error!("{:?}", reason);
                        }
                    };
                };
                let new_server_cfg = new_cfg_instance.server.clone();
                if new_server_cfg.http_base_path != cfg_instance.server.http_base_path {
                    commands
                        .write()
                        .unwrap()
                        .replace_http_base_path(&PathBuf::from(
                            new_server_cfg.http_base_path.clone(),
                        ));
                };
                if new_server_cfg.host != cfg_instance.server.host
                    || new_server_cfg.port != cfg_instance.server.port
                {
                    if new_server_cfg.port == cfg_instance.server.port {
                        _http_server_sender.send(()).unwrap();
                        sleep(Duration::from_secs(5));
                        let start_new_http_server =
                            http::setup(cfg.clone(), commands.clone(), report_state.clone()).await;
                        if let Err(reason) = start_new_http_server {
                            error!("could not start new HTTP server: {}. Attempt to start another server with old configuration settings", reason);
                            cfg.write().unwrap().config_value.server.host =
                                cfg_instance.server.host.clone();
                            cfg.write().unwrap().config_value.server.port =
                                cfg_instance.server.port.clone();
                            let start_old_http_server =
                                http::setup(cfg.clone(), commands.clone(), report_state.clone())
                                    .await;
                            if let Err(reason) = start_old_http_server {
                                let reason = format!("could not start old HTTP server: {}", reason);
                                error!("{}", reason);
                                return Err(reason);
                            };
                            (_http_server_sender, _http_server_receiver) =
                                start_old_http_server.unwrap();
                        } else {
                            (_http_server_sender, _http_server_receiver) =
                                start_new_http_server.unwrap();
                        }
                    } else {
                        let start_new_http_server =
                            http::setup(cfg.clone(), commands.clone(), report_state.clone()).await;
                        if let Err(reason) = start_new_http_server {
                            error!(
                                "could not start new HTTP server: {}. Old HTTP server still works",
                                reason
                            );
                        } else {
                            _http_server_sender.send(()).unwrap();
                            (_http_server_sender, _http_server_receiver) =
                                start_new_http_server.unwrap();
                        }
                    }
                };

                cfg_instance = new_cfg_instance;
                Ok(())
            }
            Ok(_) => Ok(()),
            Err(reason) => Err(reason),
        }?;
    }
}
