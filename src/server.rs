use std::{net::{IpAddr, Ipv4Addr}, str::FromStr};

use base64::{Engine, prelude::BASE64_STANDARD};
#[cfg(not(target_os = "macos"))]
use defguard_wireguard_rs::WGApi;
use defguard_wireguard_rs::{InterfaceConfiguration, WireguardInterfaceApi, key::Key, net::IpAddrMask};
use x25519_dalek::{PublicKey, StaticSecret};
use crate::config::{PeerConfig, ServerConfig};
use crate::error::RelayError;

#[derive(Clone, Debug, PartialEq)]
pub struct PeerAddress {
    pub ip_address: Ipv4Addr,
    mask_bits: u32,
}

#[derive(Clone, Debug)]
pub struct Peer {
    pub label: String,
    pub address: PeerAddress,
    pub port: u16,
}

impl PeerAddress {
    pub fn new(ip_address: Ipv4Addr, cidr: u8) -> Self {
        Self {
            ip_address,
            mask_bits: u32::MAX >> cidr,
        }
    }

    pub fn next_address(&self) -> Option<Self> {
        let next_bits = self.ip_address.to_bits() + 1;
        if (next_bits & self.mask_bits) > 0 {
            let ip_address = Ipv4Addr::from_bits(next_bits);
            Some(Self {
                ip_address,
                mask_bits: self.mask_bits,
            })
        } else {
            None
        }
    }
}

pub struct Server {
    pub public_key: PublicKey,
    pub address: IpAddr,
    pub cidr: u8,
    pub port: u16,
    wgapi: WGApi,
}

impl Server {
    pub fn create(config: &ServerConfig) -> anyhow::Result<Self> {
        let ifname: String = if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
            "wg0".into()
        } else {
            "utun3".into()
        };

        #[cfg(not(target_os = "macos"))]
        let wgapi = WGApi::<defguard_wireguard_rs::Kernel>::new(ifname.clone())?;
        #[cfg(target_os = "macos")]
        let wgapi = WGApi::<defguard_wireguard_rs::Userspace>::new(ifname.clone())?;

        wgapi.create_interface()?;

        let secret = StaticSecret::random();
        let prvkey = BASE64_STANDARD.encode(secret.to_bytes());

        let addr_mask = IpAddrMask::from_str(&config.ip_range)?;
        let address = addr_mask.ip.clone();
        let cidr = addr_mask.cidr;
        let public_key = PublicKey::from(&secret);

        let interface_config = InterfaceConfiguration {
            name: ifname.clone(),
            prvkey,
            addresses: vec![addr_mask],
            port: config.listen_port as u32,
            peers: vec![],
            mtu: None,
        };

        // apply initial interface configuration
        #[cfg(not(windows))]
        wgapi.configure_interface(&interface_config)?;
        #[cfg(windows)]
        wgapi.configure_interface(&interface_config, &[])?;

        log::info!("Created Wireguard interface: {} {}/{}", BASE64_STANDARD.encode(public_key.as_bytes()), &address, cidr);

        Ok(Self {
            public_key,
            address,
            cidr,
            port: config.listen_port,
            wgapi,
        })
    }

    pub fn create_peers(&self, config: &Vec<PeerConfig>) -> anyhow::Result<Vec<Peer>> {
        let mut peer_address = if let IpAddr::V4(addr) = self.address {
            PeerAddress::new(addr, self.cidr)
        } else {
            return Err(RelayError::Ipv4required.into());
        };

        let mut peers = Vec::new();

        for cfg in config {
            if let Some(next_address) = peer_address.next_address() {
                peer_address = next_address;
                let key = Key::from_str(&cfg.public_key)?;
                let mut wg_peer = defguard_wireguard_rs::host::Peer::new(key);
                let peer_addr_mask = IpAddrMask::from_str(&format!("{}/32", &peer_address.ip_address))?;
                wg_peer.allowed_ips.push(peer_addr_mask);
                self.wgapi.configure_peer(&wg_peer)?;

                peers.push(Peer {
                    label: cfg.label.clone(),
                    address: peer_address.clone(),
                    port: cfg.port,
                });
                log::info!("Created peer:  {} {}", &cfg.label, &peer_address.ip_address);
            } else {
                return Err(RelayError::OutOfAddresses.into());
            }
        }

        Ok(peers)
    }

    pub fn dispose(&self) -> anyhow::Result<()> {
        self.wgapi.remove_interface()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, str::FromStr};
    use defguard_wireguard_rs::net::IpAddrMask;

    use crate::server::PeerAddress;

    #[test]
    fn test_next_address() {
        let addr_mask = IpAddrMask::from_str("10.8.0.1/30").unwrap();
        if let IpAddr::V4(addr) = addr_mask.ip {
            let first_addr = PeerAddress::new(
                addr, addr_mask.cidr,
            );
            let next_addr = first_addr.next_address();
            assert!(next_addr.is_some());
            let next_addr = next_addr.unwrap();
            assert_eq!("10.8.0.2", &format!("{}", &next_addr.ip_address));
            let last_addr = next_addr.next_address();
            assert!(last_addr.is_some());
            let last_addr = last_addr.unwrap();
            assert_eq!("10.8.0.3", &format!("{}", &last_addr.ip_address));
            assert_eq!(last_addr.next_address(), None);
        }
    }
}
