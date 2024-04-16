use bb8::Pool;
use eyre::{Result, WrapErr};
use klickhouse::{ClientOptions, ConnectionManager};
use secrecy::ExposeSecret;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    ch_pool: Pool<ConnectionManager>,
    config: Config,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        let options = ClientOptions {
            username: config.ch_user().to_string(),
            password: config.ch_password().expose_secret().to_string(),
            default_database: config.ch_database().to_string(),
        };
        let manager = ConnectionManager::new(config.ch_host(), options)
            .await
            .wrap_err_with(|| {
                format!(
                    "Failed to create connection manager for config: {:?}",
                    config
                )
            })?;

        let pool = Pool::builder()
            .max_size(20)
            .build(manager)
            .await
            .wrap_err_with(|| {
                format!("Failed to create connection pool for config: {:?}", config)
            })?;

        Ok(Self {
            ch_pool: pool,
            config,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn ch_pool(&self) -> &Pool<ConnectionManager> {
        &self.ch_pool
    }
}
