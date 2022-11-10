use crate::settings::CfgLogging;

use std::path::PathBuf;
use std::time::SystemTime;

use thiserror::Error;
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};
use tracing::{debug, error, info};

use serde_derive::{Deserialize, Serialize};
use wildmatch::WildMatch;

const DEFAULT_SEARCH_LIMIT: usize = 10;
const MAX_SEARCH_LIMIT: usize = 1000;

#[derive(Clone, Debug)]
pub struct State {
    channel: ChannelType,
    filename: PathBuf,
}

#[derive(Clone, Debug)]
enum ChannelType {
    MPSC(mpsc::Sender<Message>),
    Stdout,
    Stderr,
    Off,
}

#[derive(Debug)]
enum Message {
    Report(String),
    Stop,
}

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Reports are forwarding to stdout/stderr or the report system is turned of.")]
    NotAvailable,
    #[error("Could not open report file {filename:?}: {error}")]
    Open { error: String, filename: PathBuf },
    #[error("Could not read report file {filename:?}: {error}")]
    Read { error: String, filename: PathBuf },
    #[error("Invalid input for {field}({value:?}): {error}")]
    InvalidInput {
        field: String,
        value: String,
        error: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    from: String,
    #[serde(skip)]
    timestamp_integer: u64,
    timestamp: String,
    context: ReportContext,
    info: String,
}

