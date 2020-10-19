use bytes::BufMut;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

pub enum Request {
    Startup,
    Query,
}

const startup_body: &'static str = "";

fn serialize_map(map: HashMap<String, String>) -> Vec<u8> {
    let mut buf = vec![];

    // [string multimap]
    buf.put_u16_le(map.len() as u16); // map.len() elements
    for (k, v) in map {
        buf.put_u16_le(k.len() as u16);
        buf.put_slice(&k[..].as_bytes());

        buf.put_u16_le(v.len() as u16);
        buf.put_slice(&v[..].as_bytes());
    }

    return buf;
}

/*
The body is a [string map] of options. Possible options are:
   - "CQL_VERSION": the version of CQL to use. This option is mandatory and
     currently the only version supported is "3.0.0". Note that this is
     different from the protocol version.
*/

impl Request {
    pub fn opcode(&self) -> u8 {
        match self {
            Self::Startup => 0x01,
            Self::Query => 0x07,
        }
    }

    pub fn body(&self) -> Vec<u8> {
        match self {
            Self::Startup => {
                let mut options: HashMap<String, String> = HashMap::new();
                options.insert("CQL_VERSION".to_string(), "3.0.0".to_owned());
                serialize_map(options)
            }
            Self::Query => Vec::new(),
        }
    }

    pub async fn write<T: AsyncWriteExt + Unpin>(
        &self,
        stream_id: u16,
        writer: &mut T,
    ) -> Result<(), std::io::Error> {
        writer.write_u8(0x04).await?; // protocol version for request
        writer.write_u8(0).await?; // flags
        writer.write_u16_le(stream_id).await?;
        writer.write_u8(self.opcode()).await?;

        let body = self.body();
        writer.write_u32_le(body.len() as u32).await?;
        writer.write_all(body.as_slice()).await
    }
}
