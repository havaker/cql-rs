use crate::Query;
use super::protocol::types::StreamId;
use super::protocol::{Request, Response};
use std::sync::Arc;
use super::streams::{StreamHandle, StreamsManager};
use tokio::io::AsyncWriteExt;
use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};
use crate::QueryError;

/*
    This is an attempt at more advanced Connection handler supporting multiple simulatous streams automatically
    It would handle future drops before completion and beutiful error handling
    It is bugged and non functional
    Stream.state erronously gets set to Free somewhere before reading response
*/

pub struct Connection {
    streams_manager: Arc<StreamsManager>,
    sender_channel: tokio::sync::mpsc::Sender<(Request, StreamId)>,
}

impl Connection {
    pub async fn new<A: ToSocketAddrs>(address: A) -> Result<Self, std::io::Error> {
        let tcp_stream: TcpStream = tokio::net::TcpStream::connect(address).await?;
        let (tcp_read_half, tcp_write_half) = tcp_stream.into_split();
        let (mut tcp_reader, mut tcp_writer) = (
            BufReader::new(tcp_read_half),
            BufWriter::new(tcp_write_half),
        );
        // Send startup request
        let startup_request = Request::Startup;
        startup_request.write(1, &mut tcp_writer).await?;
        tcp_writer.flush().await?;

        // Receive response
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

        let streams_manager: Arc<StreamsManager> = StreamsManager::new();

        // Start response receiver task
        {
            let streams_manager = streams_manager.clone();
            tokio::spawn(async move {
                let mut tcp_reader = tcp_reader; // Explicitly move tcp_reader into async task
                loop {
                    // read response
                    match Response::read(&mut tcp_reader).await {
                        Ok((response, stream_id)) => {
                            streams_manager.on_response_received(response, stream_id)
                        }
                        Err(io_error) => {
                            streams_manager.on_receive_error(io_error);
                            break;
                        }
                    }
                }
            });
        }

        // Start request sender task
        let (sender_channel_sender, mut sender_channel_receiver): (
            tokio::sync::mpsc::Sender<(Request, StreamId)>,
            tokio::sync::mpsc::Receiver<(Request, StreamId)>,
        ) = tokio::sync::mpsc::channel(1);
        {
            //let streams_manager = streams_manager.clone();
            tokio::spawn(async move {
                let mut tcp_writer = tcp_writer;
                while let Some((request, stream_id)) = sender_channel_receiver.recv().await {
                    request.write(stream_id, &mut tcp_writer).await.unwrap();
                    tcp_writer.flush().await.unwrap();
                    println!("Sent a query!");
                }
            });
        }

        return Ok(Connection {
            streams_manager,
            sender_channel: sender_channel_sender,
        });
    }

    pub async fn query(&self, query_to_perform: Query) -> Result<(), QueryError> {
        let request: Request = Request::Query(query_to_perform.get_query_text());

        let stream_handle: StreamHandle = self.streams_manager.register_stream().await;

        self.schedule_request_send(request, stream_handle.get_stream_id())
            .await;
        stream_handle.mark_request_sent();

        match stream_handle.get_response().await {
            Err(io_error) => return Err(QueryError::IOError(io_error)),
            Ok(Response::Result) => return Ok(()),
            Ok(Response::Error(message)) => return Err(QueryError::Message(message)),
            _ => panic!("Strange response!"),
        };
    }

    async fn schedule_request_send(&self, request: Request, stream_id: StreamId) {
        if let Err(_) = self.sender_channel.clone().send((request, stream_id)).await {
            panic!("oops ending request failed"); //TODO make graceful
        }
    }
}

#[cfg(test)]
#[test]
fn test_connect_to_scylla() {
    tokio_test::block_on(async {
        let mut conn = Connection::new("172.17.0.4:9042").await.unwrap();
        println!("Connected!");

        //println!("{:?}", conn.query(Query::new("sadsdasd")).await.unwrap());

        let query1 = conn.query(Query::new("sadsdasd"));
        //let query2 = conn.query(Query::new("sadsdasd"));

        query1.await;
        //futures::join!(query1, query2);
    });
}
