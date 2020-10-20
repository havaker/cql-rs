use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct SimpleConnection {
    stream: TcpStream,
    reader: BufReader,
    writer: BufWriter,
}

impl SimpleConnection {
    pub async fn new<A: ToSocketAddrs>(address: A) -> Result<Self, std::io::Error> {
        let tcp_stream: TcpStream = tokio::net::TcpStream::connect(address).await?;
        let (tcp_read_half, tcp_write_half) = tcp_stream.into_split();
        let (mut tcp_reader, mut tcp_writer) = (
            BufReader::new(tcp_read_half),
            BufWriter::new(tcp_write_half),
        );

        let startup_request = Request::Startup;
        startup_request.write(1, &mut tcp_writer).await?;
        tcp_writer.flush().await?;

        let (response, _stream_id) = Response::read(&mut tcp_reader).await?;
        match response {
            Response::Ready => { /* Ok connection succesfull */ }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to connect to server - response was not Ready",
                ))
            }
        };

        return Ok(SimpleConnection {
            stream: tcp_stream,
            reader: tcp_reader,
            writer: tcp_writer,
        });
    }
}
