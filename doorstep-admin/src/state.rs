use std::sync::Arc;

use tokio::sync::RwLock;

use crate::config::Config;

mod background;
mod init;

pub use self::background::BackgroundState;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub background: Arc<RwLock<Option<BackgroundState>>>,
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
            background: Arc::new(RwLock::new(None)),
        }
    }
}
