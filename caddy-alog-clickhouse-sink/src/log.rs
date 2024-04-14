use std::collections::HashMap;

use derive_getters::{Dissolve, Getters};
use serde::{Deserialize, Serialize};

pub mod db;

/// General information about the log entry
#[derive(Serialize, Deserialize, Dissolve, Getters, Debug)]
pub struct Meta {
    /// The log level
    level: String,
    /// The timestamp of the log entry
    #[serde(rename = "ts")]
    timestamp: f64,
    /// The logger name
    logger: String,
    /// The log message
    #[serde(rename = "msg")]
    message: String,
}

/// A HashMap of headers
pub type Headers = HashMap<String, Vec<String>>;

/// Information about the request, handled by the Caddy server
#[derive(Serialize, Deserialize, Dissolve, Getters, Debug)]
pub struct RequestInfo {
    remote_ip: String,
    remote_port: String,
    client_ip: Option<String>,
    #[serde(rename = "proto")]
    protocol: String,
    method: String,
    host: String,
    uri: String,
    headers: Headers,
}

#[derive(Serialize, Deserialize, Dissolve, Getters, Debug)]
pub struct AccessLogEntry {
    #[serde(flatten)]
    meta: Meta,
    request: RequestInfo,
    bytes_read: u64,
    user_id: Option<String>,
    duration: f64,
    size: u64,
    status: u16,
    #[serde(rename = "resp_headers")]
    response_headers: Headers,
}
