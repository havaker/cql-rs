use super::Header;
use super::StreamId;
use bytes::BufMut;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

pub enum Request<'a> {
    Startup,
    Query(&'a str),
}

impl<'a> Request<'a> {
    pub async fn write<T: AsyncWriteExt + Unpin>(
        &self,
        stream_id: StreamId,
        writer: &mut T,
    ) -> Result<(), std::io::Error> {
        let body = self.body();

        let mut header = Header {
            protocol_version: 0x04,
            flags: 0,
            stream_id: stream_id,
            opcode: self.opcode(),
            body_length: body.len() as u32,
        };

        header.serialize(writer).await?;
        return writer.write_all(body.as_slice()).await;
    }

    // https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L166
    fn opcode(&self) -> u8 {
        match self {
            Self::Startup => 0x01,
            Self::Query(_) => 0x07,
        }
    }

    // TODO get rid of vec allocation for every request
    // e.g. split this to 2 funcitons, one for writing, second for size computation
    fn body(&self) -> Vec<u8> {
        match self {
            Self::Startup => {
                // https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L269
                let mut options: HashMap<String, String> = HashMap::new();
                options.insert("CQL_VERSION".to_string(), "3.0.0".to_owned());
                return serialize_map(options);
            }
            Self::Query(q) => serialize_query(q),
        }
    }
}

// https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L244
fn serialize_map(map: HashMap<String, String>) -> Vec<u8> {
    let mut buf = vec![];

    // [short] map length
    buf.put_u16(map.len() as u16);
    for (k, v) in map {
        // [short string] key
        buf.put_u16(k.len() as u16);
        buf.put_slice(&k[..].as_bytes());

        // [short string] value
        buf.put_u16(v.len() as u16);
        buf.put_slice(&v[..].as_bytes());
    }

    return buf;
}

// https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L309
fn serialize_query(query: &str) -> Vec<u8> {
    let mut buf = vec![];

    // [long string] with query
    buf.put_u32(query.len() as u32);
    buf.put_slice(query.as_bytes());

    // [consistency] ONE
    buf.put_u16(0x0001);

    // [byte] flags - none
    buf.put_u8(0);

    return buf;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpStream;
    use tokio_test::io::Builder;

    #[test]
    fn test_startup_serialization() {
        let expected_startup = [
            4u8, 0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 22u8, 0u8, 1u8, 0u8, 11u8, 67u8, 81u8, 76u8,
            95u8, 86u8, 69u8, 82u8, 83u8, 73u8, 79u8, 78u8, 0u8, 5u8, 51u8, 46u8, 48u8, 46u8, 48u8,
        ];

        let req = Request::Startup;
        tokio_test::block_on(async {
            let mut mock = Builder::new().write(&expected_startup).build();
            req.write(0, &mut mock).await.unwrap();
        });
    }

    #[test]
    #[ignore]
    fn test_startup_scylla_response() {
        let req = Request::Startup;
        let expected_response = [0x84, 0, 0, 0, 2, 0, 0, 0, 0];

        tokio_test::block_on(async {
            let mut stream = TcpStream::connect("127.0.0.1:9042").await.unwrap();
            req.write(0, &mut stream).await.unwrap();

            let mut response = expected_response.clone();
            stream.read_exact(&mut response).await.unwrap();

            assert_eq!(expected_response, response);
        });
    }
}
