use std::sync::Arc;
use tokio::net::UdpSocket;

use super::{Error, Datagram};



pub struct Sender {
    socket: Arc<UdpSocket>
}

impl Sender {

    pub fn new(socket: Arc<UdpSocket>) -> Self {
        Self { socket }
    }

    pub async fn send(&self, datagram: Datagram) -> Result<usize, Error> {
        match datagram.1 {
            Some(dest) => {
                self.socket.send_to(&datagram.0, &dest).await.map_err(|e| Error::Send(e))
            },
            None => {
                self.socket.send(&datagram.0).await.map_err(|e| Error::Send(e))
            },
        }
    }

}