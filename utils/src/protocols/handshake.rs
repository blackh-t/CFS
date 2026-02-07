use log::info;
use rand::{Rng, rngs::OsRng};
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey},
    pkcs8::LineEnding,
};
use tokio::net::TcpStream;

use crate::{
    protocols::lvf::{recv_message, send_message},
    types::AppResult,
};

pub struct Connection {
    pub socket: TcpStream,
    pub key: Vec<u8>,
}

pub struct Handshake {}

impl Handshake {
    pub async fn client(mut socket: TcpStream) -> AppResult<Connection> {
        send_message(&mut socket, "HELLO".as_bytes()).await?;

        info!("Waiting for Public key");
        let rsa_pub = recv_message(&mut socket).await?;
        let rsa_pub = String::from_utf8(rsa_pub)?;
        let client_public_key = RsaPublicKey::from_pkcs1_pem(&rsa_pub)?;

        info!("Generate AES-256");
        let mut aes_key = [0u8; 32];
        let mut rng = OsRng;
        OsRng.fill(&mut aes_key);

        let encrypt_key = client_public_key.encrypt(&mut rng, Pkcs1v15Encrypt, &aes_key)?;
        send_message(&mut socket, &encrypt_key).await?;
        info!("Sent encrypted Session Key to Server.");

        Ok(Connection {
            socket,
            key: aes_key.to_vec(),
        })
    }

    pub async fn server(mut socket: TcpStream) -> AppResult<Connection> {
        info!("Etablish handshake...");

        let msg = recv_message(&mut socket).await?;
        let msg = String::from_utf8(msg).unwrap_or_default();
        if msg != "HELLO" {
            return Err("Protocol mismatch: Expected HELLO".into());
        }

        info!("Generate RSA");
        let mut rng = rand::rngs::OsRng;
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let pub_key = priv_key.to_public_key();

        info!("Send pub_pem");
        let pub_pem = pub_key.to_pkcs1_pem(LineEnding::LF)?;
        send_message(&mut socket, pub_pem.as_bytes()).await?;

        info!("Waiting for Encrypted AES Key...");
        let encrypted_aes_key = recv_message(&mut socket).await?;
        let aes_key = priv_key.decrypt(Pkcs1v15Encrypt, &encrypted_aes_key)?;

        Ok(Connection {
            socket,
            key: aes_key,
        })
    }
}
