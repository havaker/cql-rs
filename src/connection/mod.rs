mod protocol;
mod streams;
pub mod simple_connection;
pub mod complicated_connection;

pub use simple_connection::Connection;
pub use protocol::response::ErrorMessage;

#[derive(Debug)]
pub enum QueryError {
    IOError(std::io::Error),
    Message(ErrorMessage),
}

impl From<std::io::Error> for QueryError {
    fn from(io_error: std::io::Error) -> QueryError {
        return QueryError::IOError(io_error);
    }
}
