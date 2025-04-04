use rustls::ServerConfig;
use rustls_pemfile;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls_pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls_pki_types::{CertificateDer};
use std::convert::TryInto;
use std::{error::Error, fs::File, io::BufReader, sync::Arc};
use std::ptr::read;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

pub fn tcp_server() {
    let certs = load_certs("certificate/cert.pem")?;
    let key = load_key("certificate/key.pem")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, PrivateKeyDer::Pkcs8(key))?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind("127.0.0.1:445")?;
    println!("Serveur TLS en écoute sur 127.0.0.1:445");

    loop {
        let (stream, addr) = listener.accept()?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            println!("Nouvelle connexion de {}", addr);

            match acceptor.accept(stream).await {
                Ok(mut tls_stream) => {
                    let mut buffer = [0; 4096];

                    if let Ok(first_data)=serde_json::from_str(buffer){
                        let ()
                    }
                    match tls_stream.read(&mut buffer).await {
                        Ok(size) if size > 0 => {
                            let request = String::from_utf8_lossy(&buffer[..size]);
                            let parts: Vec<&str> = request.splitn(2, '|').collect();

                            if parts.len() == 2 {
                                let path = parts[0].trim();
                                let data = parts[1].as_bytes();

                                if let Ok(mut file) = File::create(path) {
                                    if let Err(e) = file.write_all(data) {
                                        eprintln!("Erreur d’écriture : {}", e);
                                    } else {
                                        println!("Fichier sauvegardé : {}", path);
                                    }
                                } else {
                                    eprintln!("Impossible de créer le fichier : {}", path);
                                }
                            } else {
                                eprintln!("Format invalide reçu depuis {}", addr);
                            }
                        }
                        Ok(_) => {
                            eprintln!("Connexion vide depuis {}", addr);
                        }
                        Err(e) => {
                            eprintln!("Erreur de lecture TLS : {}", e);
                        }
                    }

                }
                Err(e) => eprintln!("Échec handshake TLS avec {} : {}", addr, e),
            }
        });
    }
}

fn load_certs(path: &str) -> Result<Vec<CertificateDer>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = certs(&mut reader).collect::<Result<Vec<_>, _>>()?;
    Ok(certs)
}

pub fn load_key(path: &str) -> Result<PrivatePkcs8KeyDer<'static>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let keys = pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;
    keys.into_iter()
        .next()
        .ok_or_else(("Invalid key {}", path))
}
