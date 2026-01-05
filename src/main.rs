use std::path::PathBuf;
use std::process::exit;

mod captcha;
mod cmd;
mod http;
mod settings;
mod utils;
mod www;

use tracing_subscriber::{filter::LevelFilter, fmt};

fn main() -> Result<(), String> {
    let settings = match settings::try_setup() {
        Ok(settings) => {
            let level = settings.logging_level();
            let show_target = matches!(level, LevelFilter::DEBUG | LevelFilter::TRACE);
            let show_location = level == LevelFilter::TRACE;
            fmt::Subscriber::builder()
                .with_max_level(level)
                .json()
                .flatten_event(true)
                .log_internal_errors(true)
                .with_level(true)
                .with_file(show_location)
                .with_line_number(show_location)
                .with_target(show_target)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_writer(std::io::stderr)
                .init();
            settings
        }
        Err(error) => {
            eprintln!("{}", error);
            exit(1)
        }
    };
    let api_run_path = if let Some(stripped) = http::API_RUN_BASE_PATH.strip_prefix('/') {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(http::API_RUN_BASE_PATH)
    };
    let commands = cmd::tree::Command::new(
        &settings.root_directory,
        &PathBuf::from(&settings.http_base_path).join(api_run_path),
    )
    .map_err(|reason| reason.to_string())?;
    let server_config = http::setup(settings, commands)?;
    println!("\n_____    ____            __  ______                                          __             _\n____    / __ \\___  _____/ /_/ ____/___  ____ ___  ____ ___  ____ _____  ____/ /__  _____   __\n___    / /_/ / _ \\/ ___/ __/ /   / __ \\/ __ `__ \\/ __ `__ \\/ __ `/ __ \\/ __  / _ \\/ ___/  ___\n__    / _, _/  __(__  ) /_/ /___/ /_/ / / / / / / / / / / / /_/ / / / / /_/ /  __/ /     ____\n_    /_/ |_|\\___/____/\\__/\\____/\\____/_/ /_/ /_/_/ /_/ /_/\\__,_/_/ /_/\\__,_/\\___/_/     _____\n");

    // Start server in main thread - this will block forever
    http::start_server(server_config)?;

    Ok(())
}
