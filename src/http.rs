use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::time;

use rustls::{ServerConfig as RustlsServerConfig, ServerConnection};
use rustls_pemfile::{certs, pkcs8_private_keys};

use http::{HeaderName, HeaderValue, Request as HttpRequest, Response as HttpResponse, StatusCode};
use httparse::{Request as HttpParseRequest, Status};
use serde_derive::Deserialize;
use serde_json::json;
use thiserror::Error;

use tracing::level_enabled;

use crate::captcha;
use crate::cmd::{Command, CommandStats};
use crate::settings::CommandLine;
use crate::utils;
use crate::www;

//  for future use for HTTP "Server" header
// use structopt::clap::crate_name;

pub static API_RUN_BASE_PATH: &str = "/api/run";

// Request wrapper to maintain API compatibility
pub struct RequestWrapper {
    request: HttpRequest<Vec<u8>>,
    remote_addr: SocketAddr,
}

impl RequestWrapper {
    pub fn new(request: HttpRequest<Vec<u8>>, remote_addr: SocketAddr) -> Self {
        Self {
            request,
            remote_addr,
        }
    }

    pub fn url(&self) -> &str {
        self.request.uri().path()
    }

    pub fn method(&self) -> &str {
        self.request.method().as_str()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.request
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
    }

    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.request
            .headers()
            .iter()
            .filter_map(|(name, value)| value.to_str().ok().map(|v| (name.as_str(), v)))
    }

    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    pub fn body(&self) -> &[u8] {
        self.request.body()
    }
}

// Response helper functions
pub type HttpResponseType = HttpResponse<Vec<u8>>;

pub fn response_text(body: impl Into<String>) -> HttpResponseType {
    let body_str = body.into();
    HttpResponse::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Connection", "close")
        .body(body_str.into_bytes())
        .unwrap()
}

pub fn response_from_data(mime_type: &str, data: Vec<u8>) -> HttpResponseType {
    HttpResponse::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .header("Connection", "close")
        .body(data)
        .unwrap()
}

pub fn response_redirect_301(location: String) -> HttpResponseType {
    HttpResponse::builder()
        .status(StatusCode::MOVED_PERMANENTLY)
        .header("Location", location)
        .header("Connection", "close")
        .body(Vec::new())
        .unwrap()
}

fn response_with_status_code(response: HttpResponseType, code: u16) -> HttpResponseType {
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let (parts, body) = response.into_parts();
    let mut new_response = HttpResponse::from_parts(parts, body);
    *new_response.status_mut() = status;
    new_response
}

