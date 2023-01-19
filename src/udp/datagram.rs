use std::{net::{ToSocketAddrs, SocketAddr}};

#[cfg(feature = "serde")]
use serde::{Serialize, de::DeserializeOwned};

/// Wrapper around a tuple that holds a [Vec<u8>] representing the data and an optional [SocketAddr] that represents the destination or source address of the datagram
#[derive(Debug, Clone)]
pub struct Datagram(pub Vec<u8>, pub Option<SocketAddr>);

impl<A: ToSocketAddrs> From<(Vec<u8>, A)> for Datagram {
    fn from((data, dest): (Vec<u8>, A)) -> Self {
        let mut addrs = dest.to_socket_addrs().expect("Unable to convert socket address");
        Self(data, addrs.next())
    }
}

impl From<Vec<u8>> for Datagram {
    fn from(data: Vec<u8>) -> Self {
        Self(data, None)
    }
}

impl Datagram {

    pub fn new<A: ToSocketAddrs>(data: Vec<u8>, dest: Option<A>) -> Self {
        let mut addr: Option<SocketAddr> = None;
        if let Some(sa) = dest {
            let mut addrs = sa.to_socket_addrs().expect("Unable to convert socket address");
            addr = addrs.next();
        }
        Self(data, addr)
    }

    /// Creates a new [Datagram] with json data from a serializable data structure
    /// 
    /// # Errors
    /// 
    /// This function will return an error if the serialization fails
    #[cfg(feature = "json")]
    pub fn as_json<T, A>(object: &T, dest: A) -> std::io::Result<Datagram>
    where
        T: Serialize,
        A: ToSocketAddrs,
    {
        let string = serde_json::to_string(object)?;
        Ok((string.into_bytes(), dest).into())
    }

    #[cfg(feature = "json")]
    pub fn get_json<T: DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_slice(&self.0).ok()
    }

}

impl ToString for Datagram {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.0).to_string()
    }
}