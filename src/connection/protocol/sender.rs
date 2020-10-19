use tokio::net::tcp;
use tokio::io::BufWriter;
use tokio::io::AsyncWriteExt;

pub struct ProtocolSender {
    send_socket: BufWriter<tcp::OwnedWriteHalf>
}

impl ProtocolSender {
    fn new(write_socket: tcp::OwnedWriteHalf) -> ProtocolSender {
        return ProtocolSender{send_socket: BufWriter::new(write_socket)};
    }

    async fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.send_socket.write_all(bytes).await
    }

    async fn send_u16(&mut self, value: u16) -> Result<(), std::io::Error> {
        self.send_bytes(&value.to_be_bytes()).await
    }

    async fn send_u32(&mut self, value: u32) -> Result<(), std::io::Error> {
        self.send_bytes(&value.to_be_bytes()).await
    }

    async fn send_u64(&mut self, value: u64) -> Result<(), std::io::Error> {
        self.send_bytes(&value.to_be_bytes()).await
    }
}