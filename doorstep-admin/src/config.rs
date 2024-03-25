use std::path::{Path, PathBuf};

use ipnet::IpNet;
use serde::Deserialize;

pub const HASH_SEED: u32 = 3140;

fn default_background_width() -> u32 {
    960
}

fn default_background_height() -> u32 {
    540
}

#[derive(Deserialize)]
pub struct Config {
    listen_addr: String,
    admin_subnet: IpNet,
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

    pub fn admin_subnet(&self) -> &IpNet {
        &self.admin_subnet
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
