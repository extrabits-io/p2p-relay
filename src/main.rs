use defguard_wireguard_rs::WireguardInterfaceApi;
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use p2p_relay::{config::Configuration, proxy, server};

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
    let server = server::create_interface(&config.server).unwrap();
    let peers = server::create_peers(&config.peers, &server).unwrap();

    tokio::select! {
        _ = proxy::start(&config.proxy, &peers) => {
            log::error!("Proxy failure"); // shouldn't happen
        }
        _ = shutdown => {
            println!("Shutting down...");
            if let Err(e) = server.wgapi.remove_interface() {
                log::error!("Error removing Wireguard interface: {}", e);
            }
        }
    }
}
