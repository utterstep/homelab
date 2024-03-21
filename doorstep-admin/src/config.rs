use std::path::{Path, PathBuf};

use ipnet::IpNet;
use serde::Deserialize;

fn default_background_width() -> u32 {
    960
}

fn default_background_height() -> u32 {
    540
}

#[derive(Deserialize)]
pub struct Config {
    listen_addr: String,
    allowed_configuration_from: IpNet,
    backgrounds_dir: PathBuf,
    #[serde(default = "default_background_width")]
    background_width: u32,
    #[serde(default = "default_background_height")]
    background_height: u32,
}

/// Getters
impl Config {
    pub fn listen_addr(&self) -> &str {
        &self.listen_addr
    }

    pub fn allowed_configuration_from(&self) -> &IpNet {
        &self.allowed_configuration_from
    }

    pub fn backgrounds_dir(&self) -> &Path {
        &self.backgrounds_dir
    }

    pub fn background_width(&self) -> u32 {
        self.background_width
    }

    pub fn background_height(&self) -> u32 {
        self.background_height
    }
}
