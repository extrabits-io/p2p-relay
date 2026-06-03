use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
};

use crate::{
    config::{PeerConfig, ServerConfig},
    router::Router,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use ed25519_dalek::{
    SigningKey, VerifyingKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
        spki::der::pem::LineEnding,
    },
};
use rand::rngs::OsRng;
use tracing::info;

pub struct Server {
    tunnel: p2p_lib::server::Server,
    router: Router,
}

impl Server {
    pub fn create(config: &ServerConfig, peers: &Vec<PeerConfig>) -> anyhow::Result<Self> {
        let signing_key = Self::get_signing_key(&config.private_key_path)?;
        let pub_key_der = signing_key.verifying_key().to_public_key_der()?;
        let pub_key_str = BASE64_STANDARD.encode(pub_key_der.as_bytes());
        let allowed_clients = peers
            .iter()
            .filter_map(|peer| {
                if let Ok(key_bytes) = BASE64_STANDARD.decode(&peer.public_key) {
                    if let Ok(key) = VerifyingKey::from_public_key_der(&key_bytes) {
                        Some(key)
                    } else {
                        tracing::warn!("Invalid peer key bytes");
                        None
                    }
                } else {
                    tracing::warn!("Invalid base-64 encoded peer key");
                    None
                }
            })
            .collect();
        let mut tunnel =
            p2p_lib::server::Server::new(config.peer_port_range.clone(), Some(allowed_clients));
        tunnel.set_bind_addr(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        tunnel.set_bind_tunnels(IpAddr::V4(Ipv4Addr::LOCALHOST));

        let router = Router::new(config.listen_port);

        info!("Created server:  {}", &pub_key_str);
        Ok(Self { tunnel, router })
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let _ = tokio::try_join!(
            tokio::spawn(self.router.start()),
            tokio::spawn(self.tunnel.listen()),
        )?;
        Ok(())
        // need hook to know when peer has connected and when heartbeat is received
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
