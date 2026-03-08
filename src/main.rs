use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use p2p_relay::{config::Configuration, server::Server};

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

    tokio::select! {
        _ = server.start() => {
            log::error!("Server failure"); // shouldn't happen
        }
        _ = shutdown => {
            log::info!("Shutting down...");
        }
    }
}
