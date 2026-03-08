use std::{ops::RangeInclusive, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_private_key_path")]
    pub private_key_path: PathBuf,
    pub port_range: RangeInclusive<u16>,
}

#[derive(Debug, Deserialize)]
pub struct PeerConfig {
    pub label: String,
    pub public_key: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub server: ServerConfig,
    pub peers: Vec<PeerConfig>,
}

fn default_private_key_path() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .with_file_name("private.key")
}
