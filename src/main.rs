use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use tracing::{error, info};
use p2p_relay::{config::Configuration, server::Server};

const DEFAULT_CONFIG_FILE: &str = "config.toml";

#[tokio::main]
async fn main() {
    let config: Configuration = Figment::new()
        .merge(Toml::file(DEFAULT_CONFIG_FILE))
        .merge(Env::prefixed("P2P_"))
        .extract()
        .unwrap();

    tracing_subscriber::fmt::init();

    let shutdown = tokio::signal::ctrl_c();
    let server = Server::create(&config.server).unwrap();

    tokio::select! {
        _ = server.start() => {
            error!("Server failure"); // shouldn't happen
        }
        _ = shutdown => {
            info!("Shutting down...");
        }
    }
}
