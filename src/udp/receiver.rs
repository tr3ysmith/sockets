use std::{sync::Arc, net::SocketAddr};

use tokio::{net::UdpSocket, sync::mpsc};
use tracing::{error};

#[derive(Debug, Clone)]
pub enum Event {
    /// On data received from the socket
    Data(Vec<u8>, SocketAddr),
    /// On close of the socket
    Closed,
    /// On drop of the receiver
    Dropped
}

pub struct Receiver {
    rx: mpsc::Receiver<Event>
}

impl Receiver {

    pub fn new(socket: Arc<UdpSocket>) -> Self {
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(run_receiver_loop(socket, tx));
        Self { rx }
    }

    pub async fn on_event(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

}

async fn run_receiver_loop(socket: Arc<UdpSocket>, tx: mpsc::Sender<Event>) {

    loop {

        if let Err(e) = socket.readable().await {
            error!("UDP receiver error: {}", e);
            break;
        }

        let mut buf = [0; 16384];

        match socket.try_recv_from(&mut buf) {
            Ok((len, addr)) => {

                let bytes = buf[..len].to_vec();

                // Read into the buffer and transmit to the event channel
                let event_tx_result = tx.send(Event::Data(bytes, addr)).await;
        
                // If we were unable to transmit to the event channel, then break the loop
                if event_tx_result.is_err() {
                    break;
                }

            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            },
            Err(e) => {
                error!("Socket receive failed: {:?}", e);
                let _ = tx.send(Event::Closed);
                break;
            }
        }

    }

    let _ = tx.send(Event::Dropped);

}