mod protocol;
mod streams;

use crate::Query;
use protocol::types::StreamId;
use std::sync::Arc;
use streams::{StreamHandle, StreamsManager};
use tokio::io::{BufReader, BufWriter};
use tokio::net::{TcpStream, ToSocketAddrs};

struct Connection {
    streams_manager: Arc<StreamsManager>,
    sender_channel: tokio::sync::mpsc::Sender<(protocol::Request<'static>, StreamId)>,
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
        let startup_request = protocol::Request::Startup;
        startup_request.write(1, &mut tcp_writer).await?;

        // Receive response
        /*
        let response: protocol::Response = protocol::Response::read(&mut tcp_reader).await ?;
        match response {
            protocol::Response::Ready => {/* Ok connection succesfull */},
            _ => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to connect to server - response was not Ready"))
        };
        */

        let streams_manager: Arc<StreamsManager> = StreamsManager::new();

        // Start response receiver task
        {
            let streams_manager = streams_manager.clone();
            tokio::spawn(async move {
                let mut tcp_reader = tcp_reader; // Explicitly move tcp_reader into async task
                loop {
                    // read response
                    // let response: protocol::Response = protocol::Response::read(&mut tcp_reader).await ?;
                    //streams_manager.on_response_received(response);
                }
            });
        }

        // Start request sender task
        let (sender_channel_sender, mut sender_channel_receiver): (
            tokio::sync::mpsc::Sender<(protocol::Request<'static>, StreamId)>,
            tokio::sync::mpsc::Receiver<(protocol::Request<'static>, StreamId)>,
        ) = tokio::sync::mpsc::channel(1);
        {
            let streams_manager = streams_manager.clone();
            tokio::spawn(async move {
                let mut tcp_writer = tcp_writer;
                while let Some((request, stream_id)) = sender_channel_receiver.recv().await {
                    request.write(stream_id, &mut tcp_writer).await.unwrap();
                }
            });
        }

        return Ok(Connection {
            streams_manager,
            sender_channel: sender_channel_sender,
        });
    }

    pub async fn query(
        &mut self,
        query_to_perform: Query,
    ) -> Result<protocol::Response, std::io::Error> {
        let request: protocol::Request = protocol::Request::Query("hehe"); // = protocol::Request::from_query(query_to_perform, request_handle.get_stream_id());

        let stream_handle: StreamHandle = self.streams_manager.register_stream().await;

        self.schedule_request_send(request, stream_handle.get_stream_id())
            .await;
        stream_handle.mark_request_sent();

        return stream_handle.get_response().await;
    }

    async fn schedule_request_send(
        &mut self,
        request: protocol::Request<'static>,
        stream_id: StreamId,
    ) {
        if let Err(_) = self.sender_channel.send((request, stream_id)).await {
            panic!("oops ending request failed"); //TODO make graceful
        }
    }
}
