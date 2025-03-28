use std::{fs::File, io::BufReader, path::Path, sync::Arc, error::Error};
use std::convert::TryInto;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio_rustls::TlsConnector;
use rustls::{ClientConfig, RootCertStore};
use rustls_pemfile;
use rustls_pki_types::{CertificateDer, ServerName};

pub async fn send<P: AsRef<Path>>(file_path: P) -> Result<(), Box<dyn Error>> {
    let mut cert_reader = BufReader::new(File::open("certificate/cert.pem")?);
    let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<_, _>>()?;

    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_parsable_certificates(certs);

    let config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();
    let connector = TlsConnector::from(Arc::new(config));

    let tcp_stream = TcpStream::connect("127.0.0.1:8080").await?;

    let domain: ServerName = "localhost".try_into()
        .map_err(|_| "Nom de domaine TLS invalide")?;

    let mut tls_stream = connector.connect(domain, tcp_stream).await?;

    let buffer = std::fs::read(file_path)?;
    tls_stream.write_all(&buffer).await?;
    println!("File sent with TLS");

    Ok(())
}