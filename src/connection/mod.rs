use tokio::net::{TcpStream, ToSocketAddrs};
use crate::Query;
use std::task::{Context, Waker, Poll};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::collections::VecDeque;

mod protocol;

pub struct Connection {
    streams_manager: Arc<Mutex<StreamsManager>>,
    sender: Mutex<ProtocolSender>,
}

impl Connection {
    pub async fn new<A: ToSocketAddrs>(address: A) -> Result<Self, std::io::Error> {
        let tcp_stream: TcpStream = tokio::net::TcpStream::connect(address).await ?;
        let (read_half, write_half) = tcp_stream.into_split();

        
    }

    pub async fn query(&mut self, query_to_perform: Query) -> Result<protocol::Response, std::io::Error> {
        let request_handle: RequestHandle = self.streams_manager.lock().register_request_handle();

        let request: protocol::Request = protocol::Request::from_query(query_to_perform, request_handle.get_stream_id());
        self.sender.lock().send_request(request).await ?;

        return request_handle.get_response().await ?;
    }
}

async fn test() {
    let connection = Connection::new("127.0.0.1:1234").await.unwrap();
}

struct Stream {
    waker: Option<Waker>,
    received_response: Option<protocol::Response>
}

type StreamId = u16;

struct StreamsManager {
    streams: Vec<Mutex<Stream>>,
    free_streams: Vec<usize>,
    stream_ids_semaphore: Semaphore,
    receiver: Option<protocol::ProtocolReceiver>,
}

impl StreamsManager {
    async fn register_request_handle() 
}

struct ResponseFuture {
    stream_id: StreamId,
    streams_manager: Arc<Mutex<StreamsManager>>,
    state: ResponseFutureState,
}

enum ResponseFutureState {
    Waiting,
    Receiving {
        receiver: protocol::ProtocolReceiver,
    },
    Done
}

impl Future for ResponseFuture {
    type Output = protocol::Response;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        return Poll::Pending;
    }
}