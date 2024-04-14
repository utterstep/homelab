use std::{ops::Deref, sync::Arc};

use derive_getters::Getters;
use eyre::{Result, WrapErr};
use secrecy::SecretString;
use serde::Deserialize;

#[derive(Getters, Debug, Clone)]
pub struct Config {
    inner: Arc<ConfigInner>,
}

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Deserialize, Getters, Debug)]
pub struct ConfigInner {
    /// The address to bind to
    bind_to: String,
    /// Clickhouse server host
    ch_host: String,
    /// Clickhouse user
    ch_user: String,
    /// Clickhouse password
    ch_password: SecretString,
    /// Clickhouse database
    ch_database: String,
    /// Service name of the application, which logs are being processed
    service_name: String,
    /// Environment of the application, which logs are being processed
    /// (e.g. "production", "staging", "development")
    environment: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        envy::from_env::<ConfigInner>()
            .wrap_err("Failed to load configuration")
            .map(|inner| Self {
                inner: Arc::new(inner),
            })
    }
}
