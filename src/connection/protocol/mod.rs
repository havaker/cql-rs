pub mod request;
pub mod response;
pub mod types;

pub use request::Request;
pub use response::Response;
pub use types::Header;
pub use types::StreamId;

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;

    #[test]
    #[ignore]
    fn test_startup_scylla() {
        let startup_request = Request::Startup;

        tokio_test::block_on(async {
            let mut stream = TcpStream::connect("127.0.0.1:9042").await.unwrap();

            let stream_id: StreamId = 1;
            startup_request.write(stream_id, &mut stream).await.unwrap();
            let (resp, received_stream_id) = Response::read(&mut stream).await.unwrap();
            assert_eq!(resp, Response::Ready);
            assert_eq!(received_stream_id, stream_id);
        });
    }
}
