use std::path::PathBuf;
use std::process::exit;
use std::sync::{Arc, RwLock};

mod captcha;
mod cmd;
mod http;
mod logging;
mod settings;
mod utils;
mod www;

fn main() -> Result<(), String> {
    let mut logging_state = logging::setup(settings::LoggingConfig {
        level_name: Default::default(),
        output: std::path::PathBuf::from("stderr"),
    });
    let cfg = match settings::try_setup() {
        Ok(cfg) => {
            cfg.trace_log();
            logging::update(settings::LoggingConfig {
                level_name: cfg.config_value.level_name.clone(),
                output: cfg.config_value.output.clone(),
            }, &mut logging_state);
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
    let root_directory = cfg_instance.root_directory.clone();
    let api_run_path = if let Some(stripped) = http::API_RUN_BASE_PATH.strip_prefix('/') {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(http::API_RUN_BASE_PATH)
    };
    let commands = Arc::new(RwLock::new(
        cmd::tree::Command::new(
            &root_directory,
            &PathBuf::from(cfg_instance.http_base_path.clone()).join(api_run_path),
        )
        .map_err(|reason| reason.to_string())?,
    ));
    let server_config = http::setup(cfg.clone(), commands.clone())?;
    println!("\n_____    ____            __  ______                                          __             _\n____    / __ \\___  _____/ /_/ ____/___  ____ ___  ____ ___  ____ _____  ____/ /__  _____   __\n___    / /_/ / _ \\/ ___/ __/ /   / __ \\/ __ `__ \\/ __ `__ \\/ __ `/ __ \\/ __  / _ \\/ ___/  ___\n__    / _, _/  __(__  ) /_/ /___/ /_/ / / / / / / / / / / / /_/ / / / / /_/ /  __/ /     ____\n_    /_/ |_|\\___/____/\\__/\\____/\\____/_/ /_/ /_/_/ /_/ /_/\\__,_/_/ /_/\\__,_/\\___/_/     _____\n");

    // Start server in main thread - this will block forever
    http::start_server(server_config)?;

    Ok(())
}
