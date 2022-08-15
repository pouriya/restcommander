use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

const LOG_FORMAT: &str = "{d(%Y/%m/%d %H:%M:%S%.6f)} {l:<6} {M:<30.30} {m}{n}";

pub fn setup(level_name: LevelFilter) -> log4rs::Handle {
    log4rs::init_config(log_config(level_name)).unwrap()
}

pub fn update_log_level(level_name: LevelFilter, log_handle: &mut log4rs::Handle) {
    match level_name {
        LevelFilter::Trace => info!("Trace mode is on"),
        LevelFilter::Debug => info!("Debug mode is on"),
        _ => (),
    };
    log_handle.set_config(log_config(level_name));
}

fn log_config(level_name: LevelFilter) -> log4rs::config::runtime::Config {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_FORMAT)))
        .build();
    // Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build()));
    log4rs::config::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(level_name))
        .unwrap()
}
