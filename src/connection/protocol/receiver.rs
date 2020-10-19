use tokio::net::tcp;
use tokio::io::BufReader;
use tokio::io::AsyncReadExt;

pub struct ProtocolReceiver {
    receive_socket: BufReader<tcp::OwnedReadHalf>
}

impl ProtocolReceiver {
    fn new(read_socket: tcp::OwnedReadHalf) -> ProtocolReceiver {
        return ProtocolReceiver{receive_socket: BufReader::new(read_socket)};
    }

    async fn receive_bytes(&mut self, bytes: &mut [u8]) -> Result<(), std::io::Error> {
        self.receive_socket.read_exact(bytes).await ?;
        return Ok(());
    }

    async fn receive_u8(&mut self) -> Result<u8, std::io::Error> {
        let mut val_bytes = [0u8; 1];
        self.receive_bytes(&mut val_bytes).await ?;
        return Ok(u8::from_be_bytes(val_bytes));
    }

    async fn receive_u16(&mut self) -> Result<u16, std::io::Error> {
        let mut val_bytes = [0u8; 2];
        self.receive_bytes(&mut val_bytes).await ?;
        return Ok(u16::from_be_bytes(val_bytes));
    }
}