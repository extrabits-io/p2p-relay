use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use p2p_relay::{config::Configuration, proxy};

const DEFAULT_CONFIG_FILE: &str = "config.toml";

#[tokio::main]
async fn main() {
    let config: Configuration = Figment::new()
        .merge(Toml::file(DEFAULT_CONFIG_FILE))
        .merge(Env::prefixed("P2P_"))
        .extract()
        .unwrap();

    // let mut builder = env_logger::Builder::new();
    // builder.target(env_logger::Target::Stdout).init();
    env_logger::init();

    proxy::start(&config.proxy).await.unwrap();
}
