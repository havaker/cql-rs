use tokio::io::AsyncReadExt;

pub enum Response {
    Ready,
    Error,
    Result,
}

pub struct ResponseWithStreamID {
    response: Response,
    stream_id: u16,
}

impl Response {
    pub async fn read<T: AsyncReadExt + Unpin>(
        reader: &mut T,
    ) -> Result<ResponseWithStreamID, std::io::Error> {
        return Ok(ResponseWithStreamID {
            response: Response::Ready,
            stream_id: 0,
        });
    }
}
