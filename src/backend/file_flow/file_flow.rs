use crate::backend::file_manager::file_handling::save_binary;
use crate::backend::file_manager::metadata_handling::process_file;
use anyhow::{anyhow, Result};
use rustls::ServerConfig;
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_pki_types::PrivatePkcs8KeyDer;
use rustls_pki_types::{CertificateDer, ServerName};
use std::convert::TryInto;
use std::env;
use std::path::PathBuf;
use std::{error::Error, fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::TlsConnector;

pub async fn receive() -> Result<()> {
    let certs = load_certs("certificate/cert.pem")?;
    let key = load_key("certificate/key.pem")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, rustls_pki_types::PrivateKeyDer::Pkcs8(key))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("TCP Server listening on 127.0.0.1:8080");

    let binary_files_path = env::current_dir()?.join("binary_files");
    std::fs::create_dir_all(&binary_files_path)?;

    loop {
        let (stream, addr) = listener.accept().await?;
        let acceptor = acceptor.clone(); // important : clone pour le déplacer dans le spawn

        tokio::spawn(async move {
            println!("New connection from {}", addr);

            match acceptor.accept(stream).await {
                Ok(mut tls_stream) => {
                    let mut buffer = Vec::new();

                    match tls_stream.read_to_end(&mut buffer).await {
                        Ok(size) => {
                            println!("Received {} bytes from {}", size, addr);
                            let saved_file_name = save_binary(&buffer);
                            println!("File initially saved as {}", saved_file_name);

                            let file_path = env::current_dir()
                                .unwrap_or_else(|_| PathBuf::from("."))
                                .join("binary_files")
                                .join(&saved_file_name);

                            match process_file(&file_path) {
                                Ok(_) => println!("File successfully processed."),
                                Err(e) => eprintln!("Error processing file: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to read from TLS stream: {}", e),
                    }
                }
                Err(e) => eprintln!("TLS handshake failed with {}: {}", addr, e),
            }
        });
    }
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = certs(&mut reader).collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

pub fn load_key(path: &str) -> Result<PrivatePkcs8KeyDer<'static>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let keys = pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;
    keys.into_iter()
        .next()
        .ok_or_else(|| anyhow!("Invalid key {}", path))
}

pub async fn send<P: AsRef<Path>>(file_path: P) -> Result<(), Box<dyn Error>> {
    let mut cert_reader = BufReader::new(File::open("certificate/cert.pem")?);
    let certs: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut cert_reader).collect::<Result<_, _>>()?;

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_parsable_certificates(certs);

    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    let tcp_stream = TcpStream::connect("127.0.0.1:8080").await?;

    let domain: ServerName = "localhost"
        .try_into()
        .map_err(|_| "Nom de domaine TLS invalide")?;

    let mut tls_stream = connector.connect(domain, tcp_stream).await?;

    let buffer = std::fs::read(file_path)?;
    tls_stream.write_all(&buffer).await?;
    println!("File sent with TLS");

    tls_stream.shutdown().await?;
    println!("TLS connection closed gracefully");

    Ok(())
}
