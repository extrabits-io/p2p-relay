use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use p2p_relay::{config::Configuration, server::Server};
use tracing::{error, info};

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
    let server = Server::create(&config.server, &config.peers).unwrap();

    tokio::select! {
        result = server.start() => {
            match result {
                Ok(()) => {
                    error!("Server stopped unexpectedly");
                    std::process::exit(1);
                }
                Err(err) => {
                    error!("Server failure: {err}");
                    std::process::exit(1);
                }
            }
        }
        _ = shutdown => {
            info!("Shutting down...");
        }
    }
}
