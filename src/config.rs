use std::{ops::RangeInclusive, path::PathBuf};

use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_private_key_path")]
    pub private_key_path: PathBuf,
    /// external port to listen for incoming web requests
    pub listen_port: u16,
    /// port to listen for peer connections
    pub control_port: u16,
    /// allowed range of ports available to peers
    #[serde(deserialize_with = "deserialize_range")]
    pub peer_port_range: RangeInclusive<u16>,
}

#[derive(Debug, Deserialize)]
pub struct PeerConfig {
    pub label: String,
    pub public_key: String,
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

fn deserialize_range<'de, D>(deserializer: D) -> Result<RangeInclusive<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    // Expecting the range to be provided as a string in TOML (e.g., "3000..4000")
    let s = String::deserialize(deserializer)?;
    let parts: Vec<&str> = s.split("..").collect();

    if parts.len() != 2 {
        Err(serde::de::Error::custom(format!(
            "Invalid range format: {}. Expected 'start..end'.",
            s
        )))
    } else {
        let start = u16::from_str(parts[0]).map_err(|_| {
            serde::de::Error::custom(format!("Invalid start port in range: {}", parts[0]))
        })?;
        let end = u16::from_str(parts[1]).map_err(|_| {
            serde::de::Error::custom(format!("Invalid end port in range: {}", parts[1]))
        })?;
        Ok(start..=end)
    }
}
