use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use crate::config::ServerConfig;
use base64::{Engine, prelude::BASE64_STANDARD};
use ed25519_dalek::{
    SigningKey,
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, spki::der::pem::LineEnding},
};
use rand::rngs::OsRng;
use tracing::info;

#[derive(Clone, Debug)]
pub struct Peer {
    pub label: String,
    pub port: u16,
}

pub struct Server {
    tunnel: bore_cli::server::Server,
}

impl Server {
    pub fn create(config: &ServerConfig) -> anyhow::Result<Self> {
        let signing_key = Self::get_signing_key(&config.private_key_path)?;
        let pub_key_der = signing_key.verifying_key().to_public_key_der()?;
        let pub_key_str = BASE64_STANDARD.encode(pub_key_der.as_bytes());
        let mut tunnel =
            bore_cli::server::Server::new(config.port_range.clone(), Some(signing_key));
        tunnel.set_bind_addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        tunnel.set_bind_tunnels(IpAddr::V4(Ipv4Addr::LOCALHOST));

        info!("Created server:  {}", &pub_key_str);
        Ok(Self { tunnel })
    }

    pub async fn start(self) -> anyhow::Result<()> {
        self.tunnel.listen().await
    }

    fn get_signing_key(private_key_path: &PathBuf) -> anyhow::Result<SigningKey> {
        if private_key_path.is_file() {
            let key = SigningKey::read_pkcs8_pem_file(private_key_path)?;
            Ok(key)
        } else {
            let mut csprng = OsRng;
            let key = SigningKey::generate(&mut csprng);
            key.write_pkcs8_pem_file(private_key_path, LineEnding::LF)?;
            Ok(key)
        }
    }
}
