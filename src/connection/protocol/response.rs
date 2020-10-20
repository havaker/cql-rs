use super::Header;
use super::StreamId;
use bytes::Buf;
use tokio::io::AsyncReadExt;

#[derive(Debug, PartialEq)]
pub enum Response {
    Ready,
    Error(ErrorMessage),
    Result,

    Invalid,
}

#[derive(PartialEq, Default, Debug)]
pub struct ErrorMessage {
    message: String,
    code: u32,
}

impl Response {
    pub async fn read<T: AsyncReadExt + Unpin>(
        reader: &mut T,
    ) -> Result<(Response, StreamId), std::io::Error> {
        let header = Header::deserialize(reader).await?;

        if header.protocol_version != 0x84 {
            return Err(make_invalid_response_error());
        }

        let mut response = Response::from_opcode(header.opcode);
        if response == Response::Invalid {
            return Err(make_invalid_response_error());
        }

        let mut body_buf = vec![0u8; header.body_length as usize];

        reader.read_exact(&mut body_buf).await?;
        response.parse_body(body_buf.as_slice())?;

        return Ok((response, header.stream_id));
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
                // ignore additional info, that error body can have
                if body.remaining() < string_len as usize {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::io::Builder;

    #[test]
    #[ignore]
    fn test_startup_scylla_response() {
        let ready_response = [0x84, 0, 0, 0, 2, 0, 0, 0, 0];

        tokio_test::block_on(async {
            let mut mock = Builder::new().read(&ready_response).build();
            let (rsp, stream_id) = Response::read(&mut mock).await.unwrap();

            assert_eq!(rsp, Response::Ready);
            assert_eq!(stream_id, 0);
        });
    }

    #[test]
    fn test_error_response_reading() {
        let error_response = [
            0x84, 0x00, 0x7f, 0xfe, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x20, 0x00, 0x00,
            0x32, 0x6c, 0x69, 0x6e, 0x65, 0x20, 0x31, 0x3a, 0x30, 0x20, 0x6e, 0x6f, 0x20, 0x76,
            0x69, 0x61, 0x62, 0x6c, 0x65, 0x20, 0x61, 0x6c, 0x74, 0x65, 0x72, 0x6e, 0x61, 0x74,
            0x69, 0x76, 0x65, 0x20, 0x61, 0x74, 0x20, 0x69, 0x6e, 0x70, 0x75, 0x74, 0x20, 0x27,
            0x73, 0x61, 0x64, 0x73, 0x64, 0x61, 0x73, 0x64, 0x27,
        ];

        tokio_test::block_on(async {
            let mut mock = Builder::new().read(&error_response).build();
            let (rsp, stream_id) = Response::read(&mut mock).await.unwrap();

            assert_eq!(
                rsp,
                Response::Error(ErrorMessage {
                    code: 8192,
                    message: String::from("line 1:0 no viable alternative at input \'sadsdasd\'"),
                })
            );
            assert_eq!(stream_id, 32766);
        });
    }
}
