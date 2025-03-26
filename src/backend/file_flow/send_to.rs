use crate::backend::file_manager::file_handling::read_bytes;
use std::error::Error;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn send<P: AsRef<Path>>(file_path: P) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connexion successfully established");
    let buffer = read_bytes(file_path);
    stream.write_all(&buffer.unwrap().as_slice()).await?;
    println!("Sent");
    Ok(())
}
