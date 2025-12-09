use std::str::FromStr;

#[cfg(not(target_os = "macos"))]
use defguard_wireguard_rs::WGApi;
use defguard_wireguard_rs::{key::Key, net::IpAddrMask, WireguardInterfaceApi};
use crate::config::{PeerConfig, ServerConfig};

pub struct Peer {
    pub label: String,
    pub address: String,
}

pub fn create_interface(config: &ServerConfig) -> anyhow::Result<()> {
    let ifname: String = if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        "wg0".into()
    } else {
        "utun3".into()
    };

    #[cfg(not(target_os = "macos"))]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(ifname.clone())?;
    #[cfg(target_os = "macos")]
    let mut wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(ifname.clone())?;

    wgapi.create_interface()?;

    let host = wgapi.read_interface_data()?;
    log::info!("WireGuard interface: {host:#?}");

    let host_addr = IpAddrMask::from_str(&config.ip_range)?;
    
    Ok(())
}

pub fn create_peers(config: &Vec<PeerConfig>) -> anyhow::Result<Vec<Peer>> {
    let peers = Vec::new();

    for cfg in config {
        let key = Key::from_str(&cfg.public_key)?;
        let mut wg_peer = defguard_wireguard_rs::host::Peer::new(key);
        
    }
    Ok(peers)
}