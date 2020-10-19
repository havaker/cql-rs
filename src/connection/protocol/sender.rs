use tokio::io::AsyncWriteExt;
use tokio::io::BufWriter;
use tokio::net::tcp;

use super::Request;

pub struct ProtocolSender {
    send_socket: BufWriter<tcp::OwnedWriteHalf>,
}

impl ProtocolSender {
    fn new(write_socket: tcp::OwnedWriteHalf) -> ProtocolSender {
        return ProtocolSender {
            send_socket: BufWriter::new(write_socket),
        };
    }

    async fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.send_socket.write_all(bytes).await
    }

    async fn send_header(&mut self, stream_id: u16, req: &Request) -> Result<(), std::io::Error> {
        self.send_socket.write_u8(0x04).await?; // protocol version for request
        self.send_socket.write_u8(0).await?; // flags
        self.send_socket.write_u16_le(stream_id).await?;
        self.send_socket.write_u8(req.opcode()).await?;
    }

    async fn send_request(&mut self, req: Request) -> Result<(), std::io::Error> {
        match req {
            Request::Startup => return self.send_u32(0).await,
            Request::Query => return self.send_u32(0).await,
        }
    }
}
