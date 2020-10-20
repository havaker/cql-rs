use super::StreamId;
use bytes::Buf;
use tokio::io::AsyncReadExt;

#[derive(PartialEq)]
pub enum Response {
    Ready,
    Error(ScyllaError),
    Result,

    Invalid,
}

#[derive(PartialEq, Default)]
pub struct ScyllaError {
    message: String,
    code: u32,
}

impl Response {
    pub async fn read<T: AsyncReadExt + Unpin>(
        reader: &mut T,
    ) -> Result<(Response, StreamId), std::io::Error> {
        let protocol_version = reader.read_u8().await?;
        if protocol_version != 0x84 {
            return Err(make_invalid_response_error());
        }
        reader.read_u8().await?; // ignore received flags TODO check
        let stream_id: StreamId = reader.read_u16().await?;

        let opcode = reader.read_u8().await?;

        let mut response = Response::from_opcode(opcode);
        if response == Response::Invalid {
            return Err(make_invalid_response_error());
        }

        let body_len = reader.read_u32().await?;
        let mut body_buf = vec![0u8; body_len as usize];

        reader.read_exact(&mut body_buf).await?;
        response.parse_body(body_buf.as_slice())?;

        return Ok((Response::Ready, stream_id));
    }

    // https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L166
    fn from_opcode(opcode: u8) -> Response {
        match opcode {
            0x00 => Self::Error(Default::default()),
            0x02 => Self::Ready,
            0x08 => Self::Result,
            _ => Self::Invalid,
        }
    }

    fn parse_body(&mut self, mut body: &[u8]) -> Result<(), std::io::Error> {
        match self {
            // https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L471
            Self::Error(error) => {
                error.code = body.get_u32();

                // [string] read string with error message
                let string_len = body.get_u16();
                let mut string_content = vec![0u8; string_len as usize];
                if body.remaining() != string_len as usize {
                    return Err(make_invalid_response_error());
                }
                body.copy_to_slice(&mut string_content);

                error.message = String::from_utf8_lossy(string_content.as_slice()).to_string();
                return Ok(());
            }
            Self::Ready => Ok(()),
            Self::Result => Ok(()),
            Self::Invalid => Err(make_invalid_response_error()),
        }
    }
}

fn make_invalid_response_error() -> std::io::Error {
    return std::io::Error::new(std::io::ErrorKind::Other, "Invalid response");
}
