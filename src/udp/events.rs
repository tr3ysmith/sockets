use tokio::sync::broadcast;

use super::datagram::Datagram;



pub type UdpEventRx = broadcast::Receiver<UdpEvent>;
pub type UdpEventTx = broadcast::Sender<UdpEvent>;

#[derive(Debug, Clone)]
pub enum UdpEvent {
    Data(Datagram),
    Close
}
