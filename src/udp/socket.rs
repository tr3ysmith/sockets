use std::{net::SocketAddr, sync::Arc};

use tokio::sync::{mpsc, broadcast};
use tracing::{info, error};

use crate::udp::{receiver::{Receiver, self}, sender::Sender, UdpEvent};

use super::{Error, events::UdpEventTx, datagram::Datagram, UdpEventRx};

#[cfg(feature = "json")]
use std::net::ToSocketAddrs;

#[cfg(feature = "json")]
use serde::Serialize;

pub struct UdpSocket {
    tx: mpsc::Sender<Datagram>,
    event_tx: UdpEventTx
}

impl UdpSocket {

    /// Creates a new [UdpSocket]
    ///
    /// `src` - Local address of the socket
    pub fn new(src: SocketAddr) -> Self {
        // Create the message channels for sending and receiving UDP packets
        let (tx, sender_rx) = mpsc::channel(32);
        let (event_tx, _) = broadcast::channel(32);


        // Start the socket run loop
        tokio::spawn(run_socket(src, sender_rx, event_tx.clone()));

        Self { tx, event_tx }
    }

    /// Creates a new [UdpSocket] as an instance
    /// This is used by the parent Udp struct which aggregates multiple UDP sockets together
    /// This is done by passing in a shared event channel that this UdpSocket will use
    pub fn new_instance(src: SocketAddr, event_tx: UdpEventTx) -> Self {
        // Create the message channels for sending and receiving UDP packets
        let (tx, sender_rx) = mpsc::channel(32);

        // Start the socket run loop
        tokio::spawn(run_socket(src, sender_rx, event_tx.clone()));

        Self { tx, event_tx }
    }


    /// Sends a datagram packet out
    ///
    /// # Panics
    /// This function will return an error if the socket sender has been dropped.
    pub async fn send(&self, datagram: Datagram) -> Result<(), Error> {
        self.tx.send(datagram).await.map_err(|e| Error::Command(e.to_string()))
    }


    /// Sends an object as json. The object must implement [Serialize]. The provided destination address is where
    /// the json packet will be sent to.
    #[cfg(feature = "json")]
    pub async fn send_json<T, A>(&self, object: &T, dest: A) -> Result<(), Error>
    where
        T: Serialize,
        A: ToSocketAddrs,
    {

        let datagram = Datagram::as_json(object, dest)?;
        self.send(datagram).await
    }

    /// Gets the event channel of the socket
    pub fn get_rx(&self) -> UdpEventRx {
        self.event_tx.subscribe()
    }

}


async fn run_socket(src: SocketAddr, mut sender_rx: mpsc::Receiver<Datagram>, event_tx: UdpEventTx) {


    if let Ok(s) = create_socket(src.clone()) {
        let socket = Arc::new(s);
        let mut receiver = Receiver::new(socket.clone());
        let sender = Sender::new(socket.clone());

        info!("Udp socket bound ({:?})", socket.local_addr().unwrap());

        // This inner loop will sit and handle IO for the socket
        loop {
            tokio::select! {

                // This handles datagram messages coming in from the socket
                recv_result = receiver.on_event() => {
                    match recv_result {
                        Some(event) => {
                            match event {
                                receiver::Event::Data(data, addr) => {
                                    let _ = event_tx.send(UdpEvent::Data((data, addr).into()));
                                },
                                // Udp receiver is closed, meaning the socket died, so we need to break the loop and try again
                                receiver::Event::Closed => {
                                    break;
                                },
                                // UDP Receiver is closed, this is typically intentional, meaning we triggered it
                                receiver::Event::Dropped => {
                                    break;
                                },
                            }
                        },
                        None => {
                            error!("UDP Receiver event channel closed");
                            break;
                        }
                    }
                },

                send_result = sender_rx.recv() => {
                    match send_result {
                        Some(datagram) => {
                            let dst = datagram.1.clone();
                            let result = sender.send(datagram).await;
                            // If there was an error sending this datagram, it means the sender failed, so we need to recycle
                            if let Err(e) = result {
                                error!("Socket ({}) failed to send udp datagram to {:?} - {:?}", socket.local_addr().unwrap(), dst, e);
                                break;
                            }
                        },
                        // The sender channel is closed, meaning that the parent owner of the socket dropped us, so we exit the loop.
                        None => {
                            break;
                        }
                    }
                }

            }
        }

    }
}

/// Creates a socket that has `SO_REUSEADDR` and `SO_REUSEPORT` enabled
/// Once the socket is created, it will be automatically bound to the given address.
/// If multicast options are enabled, then the multicast address will also be joined here.
/// 
/// # Panics
/// This function will panic if the socket cannot be created due to general IO reasons, or if the socket can't
/// be bound to the given address, and finally if the socket fails to join multicast if required.
fn create_socket(address: SocketAddr) -> Result<tokio::net::UdpSocket, Error> {

    let domain = if address.is_ipv4() { socket2::Domain::IPV4 } else { socket2::Domain::IPV6 };

    let socket = socket2::Socket::new(domain, socket2::Type::DGRAM, None)?;
    socket.set_reuse_address(true)?;
    socket.set_reuse_port(true)?;
    socket.set_nonblocking(true)?;

    socket.bind(&socket2::SockAddr::from(address)).map_err(|error| Error::Bind { address, error })?;

    let async_socket = tokio::net::UdpSocket::from_std(socket.into())?;

    Ok(async_socket) 

}