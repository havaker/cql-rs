pub enum Response {
    Ready,
    Error,
    Result,
}

/*
struct Response

impl Response {
    pub async fn read<T: AsyncReadExt + Unpin>(reader: &mut T) -> Result<u16, std::io::Error> {
        // TODO move writing header to another funciton
        writer.write_u8(0x04).await?; // protocol version for request
        writer.write_u8(0).await?; // no flags
        writer.write_u16(stream_id).await?;
        writer.write_u8(self.opcode()).await?;

        let body = self.body();
        writer.write_u32(body.len() as u32).await?;
        writer.write_all(body.as_slice()).await
    }
}
*/
