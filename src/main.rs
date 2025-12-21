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

    let server = server::create_interface(&config.server).unwrap();
    let peers = server::create_peers(&config.peers, &server).unwrap();
    proxy::start(&config.proxy, &peers).await.unwrap();
}
