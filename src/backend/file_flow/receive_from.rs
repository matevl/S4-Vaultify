use crate::backend::file_manager::file_handling::save_binary;
use crate::backend::file_manager::metadata_handling::process_file;
use std::env;
use std::io;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

pub async fn receive() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("TCP Server listening on 127.0.0.1:8080");

    let binary_files_path = env::current_dir()?.join("binary_files");
    std::fs::create_dir_all(&binary_files_path)?;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from {}", addr);

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            match socket.read_to_end(&mut buffer).await {
                Ok(size) => {
                    println!("Received {} bytes from {}", size, addr);
                    let saved_file_name = save_binary(&buffer);
                    println!("File initially saved as {}", saved_file_name);
                    let file_path: PathBuf = env::current_dir()
                        .unwrap()
                        .join("binary_files")
                        .join(&saved_file_name);

                    match process_file(&file_path) {
                        Ok(_) => println!("File successfully processed."),
                        Err(e) => eprintln!("Error processing file: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to read from socket: {}", e),
            }
        });
    }
}
