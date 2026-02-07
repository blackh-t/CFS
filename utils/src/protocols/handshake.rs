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
    /// Performs the client-side handshake.
    ///
    /// # Steps:
    /// - Sends `HELLO` to the server.
    /// - Receives the server RSA public key.
    /// - Generates a random AES-256 session key.
    /// - Encrypts the AES key using the server RSA public key.
    /// - Sends the encrypted session key to the server.
    ///
    /// # Parameters:
    /// - `socket`: Connected TCP stream to the server.
    ///
    /// # Returns:
    /// - `Ok(Connection)` : established connection with shared AES-256 session key.
    /// - `Err`            : handshake or cryptographic operation fails.
    ///
    /// # Notes:
    /// - Uses RSA (PKCS#1 v1.5) for key exchange.
    /// - AES key is generated locally and kept in memory.
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

    /// Performs the server-side handshake.
    ///
    /// # Steps:
    /// - Waits for `HELLO` from the client.
    /// - Generates an RSA key pair.
    /// - Sends the RSA public key to the client.
    /// - Receives the encrypted AES-256 session key.
    /// - Decrypts the session key using the RSA private key.
    ///
    /// # Parameters:
    /// - `socket`: Connected TCP stream to the client.
    ///
    /// # Returns:
    /// - `Ok(Connection)` : established connection with shared AES-256 session key.
    /// - `Err`            : protocol mismatch or cryptographic failure.
    ///
    /// # Notes:
    /// - Expects strict protocol order (`HELLO` first).
    /// - RSA private key is kept only for the duration of the handshake.
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
