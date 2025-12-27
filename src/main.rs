use std::sync::Arc;

use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use p2p_relay::{config::Configuration, proxy, server::Server};

const DEFAULT_CONFIG_FILE: &str = "config.toml";

#[tokio::main]
async fn main() {
    let config: Configuration = Figment::new()
        .merge(Toml::file(DEFAULT_CONFIG_FILE))
        .merge(Env::prefixed("P2P_"))
        .extract()
        .unwrap();

    env_logger::init();

    let shutdown = tokio::signal::ctrl_c();
    let server = Server::create(&config.server).unwrap();
    let peers = Arc::new(
      server.create_peers(&config.peers).unwrap()
    );

    tokio::select! {
        _ = proxy::start(&config.proxy, peers) => {
            log::error!("Proxy failure"); // shouldn't happen
        }
        _ = shutdown => {
            println!("Shutting down...");
            if let Err(e) = server.dispose() {
                log::error!("Error removing Wireguard interface: {}", e);
            }
        }
    }
}
