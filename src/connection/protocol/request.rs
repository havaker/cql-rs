use bytes::BufMut;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

pub enum Request {
    Startup,
    Query,
}

impl Request {
    pub async fn write<T: AsyncWriteExt + Unpin>(
        &self,
        stream_id: u16,
        writer: &mut T,
    ) -> Result<(), std::io::Error> {
        writer.write_u8(0x04).await?; // protocol version for request
        writer.write_u8(0).await?; // no flags
        writer.write_u16(stream_id).await?;
        writer.write_u8(self.opcode()).await?;

        let body = self.body();
        writer.write_u32(body.len() as u32).await?;
        writer.write_all(body.as_slice()).await
    }

    fn opcode(&self) -> u8 {
        match self {
            Self::Startup => 0x01,
            Self::Query => 0x07,
        }
    }

    // TODO get rid of vec allocation for every request
    fn body(&self) -> Vec<u8> {
        match self {
            Self::Startup => {
                // https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L269
                let mut options: HashMap<String, String> = HashMap::new();
                options.insert("CQL_VERSION".to_string(), "3.0.0".to_owned());
                return serialize_map(options);
            }
            Self::Query => Vec::new(),
        }
    }
}

// https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L244
fn serialize_map(map: HashMap<String, String>) -> Vec<u8> {
    let mut buf = vec![];

    buf.put_u16(map.len() as u16);
    for (k, v) in map {
        buf.put_u16(k.len() as u16);
        buf.put_slice(&k[..].as_bytes());

        buf.put_u16(v.len() as u16);
        buf.put_slice(&v[..].as_bytes());
    }

    return buf;
}
