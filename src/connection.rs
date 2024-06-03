use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read_data(
        &mut self,
    ) -> Result<Option<BytesMut>, Box<dyn std::error::Error + Send + Sync>> {
        loop {
            if let Some(data) = self.parse_data()? {
                return Ok(Some(data));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    fn parse_data(&mut self) -> Result<Option<BytesMut>, Box<dyn std::error::Error + Send + Sync>> {
        if !self.buffer.is_empty() {
            let len = self.buffer.len();
            let data = self.buffer.split_to(len);
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub async fn write_data(
        &mut self,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