impl Report {
    fn matches(
        &self,
        maybe_from: Option<String>,
        maybe_before_time: Option<u64>,
        maybe_after_time: Option<u64>,
        maybe_context: Option<ReportContext>,
    ) -> bool {
        if let Some(context) = maybe_context {
            if context != self.context {
                return false;
            }
        }
        if let Some(from) = maybe_from {
            if !WildMatch::new(from.as_str()).matches(self.from.as_str()) {
                return false;
            };
        }
        if let Some(before_time) = maybe_before_time {
            if before_time < self.timestamp_integer {
                return false;
            }
        };
        if let Some(after_time) = maybe_after_time {
            if after_time > self.timestamp_integer {
                return false;
            }
        };
        true
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportContext {
    Run,
    State,
}

pub async fn maybe_setup(config: CfgLogging, last_state: Option<State>) -> Result<State, String> {
    maybe_stop(last_state).await;
    match config.report.to_str().unwrap_or_default() {
        "stdout" => Ok(State {
            channel: ChannelType::Stdout,
            filename: PathBuf::from("stdout"),
        }),
        "stderr" => Ok(State {
            channel: ChannelType::Stderr,
            filename: PathBuf::from("stderr"),
        }),
        "off" => Ok(State {
            channel: ChannelType::Off,
            filename: PathBuf::from("off"),
        }),
        _ => setup(config).await,
    }
}

pub async fn maybe_stop(state: Option<State>) {
    if state.is_none() {
        return;
    }
    if let ChannelType::MPSC(producer) = state.unwrap().channel {
        producer.send(Message::Stop).await.unwrap_or_else(|error| {
            error!(
                error = error.to_string().as_str(),
                "Could not send `Stop` message to report channel"
            )
        });
    }
}

pub async fn report(
    from: String,
    context: ReportContext,
    info: String,
    state: State,
    maybe_timestamp: Option<SystemTime>,
) {
    let timestamp = if let Some(timestamp) = maybe_timestamp {
        timestamp
    } else {
        SystemTime::now()
    };
    let timestamp = humantime::format_rfc3339_millis(timestamp).to_string();
    let report = serde_json::to_string(&Report {
        from,
        timestamp,
        context,
        info,
        timestamp_integer: 0,
    })
    .unwrap()
        + "\n";
    match state.channel {
        ChannelType::Off => {}
        ChannelType::Stdout => print!("{}", report),
        ChannelType::Stderr => eprint!("{}", report),
        ChannelType::MPSC(producer) => {
            debug!("Attempt to send report to reporter thread");
            producer
                .send(Message::Report(report))
                .await
                .unwrap_or_else(|error| {
                    error!(
                        error = error.to_string().as_str(),
                        "Could not send `Report(_)` message to report channel"
                    )
                });
        }
    };
}

pub async fn search(
    maybe_from: Option<String>,
    maybe_before_time: Option<String>,
    maybe_after_time: Option<String>,
    maybe_context: Option<ReportContext>,
    maybe_limit: Option<usize>,
    state: State,
) -> Result<Vec<Report>, ReportError> {
    match state.channel {
        ChannelType::MPSC(_) => {
            search_in_file(
                maybe_from,
                maybe_before_time,
                maybe_after_time,
                maybe_context,
                maybe_limit,
                state.filename,
            )
            .await
        }
        _ => Err(ReportError::NotAvailable),
    }
}

async fn search_in_file(
    maybe_from: Option<String>,
    maybe_before_time: Option<String>,
    maybe_after_time: Option<String>,
    maybe_context: Option<ReportContext>,
    maybe_limit: Option<usize>,
    filename: PathBuf,
) -> Result<Vec<Report>, ReportError> {
    let file = OpenOptions::new()
        .read(true)
        .open(filename.clone())
        .await
        .map_err(|error| ReportError::Open {
            filename: filename.clone(),
            error: error.to_string(),
        })?;
    let mut file_reader = BufReader::new(file);
    let mut report_list = Vec::new();
    let mut line_number = 0;
    let maybe_before_time = if let Some(before_time) = maybe_before_time {
        Some(
            humantime::parse_rfc3339_weak(&before_time)
                .map_err(|error| ReportError::InvalidInput {
                    field: "before_time".to_string(),
                    value: before_time,
                    error: error.to_string(),
                })?
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        )
    } else {
        None
    };
    let maybe_after_time = if let Some(after_time) = maybe_after_time {
        Some(
            humantime::parse_rfc3339_weak(&after_time)
                .map_err(|error| ReportError::InvalidInput {
                    field: "after_time".to_string(),
                    value: after_time,
                    error: error.to_string(),
                })?
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        )
    } else {
        None
    };
    let limit = if let Some(limit) = maybe_limit {
        limit
    } else {
        DEFAULT_SEARCH_LIMIT
    };
    if limit > MAX_SEARCH_LIMIT {
        return Err(ReportError::InvalidInput {
            field: "limit".to_string(),
            value: limit.to_string(),
            error: format!(
                "Reached maximum number for limit which is {}",
                MAX_SEARCH_LIMIT
            ),
        });
    }
    loop {
        let mut buffer = String::new();
        match file_reader.read_line(&mut buffer).await {
            Ok(0) => break,
            Err(error) => {
                return Err(ReportError::Read {
                    filename: filename.clone(),
                    error: error.to_string(),
                })
            }
            Ok(_) => {
                line_number += 1;
                match serde_json::from_str::<Report>(&buffer) {
                    Ok(mut report) => {
                        match humantime::parse_rfc3339(&report.timestamp) {
                            Ok(system_time) => {
                                report.timestamp_integer = system_time
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis()
                                    as u64;
                            }
                            Err(error) => {
                                error!(line = line_number, filename = ?filename.clone(), error = error.to_string().as_str(), "Could not decode timestamp.");
                                continue;
                            }
                        }
                        if report.matches(
                            maybe_from.clone(),
                            maybe_before_time,
                            maybe_after_time,
                            maybe_context.clone(),
                        ) {
                            report_list.push(report)
                        }
                        if report_list.len() > limit {
                            report_list = report_list.into_iter().skip(1).collect::<Vec<_>>();
                        }
                    }
                    Err(error) => {
                        error!(line = line_number, filename = ?filename.clone(), error = error.to_string().as_str(), line = ?buffer, "Could not decode line.");
                        continue;
                    }
                }
            }
        }
    }
    Ok(report_list)
}

async fn setup(config: CfgLogging) -> Result<State, String> {
    let (producer, consumer) = mpsc::channel(32);
    let report_file = tokio::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(config.report.clone())
        .await
        .map_err(|error| {
            format!(
                "Could not open report file {:?}: {}",
                config.report,
                error.to_string()
            )
        })?;
    tokio::task::spawn({
        let report_filename = config.report.clone();
        async move {
            debug!("Reporter thread started.");
            report_loop(report_file, report_filename, consumer).await;
        }
    });
    Ok(State {
        channel: ChannelType::MPSC(producer),
        filename: config.report.clone(),
    })
}

async fn report_loop(mut file: File, filename: PathBuf, mut consumer: mpsc::Receiver<Message>) {
    loop {
        match consumer.recv().await {
            Some(message) => match message {
                Message::Report(report) => match file.write_all(report.as_bytes()).await {
                    Err(error) => {
                        error!(report_file = ?filename, error = error.to_string().as_str(), "Could not write to report file");
                        debug!(report_file = ?filename, "Attempt to reopen report file");
                        match tokio::fs::OpenOptions::new()
                            .write(true)
                            .append(true)
                            .open(filename.clone())
                            .await
                        {
                            Ok(new_file) => {
                                file = new_file;
                                info!(report_file = ?filename, "Reopened report file.");
                            }
                            Err(error) => {
                                error!(report_file = ?filename, error = error.to_string().as_str(), "Could not reopen report file")
                            }
                        }
                    }
                    _ => {}
                },
                Message::Stop => {
                    info!(report_file = ?filename, "Reported thread stopped.");
                    break;
                }
            },
            None => {
                error!(report_file = ?filename, "Report channel is closed but the reporter thread did not get `Stop` message");
                break;
            }
        }
    }
}
