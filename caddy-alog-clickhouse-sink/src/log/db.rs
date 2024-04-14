use derive_getters::Getters;
use klickhouse::{DateTime64, Row, Tz, Uuid};
use serde::{Deserialize, Serialize};

use crate::log::{AccessLogEntry, Headers};

#[derive(Serialize, Deserialize, Row, Getters, Debug)]
pub struct DbAccessLogEntry {
    // Added by the sink service
    id: Uuid,
    service: String,
    environment: String,
    // Caddy logger meta
    level: String,
    logger_timestamp: DateTime64<3>,
    logger: String,
    message: String,
    // Caddy request info
    remote_ip: String,
    remote_port: String,
    client_ip: Option<String>,
    protocol: String,
    method: String,
    host: String,
    uri: String,
    headers: Headers,
    // Caddy request handling info
    bytes_read: u64,
    user_id: Option<String>,
    duration: f64,
    size: u64,
    status: u16,
    // Caddy response headers
    response_headers: Headers,
}

impl DbAccessLogEntry {
    pub fn new(
        id: uuid::Uuid,
        service: &str,
        environment: &str,
        access_log_entry: AccessLogEntry,
    ) -> Self {
        let (meta, request, bytes_read, user_id, duration, size, status, response_headers) =
            access_log_entry.dissolve();
        let (level, logger_timestamp, logger, message) = meta.dissolve();
        let (remote_ip, remote_port, client_ip, protocol, method, host, uri, headers) =
            request.dissolve();
        let millis = logger_timestamp as u64 * 1_000 + (logger_timestamp.fract() * 1_000.0) as u64;
        let logger_timestamp = DateTime64(Tz::UTC, millis);

        Self {
            id,
            service: service.to_string(),
            environment: environment.to_string(),
            level,
            logger_timestamp,
            logger,
            message,
            remote_ip,
            remote_port,
            client_ip,
            protocol,
            method,
            host,
            uri,
            headers,
            bytes_read,
            user_id,
            duration,
            size,
            status,
            response_headers,
        }
    }
}