#[derive(Error, Debug)]
pub enum ServerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("Read timeout from {remote_addr}")]
    ReadTimeout { remote_addr: SocketAddr },
    #[error("Write timeout")]
    WriteTimeout,
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Connection closed while reading body")]
    ConnectionClosedWhileReadingBody,
    #[error("Request headers too large")]
    RequestHeadersTooLarge,
    #[error("Content-Length too large (max 65535)")]
    ContentLengthTooLarge,
    #[error("Failed to parse HTTP request: {0}")]
    HttpParse(#[from] httparse::Error),
    #[error("Invalid URI: {0}")]
    InvalidUri(#[from] http::uri::InvalidUri),
    #[error("Invalid header name: {0}")]
    InvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error("Invalid Content-Length header (not UTF-8): {0}")]
    InvalidContentLengthUtf8(#[from] http::header::ToStrError),
    #[error("Invalid Content-Length header (not a number): {0}")]
    InvalidContentLengthNumber(#[from] std::num::ParseIntError),
    #[error("Failed to build request: {0}")]
    FailedToBuildRequest(#[from] http::Error),
    #[error("Configuration lock poisoned: {0}")]
    ConfigurationLockPoisoned(String),
    #[error("Commands lock poisoned: {0}")]
    CommandsLockPoisoned(String),
    #[error("Failed to bind to {address}: {source}")]
    FailedToBind {
        address: String,
        source: std::io::Error,
    },
    #[error("Failed to parse certificate: {source}")]
    FailedToParseCertificate { source: std::io::Error },
    #[error("Failed to parse private key: {source}")]
    FailedToParsePrivateKey { source: std::io::Error },
    #[error("No private keys found in key file")]
    NoPrivateKeysFound,
    #[error("Failed to create TLS config: {0}")]
    FailedToCreateTlsConfig(#[from] rustls::Error),
    #[error("Missing HTTP method")]
    MissingHttpMethod,
    #[error("Missing HTTP path")]
    MissingHttpPath,
    #[error("Failed to build request headers")]
    FailedToBuildRequestHeaders,
}

impl ServerError {
    pub fn is_timeout(&self) -> bool {
        match self {
            Self::ReadTimeout { .. } | Self::WriteTimeout => true,
            Self::Io(e) => e.kind() == ErrorKind::TimedOut,
            _ => false,
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for ServerError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::ConfigurationLockPoisoned(err.to_string())
    }
}

impl From<ServerError> for String {
    fn from(err: ServerError) -> Self {
        err.to_string()
    }
}

#[derive(Error, Debug, Clone)]
pub enum HTTPError {
    #[error(transparent)]
    Authentication(#[from] HTTPAuthenticationError),
    #[error(transparent)]
    #[allow(clippy::upper_case_acronyms)]
    API(#[from] HTTPAPIError),
    #[error("{0}")]
    Deserialize(String),
}

impl HTTPError {
    fn http_error_code(&self) -> i32 {
        match self {
            Self::Authentication(x) => x.http_error_code(),
            Self::API(x) => x.http_error_code(),
            Self::Deserialize(_) => 2000,
        }
    }

    fn http_status_code(&self) -> u16 {
        match self {
            Self::Authentication(x) => x.http_status_code(),
            Self::API(x) => x.http_status_code(),
            Self::Deserialize(_) => 400,
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum HTTPAuthenticationError {
    #[error("could not decode authorization header value {data:?} to base64 ({source:?})")]
    Base64Decode {
        data: String,
        source: base64::DecodeError,
    },
    #[error("Username or password is not set in server configuration")]
    UsernameOrPasswordIsNotSet,
    #[error("Could not found username or password in {data:?}")]
    UsernameOrPasswordIsNotFound { data: String },
    #[error("Unknown authentication method {method:?}")]
    UnknownMethod { method: String },
    #[error("Invalid basic authentication with header value {header_value:?}")]
    InvalidBasicAuthentication { header_value: String },
    #[error("Invalid username or password")]
    InvalidUsernameOrPassword,
    #[error("invalid CAPTCHA")]
    InvalidCaptcha,
    #[error("Invalid CAPTCHA form")]
    InvalidCaptchaForm,
    #[error("Token not found or expired")]
    TokenNotFound,
    #[error("Token is expired")]
    TokenExpired,
    #[error("Token is invalid")]
    InvalidToken,
    #[error("{0}")]
    Captcha(String),
}

#[derive(Error, Debug, Clone)]
pub enum HTTPAPIError {
    #[error("Password should not be empty")]
    EmptyPassword,
    #[error("Previous password is required")]
    PreviousPasswordRequired,
    #[error("Invalid previous password")]
    InvalidPreviousPassword,
    #[error("Server configuration does not allow client to change the password")]
    NoPasswordFile,
    #[error("Could not save new password to configured password file ({message})")]
    SaveNewPassword { message: String },
}

impl HTTPAPIError {
    fn http_error_code(&self) -> i32 {
        match self {
            Self::EmptyPassword => 1007,
            Self::PreviousPasswordRequired => 1009,
            Self::InvalidPreviousPassword => 1011,
            Self::NoPasswordFile => 1008,
            Self::SaveNewPassword { .. } => 1010,
        }
    }

    fn http_status_code(&self) -> u16 {
        match self {
            Self::EmptyPassword => 400,
            Self::PreviousPasswordRequired => 400,
            Self::InvalidPreviousPassword => 401,
            Self::NoPasswordFile => 503,
            Self::SaveNewPassword { .. } => 500,
        }
    }
}

impl HTTPAuthenticationError {
    fn http_error_code(&self) -> i32 {
        match self {
            Self::Base64Decode { .. } => 2002,
            Self::UsernameOrPasswordIsNotFound { .. } => 2003,
            Self::UsernameOrPasswordIsNotSet { .. } => 2004,
            Self::UnknownMethod { .. } => 2005,
            Self::InvalidBasicAuthentication { .. } => 2006,
            Self::InvalidUsernameOrPassword => 2007,
            Self::InvalidCaptcha => 2008,
            Self::InvalidCaptchaForm => 2009,
            Self::TokenNotFound => 2010,
            Self::TokenExpired => 2011,
            Self::InvalidToken => 2012,
            Self::Captcha(_) => 2012,
        }
    }

    fn http_status_code(&self) -> u16 {
        match self {
            Self::Base64Decode { .. } => 400,
            Self::UsernameOrPasswordIsNotFound { .. } => 401,
            Self::UsernameOrPasswordIsNotSet { .. } => 409,
            Self::UnknownMethod { .. } => 400,
            Self::InvalidBasicAuthentication { .. } => 400,
            Self::InvalidUsernameOrPassword => 401,
            Self::InvalidCaptcha => 401,
            Self::InvalidCaptchaForm => 400,
            Self::TokenNotFound => 401,
            Self::TokenExpired => 401,
            Self::InvalidToken => 401,
            Self::Captcha(_) => 406,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SetPassword {
    password: String,
    hash: bool, // Required field - indicates if password is already SHA256 hashed
    previous_password: Option<String>, // Previous password (uses same hash flag)
}

pub struct ServerConfig {
    pub handler_state: HandlerState,
    pub address: String,
    pub tls_cert: Option<Vec<u8>>,
    pub tls_key: Option<Vec<u8>>,
    pub has_tls: bool,
}

pub fn setup(cfg: CommandLine, commands: Command) -> Result<ServerConfig, ServerError> {
    let host = cfg.http_host.clone();
    let port = cfg.http_port;

    // Convert commands from RwLock

    let maybe_captcha = if cfg.http_auth_captcha {
        Some(captcha::Captcha::new())
    } else {
        None
    };
    let tokens = HashMap::new();

    // Create state for handlers
    let handler_state = HandlerState {
        cfg,
        commands,
        tokens,
        maybe_captcha,
    };

    let address = format!("{}:{}", host, port);

    if let (Some(ref tls_cert_file), Some(ref tls_key_file)) = (
        handler_state.cfg.http_tls_cert_file.clone(),
        handler_state.cfg.http_tls_key_file.clone(),
    ) {
        // Load TLS certificates as raw bytes
        let cert_bytes = std::fs::read(tls_cert_file)?;
        let key_bytes = std::fs::read(tls_key_file)?;

        tracing::debug!(
            msg = "Prepared HTTPS server",
            host = host.as_str(),
            port = port,
            cert_file = ?tls_cert_file,
            key_file = ?tls_key_file,
        );

        Ok(ServerConfig {
            handler_state,
            address,
            tls_cert: Some(cert_bytes),
            tls_key: Some(key_bytes),
            has_tls: true,
        })
    } else {
        tracing::debug!(
            msg = "Prepared HTTP server",
            host = host.as_str(),
            port = port,
        );

        Ok(ServerConfig {
            handler_state,
            address,
            tls_cert: None,
            tls_key: None,
            has_tls: false,
        })
    }
}

pub fn start_server(config: ServerConfig) -> Result<(), ServerError> {
    let mut state = config.handler_state;
    let address = config.address.clone();

    tracing::info!(
        msg = "Starting HTTP server",
        address = address.as_str(),
        tls = config.has_tls,
    );

    let listener = TcpListener::bind(&address).map_err(|e| ServerError::FailedToBind {
        address: address.clone(),
        source: e,
    })?;

    if let (Some(cert_bytes), Some(key_bytes)) = (config.tls_cert, config.tls_key) {
        // HTTPS server
        // Load certificates and keys
        let mut cert_reader = std::io::Cursor::new(cert_bytes);
        let certs = certs(&mut cert_reader)
            .map_err(|e| ServerError::FailedToParseCertificate { source: e })?
            .into_iter()
            .map(rustls::Certificate)
            .collect::<Vec<_>>();

        let mut key_reader = std::io::Cursor::new(key_bytes);
        let mut keys = pkcs8_private_keys(&mut key_reader)
            .map_err(|e| ServerError::FailedToParsePrivateKey { source: e })?;

        if keys.is_empty() {
            return Err(ServerError::NoPrivateKeysFound);
        }

        let key = rustls::PrivateKey(keys.remove(0));

        // Create TLS server config
        let tls_config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, key)?;

        let tls_config = Arc::new(tls_config);

        // HTTPS server loop
        let read_timeout = std::time::Duration::from_secs(state.cfg.http_read_timeout_secs);
        let write_timeout = std::time::Duration::from_secs(state.cfg.http_write_timeout_secs);

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(mut stream) => {
                    let peer_addr = match stream.peer_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            tracing::warn!(msg = "Failed to get peer address", error = %e);
                            continue;
                        }
                    };

                    // Set read and write timeouts before TLS handshake
                    if let Err(e) = stream.set_read_timeout(Some(read_timeout)) {
                        tracing::warn!(
                            msg = "Failed to set read timeout",
                            peer_addr = %peer_addr,
                            error = %e
                        );
                        continue;
                    }
                    if let Err(e) = stream.set_write_timeout(Some(write_timeout)) {
                        tracing::warn!(
                            msg = "Failed to set write timeout",
                            peer_addr = %peer_addr,
                            error = %e
                        );
                        continue;
                    }

                    // Create TLS connection - log error and continue if it fails
                    let mut tls_conn = match ServerConnection::new(tls_config.clone()) {
                        Ok(conn) => conn,
                        Err(e) => {
                            tracing::warn!(
                                msg = "Failed to create TLS connection",
                                peer_addr = %peer_addr,
                                error = %e
                            );
                            continue;
                        }
                    };

                    // Complete TLS handshake - log error and continue if it fails
                    if let Err(e) = tls_conn.complete_io(&mut stream) {
                        tracing::warn!(
                            msg = "TLS handshake failed",
                            peer_addr = %peer_addr,
                            error = %e
                        );
                        continue;
                    }

                    // Create TLS stream for reading/writing
                    let mut tls_stream = rustls::Stream::new(&mut tls_conn, &mut stream);

                    // Read request through TLS
                    match parse_http_request(&mut tls_stream, peer_addr) {
                        Ok(request) => {
                            let response = handle_request(&request, &mut state);

                            // Log request and response together
                            log_http_request_response(&request, &response);

                            if let Err(e) = write_http_response(&mut tls_stream, response) {
                                if e.is_timeout() {
                                    tracing::warn!(
                                        msg = "Client write timeout - dropping connection",
                                        peer_addr = %peer_addr,
                                        error = %e
                                    );
                                } else {
                                    tracing::warn!(
                                        msg = "Failed to write response",
                                        peer_addr = %peer_addr,
                                        error = %e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            if e.is_timeout() {
                                tracing::warn!(
                                    msg = "Client read timeout - dropping connection",
                                    peer_addr = %peer_addr,
                                    error = %e
                                );
                                // Connection will be dropped when stream goes out of scope
                                continue;
                            } else {
                                tracing::warn!(
                                    msg = "Failed to parse request",
                                    peer_addr = %peer_addr,
                                    error = %e
                                );
                                let error_response =
                                    response_with_status_code(response_text("Bad Request"), 400);
                                if let Err(write_err) =
                                    write_http_response(&mut tls_stream, error_response)
                                {
                                    if write_err.is_timeout() {
                                        tracing::warn!(
                                            msg = "Client write timeout - dropping connection",
                                            peer_addr = %peer_addr,
                                            error = %write_err
                                        );
                                    } else {
                                        tracing::warn!(
                                            msg = "Failed to write error response",
                                            peer_addr = %peer_addr,
                                            error = %write_err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(msg = "Failed to accept connection", error = %e);
                }
            }
        }
    } else {
        // HTTP server
        let read_timeout = std::time::Duration::from_secs(state.cfg.http_read_timeout_secs);
        let write_timeout = std::time::Duration::from_secs(state.cfg.http_write_timeout_secs);

        for stream_result in listener.incoming() {
            match stream_result {
                Ok(mut stream) => {
                    let peer_addr = match stream.peer_addr() {
                        Ok(addr) => addr,
                        Err(e) => {
                            tracing::warn!(msg = "Failed to get peer address", error = %e);
                            continue;
                        }
                    };

                    // Set read and write timeouts
                    if let Err(e) = stream.set_read_timeout(Some(read_timeout)) {
                        tracing::warn!(
                            msg = "Failed to set read timeout",
                            peer_addr = %peer_addr,
                            error = %e
                        );
                        continue;
                    }
                    if let Err(e) = stream.set_write_timeout(Some(write_timeout)) {
                        tracing::warn!(
                            msg = "Failed to set write timeout",
                            peer_addr = %peer_addr,
                            error = %e
                        );
                        continue;
                    }

                    match parse_http_request(&mut stream, peer_addr) {
                        Ok(request) => {
                            let response = handle_request(&request, &mut state);

                            // Log request and response together
                            log_http_request_response(&request, &response);

                            if let Err(e) = write_http_response(&mut stream, response) {
                                if e.is_timeout() {
                                    tracing::warn!(
                                        msg = "Client write timeout - dropping connection",
                                        peer_addr = %peer_addr,
                                        error = %e
                                    );
                                } else {
                                    tracing::warn!(
                                        msg = "Failed to write response",
                                        peer_addr = %peer_addr,
                                        error = %e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            if e.is_timeout() {
                                tracing::warn!(
                                    msg = "Client read timeout - dropping connection",
                                    peer_addr = %peer_addr,
                                    error = %e
                                );
                                // Connection will be dropped when stream goes out of scope
                                continue;
                            } else {
                                tracing::warn!(
                                    msg = "Failed to parse request",
                                    peer_addr = %peer_addr,
                                    error = %e
                                );
                                let error_response =
                                    response_with_status_code(response_text("Bad Request"), 400);
                                if let Err(write_err) =
                                    write_http_response(&mut stream, error_response)
                                {
                                    if write_err.is_timeout() {
                                        tracing::warn!(
                                            msg = "Client write timeout - dropping connection",
                                            peer_addr = %peer_addr,
                                            error = %write_err
                                        );
                                    } else {
                                        tracing::warn!(
                                            msg = "Failed to write error response",
                                            peer_addr = %peer_addr,
                                            error = %write_err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(msg = "Failed to accept connection", error = %e);
                }
            }
        }
    }

    Ok(())
}

fn parse_http_request<R: Read>(
    stream: &mut R,
    remote_addr: SocketAddr,
) -> Result<RequestWrapper, ServerError> {
    // Read up to 8KB for headers
    let mut buffer = vec![0u8; 8192];
    let mut total_read = 0;

    loop {
        // Ensure we don't exceed buffer bounds
        if total_read >= buffer.len() {
            return Err(ServerError::RequestHeadersTooLarge);
        }
        let n = stream.read(&mut buffer[total_read..]).map_err(|e| {
            if e.kind() == ErrorKind::TimedOut {
                ServerError::ReadTimeout { remote_addr }
            } else {
                ServerError::Io(e)
            }
        })?;
        if n == 0 {
            return Err(ServerError::ConnectionClosed);
        }
        total_read += n;

        // Try to parse headers
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = HttpParseRequest::new(&mut headers);
        match req.parse(&buffer[..total_read]) {
            Ok(Status::Complete(header_len)) => {
                // Headers parsed successfully
                let method = req.method.ok_or(ServerError::MissingHttpMethod)?;
                let path = req.path.ok_or(ServerError::MissingHttpPath)?;

                // Parse URI to extract path and query
                let uri_str = format!("http://localhost{}", path);
                let uri = uri_str.parse::<http::Uri>()?;

                // Build headers map
                let mut header_map = http::HeaderMap::new();
                for header in req.headers {
                    let name = HeaderName::from_bytes(header.name.as_bytes())?;
                    let value = HeaderValue::from_bytes(header.value)?;
                    header_map.insert(name, value);
                }

                // Read body if Content-Length is present
                let body = if let Some(content_length_str) = header_map.get("content-length") {
                    let content_length_str = content_length_str.to_str()?;
                    let content_length: usize = content_length_str.parse()?;

                    if content_length > 65535 {
                        return Err(ServerError::ContentLengthTooLarge);
                    }

                    // Check if body data is already in the buffer
                    let body_already_read = total_read.saturating_sub(header_len);

                    let mut body = vec![0u8; content_length];
                    let mut body_read = 0;

                    // Copy body data that was already read into the buffer
                    if body_already_read > 0 {
                        let body_in_buffer = body_already_read.min(content_length);
                        if header_len + body_in_buffer <= buffer.len() {
                            body[..body_in_buffer]
                                .copy_from_slice(&buffer[header_len..header_len + body_in_buffer]);
                            body_read = body_in_buffer;
                        }
                    }

                    // Read remaining body bytes from stream
                    while body_read < content_length {
                        let n = stream.read(&mut body[body_read..]).map_err(|e| {
                            if e.kind() == ErrorKind::TimedOut {
                                ServerError::ReadTimeout { remote_addr }
                            } else {
                                ServerError::Io(e)
                            }
                        })?;
                        if n == 0 {
                            return Err(ServerError::ConnectionClosedWhileReadingBody);
                        }
                        body_read += n;
                    }
                    body
                } else {
                    Vec::new()
                };

                // Build http::Request
                let mut request_builder = HttpRequest::builder().method(method).uri(uri);

                // Copy headers
                let headers = request_builder
                    .headers_mut()
                    .ok_or(ServerError::FailedToBuildRequestHeaders)?;
                *headers = header_map;

                let request = request_builder.body(body)?;

                return Ok(RequestWrapper::new(request, remote_addr));
            }
            Ok(Status::Partial) => {
                // Need more data, but check if we've exceeded buffer size
                if total_read >= buffer.len() {
                    return Err(ServerError::RequestHeadersTooLarge);
                }
                // Continue reading
            }
            Err(e) => {
                return Err(ServerError::HttpParse(e));
            }
        }
    }
}

fn write_http_response<W: Write>(
    stream: &mut W,
    response: HttpResponseType,
) -> Result<(), ServerError> {
    // Write status line
    let status_line = format!(
        "HTTP/1.1 {} {}\r\n",
        response.status().as_u16(),
        response.status().canonical_reason().unwrap_or("Unknown")
    );
    stream.write_all(status_line.as_bytes()).map_err(|e| {
        if e.kind() == ErrorKind::TimedOut {
            ServerError::WriteTimeout
        } else {
            ServerError::Io(e)
        }
    })?;

    // Write headers
    for (name, value) in response.headers() {
        let header_line = format!("{}: {}\r\n", name, value.to_str().unwrap_or(""));
        stream.write_all(header_line.as_bytes()).map_err(|e| {
            if e.kind() == ErrorKind::TimedOut {
                ServerError::WriteTimeout
            } else {
                ServerError::Io(e)
            }
        })?;
    }

    // Ensure Connection: close header is present
    if !response.headers().contains_key("connection") {
        stream.write_all(b"Connection: close\r\n").map_err(|e| {
            if e.kind() == ErrorKind::TimedOut {
                ServerError::WriteTimeout
            } else {
                ServerError::Io(e)
            }
        })?;
    }

    // Write empty line to separate headers from body
    stream.write_all(b"\r\n").map_err(|e| {
        if e.kind() == ErrorKind::TimedOut {
            ServerError::WriteTimeout
        } else {
            ServerError::Io(e)
        }
    })?;

    // Write body
    stream.write_all(response.body()).map_err(|e| {
        if e.kind() == ErrorKind::TimedOut {
            ServerError::WriteTimeout
        } else {
            ServerError::Io(e)
        }
    })?;

    stream.flush().map_err(|e| {
        if e.kind() == ErrorKind::TimedOut {
            ServerError::WriteTimeout
        } else {
            ServerError::Io(e)
        }
    })?;

    Ok(())
}

fn log_http_request_response(request: &RequestWrapper, response: &HttpResponseType) {
    let method = request.method();
    let path = request.url();
    let status_code = response.status().as_u16();

    if level_enabled!(tracing::Level::TRACE) {
        // Trace level: log everything (headers, body) for both request and response
        let mut req_headers_str = String::new();
        for (name, value) in request.headers() {
            req_headers_str.push_str(&format!("{}: {}\r\n", name, value));
        }

        let req_body_str = if request.body().is_empty() {
            "<empty>".to_string()
        } else {
            match std::str::from_utf8(request.body()) {
                Ok(s) => s.to_string(),
                Err(_) => format!("<binary {} bytes>", request.body().len()),
            }
        };

        let mut resp_headers_str = String::new();
        for (name, value) in response.headers() {
            resp_headers_str.push_str(&format!("{}: {}\r\n", name, value.to_str().unwrap_or("")));
        }

        let resp_body_str = if response.body().is_empty() {
            "<empty>".to_string()
        } else {
            match std::str::from_utf8(response.body()) {
                Ok(s) => s.to_string(),
                Err(_) => format!("<binary {} bytes>", response.body().len()),
            }
        };

        tracing::trace!(
            msg = "HTTP request/response",
            method = method,
            path = path,
            status_code = status_code,
            remote_addr = %request.remote_addr(),
            request_headers = req_headers_str,
            request_body = req_body_str,
            response_headers = resp_headers_str,
            response_body = resp_body_str,
        );
    } else if level_enabled!(tracing::Level::DEBUG) {
        // Debug level: log path, content-length, response length, status code
        let req_content_length = request
            .header("content-length")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let resp_content_length = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let response_length = response.body().len();

        tracing::debug!(
            msg = "HTTP request/response",
            method = method,
            path = path,
            status_code = status_code,
            remote_addr = %request.remote_addr(),
            request_content_length = req_content_length,
            response_content_length = resp_content_length,
            response_length = response_length,
        );
    } else {
        // Info/Warn/Error level: log path and status code
        tracing::info!(
            msg = "HTTP request/response",
            method = method,
            path = path,
            status_code = status_code,
            remote_addr = %request.remote_addr(),
        );
    }
}

pub struct HandlerState {
    cfg: CommandLine,
    commands: Command,
    tokens: HashMap<String, usize>,
    maybe_captcha: Option<captcha::Captcha>,
}

fn handle_request(request: &RequestWrapper, state: &mut HandlerState) -> HttpResponseType {
    let url = request.url();
    let method = request.method();

    // Handle root redirect
    if method == "GET" && url == "/" {
        return redirect_root_to_index_html(&state.cfg);
    }

    // Handle static files
    if method == "GET" && url.starts_with("/static/") {
        let tail = url.strip_prefix("/static/").unwrap_or("");
        return handle_static(request, &state.cfg, tail);
    }

    // Manual routing
    match (method, url) {
        ("GET", "/api/public/captcha") => api_captcha(&mut state.maybe_captcha),
        ("GET", "/api/public/configuration") => api_configuration(&state.cfg),
        ("GET", "/api/auth/test") => api_auth_test(request, &state.tokens, &state.cfg),
        ("POST", "/api/auth/token") => api_auth_token_handler(
            request,
            &mut state.cfg,
            &mut state.maybe_captcha,
            &mut state.tokens,
        ),
        ("POST", "/api/setPassword") => {
            api_set_password(request, &mut state.cfg, &mut state.tokens)
        }
        ("POST", "/api/mcp") => {
            // Validate Content-Type
            let content_type = request.header("Content-Type").unwrap_or("");
            if !content_type.contains("application/json") {
                return crate::mcp::json_rpc_error_response(
                    serde_json::Value::Null,
                    -32700,
                    "Content-Type must be application/json",
                    None,
                );
            }

            // Check authentication
            if let Err(e) = check_authentication(request, &state.tokens, &state.cfg) {
                return make_api_response(Err(HTTPError::Authentication(e)));
            }

            // Handle MCP request(s)
            crate::mcp::handle_mcp_body(request.body(), &mut state.commands, &state.cfg)
        }
        _ => response_with_status_code(response_text("Not Found"), 404),
    }
}

fn redirect_root_to_index_html(cfg: &CommandLine) -> HttpResponseType {
    if cfg.www_ui_enable {
        response_redirect_301(format!("{}static/index.html", cfg.http_base_path))
    } else {
        response_with_status_code(
            response_text("<html><body>Service Unavailable!</body></html>"),
            403,
        )
    }
}

fn handle_static(_request: &RequestWrapper, cfg: &CommandLine, tail: &str) -> HttpResponseType {
    // Try external static directory first
    if cfg.www_ui_enable
        && cfg
            .www_static_directory
            .as_ref()
            .map(|d| d.is_dir())
            .unwrap_or(false)
    {
        let file_path = cfg.www_static_directory.as_ref().unwrap().join(tail);
        if file_path.exists() && file_path.is_file() {
            if let Ok(data) = std::fs::read(&file_path) {
                // Simple mime type detection
                let mime_type = match file_path.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html",
                    Some("css") => "text/css",
                    Some("js") => "text/javascript",
                    Some("json") => "application/json",
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("png") => "image/png",
                    Some("ico") => "image/x-icon",
                    Some("ttf") => "font/ttf",
                    _ => "application/octet-stream",
                };
                return response_from_data(mime_type, data);
            }
        }
    }

    // Fall back to internal static files
    if let Some((bytes, maybe_mime_type)) = www::handle_static(tail.to_string()) {
        let mime_type = maybe_mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
        response_from_data(&mime_type, bytes)
    } else {
        response_with_status_code(response_text("Not Found"), 404)
    }
}

fn api_captcha(maybe_captcha: &mut Option<captcha::Captcha>) -> HttpResponseType {
    if let Some(captcha) = maybe_captcha {
        let (id, _, png_image) = captcha.generate(true);
        make_api_response_ok_with_result(serde_json::json!({"id": id, "image": png_image}))
    } else {
        make_api_response(Err(HTTPError::Authentication(
            HTTPAuthenticationError::Captcha(
                std::io::Error::from(std::io::ErrorKind::Unsupported).to_string(),
            ),
        )))
    }
}

fn api_configuration(cfg: &CommandLine) -> HttpResponseType {
    let config_map = cfg.www_configuration_map.clone();
    make_api_response_ok_with_result(serde_json::Value::Object(config_map.into_iter().fold(
        serde_json::Map::new(),
        |mut acc, item| {
            acc.insert(item.0, serde_json::Value::String(item.1));
            acc
        },
    )))
}

fn api_auth_test(
    request: &RequestWrapper,
    tokens: &HashMap<String, usize>,
    cfg: &CommandLine,
) -> HttpResponseType {
    match check_authentication(request, tokens, cfg) {
        Ok(_) => make_api_response_ok(),
        Err(e) => make_api_response(Err(HTTPError::Authentication(e))),
    }
}

fn api_auth_token_handler(
    request: &RequestWrapper,
    cfg: &mut CommandLine,
    maybe_captcha: &mut Option<captcha::Captcha>,
    tokens: &mut HashMap<String, usize>,
) -> HttpResponseType {
    let authorization_value = request.header("Authorization").unwrap_or("");
    // Parse form data manually from POST body
    let form: HashMap<String, String> = if request.method() == "POST" {
        match serde_urlencoded::from_bytes::<Vec<(String, String)>>(request.body()) {
            Ok(data) => data.into_iter().collect(),
            Err(_) => HashMap::new(),
        }
    } else {
        HashMap::new()
    };

    match authentication_with_basic(
        request,
        cfg,
        maybe_captcha,
        authorization_value.to_string(),
        form,
    ) {
        Err(error) => make_api_response(Err(HTTPError::Authentication(error))),
        Ok(_) => {
            let token = utils::to_sha512(uuid::Uuid::new_v4().to_string());
            let token_timeout = cfg.http_auth_token_timeout;
            let timestamp = time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize
                + token_timeout;
            tokens.insert(token.clone(), timestamp);
                make_api_response_with_headers(
                    Ok(serde_json::json!({ "token": token })),
                    Some(vec![(
                        std::borrow::Cow::Borrowed("Set-Cookie"),
                        std::borrow::Cow::Owned(format!(
                            "mcpd_token={}; Path=/; Max-Age={}; SameSite=None; Secure;",
                            token, token_timeout
                        )),
                    )]),
                )
        }
    }
}

fn api_set_password(
    request: &RequestWrapper,
    cfg: &mut CommandLine,
    tokens: &mut HashMap<String, usize>,
) -> HttpResponseType {
    match check_authentication(request, tokens, cfg) {
        Ok(_) => {
            // Extract token for potential invalidation on wrong previous password
            let token = extract_token(request).ok();

            let input: SetPassword = match serde_json::from_slice::<SetPassword>(request.body()) {
                Ok(p) => p,
                Err(_) => {
                    return make_api_response(Err(HTTPError::Deserialize(
                        "Invalid JSON".to_string(),
                    )))
                }
            };
            match try_set_password(cfg, input, tokens, token) {
                Ok(_) => make_api_response_ok(),
                Err(e) => make_api_response(Err(HTTPError::API(e))),
            }
        }
        Err(e) => make_api_response(Err(HTTPError::Authentication(e))),
    }
}

fn check_authentication(
    request: &RequestWrapper,
    tokens: &HashMap<String, usize>,
    cfg: &CommandLine,
) -> Result<(), HTTPAuthenticationError> {
    if cfg.http_auth_password_sha512.is_none() {
        return Ok(());
    }

    let token = extract_token(request)?;
    authentication_with_token(tokens, token, cfg)
}

fn extract_token(request: &RequestWrapper) -> Result<String, HTTPAuthenticationError> {
    // Try cookie first
    if let Some(cookie_header) = request.header("Cookie") {
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if cookie.starts_with("mcpd_token=") {
                return Ok(cookie
                    .strip_prefix("mcpd_token=")
                    .unwrap_or("")
                    .to_string());
            }
        }
    }

    // Try Authorization header
    if let Some(auth_header) = request.header("Authorization") {
        let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
        if parts.len() >= 2 && parts[0] == "Bearer" {
            return Ok(parts[1].to_string());
        }
        return Err(HTTPAuthenticationError::InvalidBasicAuthentication {
            header_value: auth_header.to_string(),
        });
    }

    Err(HTTPAuthenticationError::TokenNotFound)
}

fn authentication_with_basic(
    request: &RequestWrapper,
    cfg: &CommandLine,
    maybe_captcha: &mut Option<captcha::Captcha>,
    authorization_value: String,
    form: HashMap<String, String>,
) -> Result<(), HTTPAuthenticationError> {
    if cfg.http_auth_password_sha512.is_none() && cfg.http_auth_username.is_empty() {
        return Ok(());
    };
    if cfg.http_auth_password_sha512.is_none() || cfg.http_auth_username.is_empty() {
        return Err(HTTPAuthenticationError::UsernameOrPasswordIsNotSet);
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
            let decoded_string = String::from_utf8(decoded_username_password).map_err(|_| {
                HTTPAuthenticationError::UsernameOrPasswordIsNotFound {
                    data: username_password.to_string(),
                }
            })?;
            match decoded_string.splitn(2, ':').collect::<Vec<&str>>()[..] {
                [username, password] => {
                    if username == cfg.http_auth_username {
                        // Read X-MCPD-PASSWORD-HASHED header (default to "false")
                        let password_hashed_header = request
                            .header("X-MCPD-PASSWORD-HASHED")
                            .unwrap_or("false")
                            .to_lowercase();
                        let is_password_hashed = password_hashed_header == "true";

                        // Hash password with SHA256 if needed
                        let password_sha256 = if is_password_hashed {
                            password.to_string()
                        } else {
                            utils::to_sha256(password)
                        };

                        tracing::debug!(
                            msg = "New client provided authentication credentials",
                            username = username,
                            password_hashed = is_password_hashed,
                        );

                        // Verify password using bcrypt
                        let password_hash = cfg
                            .http_auth_password_sha512
                            .as_ref()
                            .ok_or(HTTPAuthenticationError::UsernameOrPasswordIsNotSet)?;
                        let password_valid = utils::verify_bcrypt(&password_sha256, password_hash)
                            .map_err(|_| HTTPAuthenticationError::InvalidUsernameOrPassword)?;

                        if password_valid {
                            if maybe_captcha.is_none() {
                                return Ok(());
                            };
                            return if form.len() == 1 {
                                let (key, value) = form
                                    .into_iter()
                                    .next()
                                    .ok_or(HTTPAuthenticationError::InvalidCaptchaForm)?;
                                if let Some(captcha) = maybe_captcha {
                                    if captcha.compare_and_update(
                                        key.to_string(),
                                        value,
                                        cfg.http_auth_captcha_case_sensitive,
                                    ) {
                                        Ok(())
                                    } else {
                                        Err(HTTPAuthenticationError::InvalidCaptcha {})
                                    }
                                } else {
                                    Err(HTTPAuthenticationError::InvalidCaptcha {})
                                }
                            } else {
                                Err(HTTPAuthenticationError::InvalidCaptchaForm {})
                            };
                        };
                    } else {
                        tracing::debug!(
                            msg = "Client attempted authentication with unknown username",
                            username = username,
                        );
                    };
                    Err(HTTPAuthenticationError::InvalidUsernameOrPassword)
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

fn authentication_with_token(
    tokens: &HashMap<String, usize>,
    token: String,
    cfg: &CommandLine,
) -> Result<(), HTTPAuthenticationError> {
    if cfg.http_auth_password_sha512.is_none() {
        return Ok(());
    }
    if token.is_empty() {
        return Err(HTTPAuthenticationError::TokenNotFound);
    };
    if let Some(ref api_token) = cfg.http_auth_api_token {
        if &token == api_token {
            return Ok(());
        }
    }
    if let Some(expire_time) = tokens.get(token.as_str()) {
        if expire_time
            > &(time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize)
        {
            return Ok(());
        };
        Err(HTTPAuthenticationError::TokenExpired)
    } else {
        Err(HTTPAuthenticationError::InvalidToken)
    }
}

fn try_set_password(
    cfg: &mut CommandLine,
    password: SetPassword,
    tokens: &mut HashMap<String, usize>,
    token: Option<String>,
) -> Result<HttpResponseType, HTTPAPIError> {
    if password.password.is_empty() {
        return Err(HTTPAPIError::EmptyPassword);
    };

    // Verify previous password if provided

    if let Some(previous_password) = password.previous_password {
        // Hash previous password with SHA256 if needed (uses same hash flag as new password)
        let previous_password_sha256 = if password.hash {
            previous_password
        } else {
            utils::to_sha256(&previous_password)
        };

        // Verify previous password against current password
        let current_password_hash = cfg
            .http_auth_password_sha512
            .as_ref()
            .ok_or(HTTPAPIError::InvalidPreviousPassword)?;
        let previous_password_valid =
            utils::verify_bcrypt(&previous_password_sha256, current_password_hash)
                .map_err(|_| HTTPAPIError::InvalidPreviousPassword)?;

        if !previous_password_valid {
            // Invalidate token if previous password is wrong
            if let Some(ref token_str) = token {
                tokens.remove(token_str);
            }
            return Err(HTTPAPIError::InvalidPreviousPassword);
        }
    } else {
        return Err(HTTPAPIError::PreviousPasswordRequired);
    }

    let password_file = if let Some(pf) = cfg.http_auth_password_file.clone() {
        pf
    } else {
        return Err(HTTPAPIError::NoPasswordFile);
    };

    // Hash password with SHA256 if needed
    let password_sha256 = if password.hash {
        password.password
    } else {
        utils::to_sha256(&password.password)
    };

    // Generate bcrypt hash from SHA256-hashed password
    let password_bcrypt = utils::hash_bcrypt(&password_sha256, 12).map_err(|reason| {
        HTTPAPIError::SaveNewPassword {
            message: format!("Failed to hash password with bcrypt: {}", reason),
        }
    })?;

    std::fs::write(password_file, password_bcrypt.clone()).map_err(|reason| {
        HTTPAPIError::SaveNewPassword {
            message: reason.to_string(),
        }
    })?;
    cfg.http_auth_password_sha512 = Some(password_bcrypt);
    Ok(make_api_response_ok())
}

fn make_api_response_ok() -> HttpResponseType {
    make_api_response_with_header_and_stats(Ok(serde_json::Value::Null), None, None, None)
}

fn make_api_response_ok_with_result(result: serde_json::Value) -> HttpResponseType {
    make_api_response_with_header_and_stats(Ok(result), None, None, None)
}

fn make_api_response(result: Result<serde_json::Value, HTTPError>) -> HttpResponseType {
    make_api_response_with_header_and_stats(result, None, None, None)
}

fn make_api_response_with_headers(
    result: Result<serde_json::Value, HTTPError>,
    maybe_headers: Option<
        Vec<(
            std::borrow::Cow<'static, str>,
            std::borrow::Cow<'static, str>,
        )>,
    >,
) -> HttpResponseType {
    make_api_response_with_header_and_stats(result, maybe_headers, None, None)
}

fn make_api_response_with_header_and_stats(
    result: Result<serde_json::Value, HTTPError>,
    maybe_headers: Option<
        Vec<(
            std::borrow::Cow<'static, str>,
            std::borrow::Cow<'static, str>,
        )>,
    >,
    maybe_statistics: Option<CommandStats>,
    maybe_status_code: Option<u16>,
) -> HttpResponseType {
    let mut body = json!(
        {
            "ok": if let Some(ref status_code) = maybe_status_code {
                *status_code == 200
            } else {
                result.is_ok()
            }
        }
    );
    body.as_object_mut().unwrap().insert(
        "result".to_string(),
        result
            .clone()
            .or_else::<serde_json::Value, _>(|error| {
                Ok(serde_json::Value::String(error.to_string()))
            })
            .unwrap(),
    );
    if let Some(statistics) = maybe_statistics {
        body.as_object_mut().unwrap().insert(
            "statistics".to_string(),
            serde_json::to_value(&statistics).unwrap(),
        );
    };
    let status_code = if let Some(status_code) = maybe_status_code {
        status_code
    } else if let Err(ref error) = result {
        error.http_status_code()
    } else {
        200
    };
    if let Err(error) = result {
        body.as_object_mut().unwrap().insert(
            "code".to_string(),
            serde_json::Value::Number(serde_json::Number::from(error.http_error_code())),
        );
    };
    let mut response = response_with_status_code(
        response_text(serde_json::to_string(&body).unwrap()),
        status_code,
    );

    // Set Content-Type header
    response.headers_mut().insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json; charset=utf-8"),
    );

    // Add custom headers
    if let Some(headers) = maybe_headers {
        for (name, value) in headers {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .unwrap_or_else(|_| HeaderName::from_static("x-custom-header"));
            let header_value =
                HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static(""));
            response.headers_mut().append(header_name, header_value);
        }
    };
    response
}

// Old handle_rejection and http_logging functions removed - rouille handles errors differently
