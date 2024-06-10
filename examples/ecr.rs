use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut client = TcpStream::connect("127.0.0.1:8080").await?;
    let message = "ACQ103TID88824756\0\nECR0110X/0";

    client.write_all(message.as_bytes()).await?;

    let mut buf = [0; 4 * 1024];
    let n = client.read(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..n])?;
    println!("{}", response);

    Ok(())
}
