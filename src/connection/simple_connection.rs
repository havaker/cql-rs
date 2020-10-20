use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs, tcp::{OwnedReadHalf, OwnedWriteHalf}};
use super::QueryError;
use crate::Query;
use super::protocol::{Request, Response};
use tokio::io::AsyncWriteExt;

pub struct Connection {
    tcp_reader: BufReader<OwnedReadHalf>,
    tcp_writer: BufWriter<OwnedWriteHalf>,
}

impl Connection {
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

        return Ok(Connection {
            tcp_reader: tcp_reader,
            tcp_writer: tcp_writer,
        });
    }

    pub async fn query(&mut self, query_to_perform: Query) -> Result<(), QueryError> {
        let request: Request = Request::Query(query_to_perform.get_query_text());

        request.write(1, &mut self.tcp_writer).await ?;
        self.tcp_writer.flush().await ?;

        match Response::read(&mut self.tcp_reader).await {
            Err(io_error) => return Err(QueryError::IOError(io_error)),
            Ok((Response::Result, _)) => return Ok(()),
            Ok((Response::Error(message), _)) => return Err(QueryError::Message(message)),
            _ => panic!("Strange response!"),
        };
    }
}
