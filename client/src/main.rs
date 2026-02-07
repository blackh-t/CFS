use std::{
    env,
    io::{self, Write},
};

use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::Pkcs7, generic_array::GenericArray};
use log::{debug, info, warn};
use rand::{Rng, rngs::OsRng, thread_rng};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs1::DecodeRsaPublicKey};
use tokio::net::TcpStream;
use utils::{
    logger::init_logger,
    protocol::{Connection, recv_message, send_message},
    types::{Aes256CbcDec, AppResult},
};

#[tokio::main]
async fn main() -> AppResult<()> {
    init_logger();
    let port = env::var("CFS_PORT").expect("CFS_PORT not found");
    let ip = env::var("CFS_IP").expect("CFS_IP not found");

    let server_addr = format!("{}:{}", ip, port);
    let socket = TcpStream::connect(&server_addr).await?;
    info!("Connecting to {}...", server_addr);

    let connection = handshake(socket).await?;
    info!("Secure connection established!");

    // User input
    print!("Enter the filename to download: ");
    io::stdout().flush()?;

    let mut filename = String::new();
    io::stdin().read_line(&mut filename)?;

    let filename = filename.trim();
    if filename.is_empty() {
        println!("No filename entered. Exiting.");
        return Ok(());
    }

    // Requesting file content.
    download_from(connection.socket, connection.key, filename).await?;

    Ok(())
}

async fn handshake(mut socket: TcpStream) -> AppResult<Connection> {
    send_message(&mut socket, "HELLO".as_bytes()).await?;

    info!("Waiting for Public key");
    let rsa_pub = recv_message(&mut socket).await?;
    let rsa_pub = String::from_utf8(rsa_pub)?;
    let client_public_key = RsaPublicKey::from_pkcs1_pem(&rsa_pub)?;

    info!("Generate AES-256");
    let mut aes_key = [0u8; 32];
    let mut rng = OsRng;
    thread_rng().fill(&mut aes_key);

    let encrypt_key = client_public_key.encrypt(&mut rng, Pkcs1v15Encrypt, &aes_key)?;
    send_message(&mut socket, &encrypt_key).await?;
    info!("Sent encrypted Session Key to Server.");

    Ok(Connection {
        socket,
        key: aes_key.to_vec(),
    })
}

async fn download_from(mut socket: TcpStream, key: Vec<u8>, filename: &str) -> AppResult<()> {
    send_message(&mut socket, filename.as_bytes()).await?;
    debug!("Share key: {:?}", key);

    let mut cipher_data = recv_message(&mut socket).await?;
    if cipher_data.is_empty() {
        warn!("File not found");
        return Ok(());
    }

    let (iv, ciphertext) = cipher_data.split_at_mut(16);
    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(iv);
    let cipher = Aes256CbcDec::new(&key, &iv);

    let data = cipher.decrypt_padded_mut::<Pkcs7>(ciphertext).unwrap();

    info!("got file {}", String::from_utf8(data.to_vec()).unwrap());

    Ok(())
}
