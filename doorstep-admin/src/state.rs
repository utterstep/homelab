use std::sync::Arc;

use tokio::sync::Mutex;

use crate::config::Config;

mod init;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub background: Arc<Mutex<Option<Vec<u8>>>>,
}

pub struct AppStateBuilder {
    config: Option<Arc<Config>>,
}

impl AppStateBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(Arc::new(config));
        self
    }

    pub fn build(self) -> AppState {
        AppState {
            config: self.config.expect("config is required"),
            background: Arc::new(Mutex::new(None)),
        }
    }
}
