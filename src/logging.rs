use crate::settings::LoggingConfig;

use std::io::Write;

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    filter::Filtered, fmt::format::JsonFields, layer::SubscriberExt, reload::Handle,
    util::SubscriberInitExt, Layer, Registry,
};

type JsonHandle = Handle<
    Filtered<
        tracing_subscriber::fmt::Layer<
            Registry,
            JsonFields,
            tracing_subscriber::fmt::format::Format<tracing_subscriber::fmt::format::Json>,
            NonBlocking,
        >,
        tracing_subscriber::filter::LevelFilter,
        Registry,
    >,
    Registry,
>;

#[derive(Debug)]
pub struct LoggingState {
    worker_guard: WorkerGuard,
    json_handle: JsonHandle,
}

#[derive(Debug)]
struct StdNull;
impl Write for StdNull {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn setup(config: LoggingConfig) -> LoggingState {
    let (logging_writer, logging_writer_guard) = writer(&config);
    let logging_json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_level(true)
        .with_file(false)
        .with_line_number(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_writer(logging_writer)
        .with_filter(tracing_subscriber::filter::LevelFilter::OFF);
    let (logging_json_layer, logging_json_handle) =
        tracing_subscriber::reload::Layer::new(logging_json_layer);

    tracing_subscriber::registry()
        .with(logging_json_layer)
        .init();
    LoggingState {
        worker_guard: logging_writer_guard,
        json_handle: logging_json_handle,
    }
}

pub fn update(config: LoggingConfig, state: &mut LoggingState) {
    let (logging_writer, logging_writer_guard) = writer(&config);
    tracing::debug!(
        msg = "Updating logging options",
        level = ?config.level_name,
        output = ?config.output,
    );
    state
        .json_handle
        .modify(|json_layer| {
            *json_layer.filter_mut() = config.level_name.to_level_filter();
            *json_layer.inner_mut().writer_mut() = logging_writer;
        })
        .unwrap();
    state.worker_guard = logging_writer_guard;
    tracing::debug!(
        msg = "Logging options updated successfully",
        level = ?config.level_name,
        output = ?config.output,
    );
}

fn writer(config: &LoggingConfig) -> (NonBlocking, WorkerGuard) {
    match config.output.to_str() {
        Some("stdout") => tracing_appender::non_blocking(std::io::stdout()),
        Some("stderr") => tracing_appender::non_blocking(std::io::stderr()),
        Some("off") => tracing_appender::non_blocking(StdNull),
        _ => tracing_appender::non_blocking(tracing_appender::rolling::daily(
            config.output.clone(),
            env!("CARGO_PKG_NAME").to_owned() + ".log",
        )),
    }
}
