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
use crate::backend::account_manager::account_server::{VaultInfo, JWT, ROOT, SESSION_CACHE};
use crate::backend::{VAULTS_DATA, VAULT_USERS_DIR};
use crate::backend::aes_keys::crypted_key::encrypt;

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

                    if let Ok(nb) = tls_stream.read(&mut buffer).await {
                        if let Ok((jwt, vault_info, filename)) = serde_json::from_str::<(JWT, VaultInfo, String)>(buffer) {
                            let key = {
                                let sessions = SESSION_CACHE.lock().unwrap();
                                sessions.get(&jwt.session_id).expect("no key loaded").clone()
                            };

                            let tmp_path = format!("{}/{}{}/{}", ROOT.to_str().unwrap(), VAULTS_DATA, vault_info.name, filename);
                            let mut file = File::create(tmp_path).expect("create_vault");

                            while let Ok(size)= tls_stream.read(&mut buffer).await {
                                let content = String::from_utf8_lossy(&buffer[..size]);
                                let encrypt_content = encrypt(content, &key);

                                file.write_all(encrypt_content.as_bytes()).await.expect("write_file");
                            }
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
