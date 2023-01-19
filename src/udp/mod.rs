use thiserror::Error;

mod socket;
mod datagram;
mod events;
mod receiver;
mod sender;

pub use datagram::Datagram;
pub use events::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to bind socket using address: {address:?} - {error:?}")]
    Bind { address: std::net::SocketAddr, error: std::io::Error },
    #[error("Io error: {0:?}")]
    Io(#[from] std::io::Error),
    #[error("Failed to send data: {0}")]
    Send(std::io::Error),
    #[error("Socket has been closed")]
    Closed,
    #[error("failed to execute command: {0:?}")]
    Command(String),
}