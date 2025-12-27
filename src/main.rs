use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};

mod captcha;
mod cmd;
mod http;
mod logging;
mod samples;
mod settings;
mod utils;
mod www;

fn main() -> Result<(), String> {
    let mut logging_state = logging::setup(settings::CfgLogging::default());
    let cfg = match settings::try_setup() {
        Ok(cfg) => {
            cfg.trace_log();
            logging::update(cfg.config_value.clone().logging, &mut logging_state);
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
    let cfg_instance = cfg
        .read()
        .map_err(|e| format!("Configuration lock poisoned: {}", e))?
        .config_value
        .clone();
    let root_directory = cfg_instance.commands.root_directory.clone();
    let api_run_path = if let Some(stripped) = http::API_RUN_BASE_PATH.strip_prefix('/') {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(http::API_RUN_BASE_PATH)
    };
    let commands = Arc::new(RwLock::new(
        cmd::tree::Command::new(
            &root_directory,
            &PathBuf::from(cfg_instance.server.http_base_path.clone()).join(api_run_path),
        )
        .map_err(|reason| reason.to_string())?,
    ));
    let server_config = http::setup(cfg.clone(), commands.clone())?;
    if cfg_instance.server.print_banner {
        samples::maybe_print(samples::CMDSample::Banner)
    }

    // Start server in main thread - this will block forever
    http::start_server(server_config)?;

    Ok(())
}
