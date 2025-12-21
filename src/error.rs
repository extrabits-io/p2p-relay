use thiserror::Error;

#[derive(Debug, Error)]
pub enum RelayError {
    #[error("Only IP v4 addresses are supported")]
    Ipv4required,
    #[error("No more available addresses in address range")]
    OutOfAddresses,
}