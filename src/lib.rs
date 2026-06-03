pub mod config;
pub mod error;
pub mod router;
pub mod server;

#[derive(Clone, Debug)]
pub struct Peer {
    pub label: String,
    pub port: u16,
    pub last_heartbeat: Option<u64>,
    pub last_latency: Option<u32>,
}
