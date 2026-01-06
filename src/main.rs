use std::path::PathBuf;
use std::process::exit;

mod captcha;
mod cmd;
mod http;
mod mcp;
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
    println!("\n__  __  ____ ____  ____\n|  \\/  |/ ___|  _ \\|  _ \\\n| |\\/| | |   | |_) | | | |\n| |  | | |___|  __/| |_| |\n|_|  |_|\\____|_|   |____/\n\nMCP daemon - expose simple scripts as MCP tools and resources\n");

    // Start server in main thread - this will block forever
    http::start_server(server_config)?;

    Ok(())
}
