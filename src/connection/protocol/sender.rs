use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::tcp;

use super::Request;

pub struct ProtocolSender {
    send_socket: BufWriter<tcp::OwnedWriteHalf>,
}
