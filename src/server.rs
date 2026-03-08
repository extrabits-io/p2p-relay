use std::{
    fs::File,
    io::{Read, Write},
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use crate::config::ServerConfig;
use base64::{Engine, prelude::BASE64_STANDARD};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Clone, Debug)]
pub struct Peer {
    pub label: String,
    pub port: u16,
}

pub struct Server {
    pub public_key: PublicKey,
    tunnel: bore_cli::server::Server,
}

impl Server {
    pub fn create(config: &ServerConfig) -> anyhow::Result<Self> {
        let (_, public_key) = Self::get_secret(&config.private_key_path)?;
        let pub_key_str = BASE64_STANDARD.encode(public_key.as_bytes());
        let mut tunnel =
            bore_cli::server::Server::new(config.port_range.clone(), Some(&pub_key_str));
        tunnel.set_bind_addr(IpAddr::V4(Ipv4Addr::LOCALHOST));
        tunnel.set_bind_tunnels(IpAddr::V4(Ipv4Addr::LOCALHOST));

        log::info!("Created server:  {}", &pub_key_str);
        Ok(Self { public_key, tunnel })
    }

    pub async fn start(self) -> anyhow::Result<()> {
        self.tunnel.listen().await
    }

    fn get_secret(private_key_path: &PathBuf) -> anyhow::Result<(String, PublicKey)> {
        if private_key_path.is_file() {
            let mut file = File::open(private_key_path)?;
            let mut buff = String::new();
            file.read_to_string(&mut buff)?;
            let mut data: [u8; 32] = [0; 32];
            BASE64_STANDARD.decode_slice(&buff, &mut data)?;
            let secret = StaticSecret::from(data);
            let pubkey = PublicKey::from(&secret);
            Ok((buff, pubkey))
        } else {
            let secret = StaticSecret::random();
            let prvkey = BASE64_STANDARD.encode(secret.to_bytes());
            let mut file = File::create(private_key_path)?;
            file.write_all(prvkey.as_bytes())?;
            let pubkey = PublicKey::from(&secret);
            Ok((prvkey, pubkey))
        }
    }
}
