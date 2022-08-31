use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use log::{error, info, trace, LevelFilter};

mod captcha;
mod cmd;
mod http;
mod logging;
mod samples;
mod settings;
mod utils;
mod www;

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut log_handle = logging::setup(LevelFilter::Info);
    let cfg = match settings::try_setup() {
        Ok(cfg) => Arc::new(RwLock::new(cfg)),
        Err(maybe_error) => {
            if maybe_error.is_none() {
                exit(0)
            } else {
                println!("{}", maybe_error.unwrap());
                exit(1)
            }
        }
    };
    let mut cfg_instance = cfg.read().unwrap().config_value.clone();
    logging::update_log_level(
        cfg_instance.logging.level_name.to_log_level(),
        &mut log_handle,
    );
    trace!("{:#?}", cfg);
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
    let (mut _http_server_sender, mut _http_server_receiver) =
        http::setup(cfg.clone(), commands.clone()).await?;
    loop {
        match http::maybe_handle_message(&mut _http_server_receiver).await {
            Ok(true) => {
                // Update logging:
                let new_cfg_instance = cfg.write().unwrap().config_value.clone();
                let new_cfg_log_level = new_cfg_instance.logging.level_name.clone();
                if cfg_instance.logging.level_name != new_cfg_log_level {
                    logging::update_log_level(new_cfg_log_level.to_log_level(), &mut log_handle)
                };
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
                            http::setup(cfg.clone(), commands.clone()).await;
                        if let Err(reason) = start_new_http_server {
                            error!("could not start new HTTP server: {}. Attempt to start another server with old configuration settings", reason);
                            cfg.write().unwrap().config_value.server.host =
                                cfg_instance.server.host.clone();
                            cfg.write().unwrap().config_value.server.port =
                                cfg_instance.server.port.clone();
                            let start_old_http_server =
                                http::setup(cfg.clone(), commands.clone()).await;
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
                            http::setup(cfg.clone(), commands.clone()).await;
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
                }
                info!(
                    "configuration reloaded from {:?}",
                    cfg.read().unwrap().filename.clone().unwrap()
                );
                cfg_instance = new_cfg_instance;
                Ok(())
            }
            Ok(_) => Ok(()),
            Err(reason) => Err(reason),
        }?;
    }
}
