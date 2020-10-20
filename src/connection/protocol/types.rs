use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub type StreamId = i16;

#[derive(Default)]
pub struct Header {
    pub protocol_version: u8,
    pub flags: u8,
    pub stream_id: StreamId,
    pub opcode: u8,
    pub body_length: u32,
}

impl Header {
    pub async fn serialize<T: AsyncWriteExt + Unpin>(
        &self,
        writer: &mut T,
    ) -> Result<(), std::io::Error> {
        writer.write_u8(self.protocol_version).await?;
        writer.write_u8(self.flags).await?;
        writer.write_i16(self.stream_id).await?;
        writer.write_u8(self.opcode).await?;
        return writer.write_u32(self.body_length).await;
    }

    pub async fn deserialize<T: AsyncReadExt + Unpin>(
        reader: &mut T,
    ) -> Result<Self, std::io::Error> {
        let mut h: Header = Default::default();
        h.protocol_version = reader.read_u8().await?;
        h.flags = reader.read_u8().await?;
        h.stream_id = reader.read_i16().await?;
        h.opcode = reader.read_u8().await?;
        h.body_length = reader.read_u32().await?;
        return Ok(h);
    }
}
