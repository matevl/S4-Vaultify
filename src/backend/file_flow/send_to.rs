use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;

async fn send() -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    stream.write_all(b"hello world!").await?;

    Ok(())
}