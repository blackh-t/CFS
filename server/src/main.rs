use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7, generic_array::GenericArray};
use log::{debug, error, info, warn};
use rand::{Rng, thread_rng};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs1::EncodeRsaPublicKey, pkcs8::LineEnding};
use std::{env, io};
use tokio::{
    fs,
    net::{TcpListener, TcpStream},
};
use utils::{
    logger::init_logger,
    protocol::{Connection, recv_message, send_message},
    types::{Aes256CbcEnc, AppResult},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    init_logger();
    let port = env::var("CFS_PORT").expect("CFS_PORT not found");
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {:?}", listener.local_addr());

    loop {
        if let Ok((socket, ip)) = listener.accept().await {
            info!("connected to {}", ip);
            tokio::spawn(async move {
                if let Err(e) = connection_handler(socket).await {
                    error!("Handler Err: {:?}", e);
                }
            });
        }
    }
}

async fn connection_handler(socket: TcpStream) -> AppResult<()> {
    let mut conn = handshake(socket).await?;
    info!("Secure connection established!");

    // File Transfer
    loop {
        let packet = match recv_message(&mut conn.socket).await {
            Ok(data) => data,
            Err(_) => break,
        };

        let filenme = String::from_utf8(packet)?;
        let path = format!("data/{}", filenme);
        info!("Requesting filenme {}", filenme);
        debug!("Share key: {:?}", conn.key);

        let mut file_content = match fs::read(path).await {
            Ok(content) => content,
            Err(_) => {
                warn!("File not found");
                return Ok(());
            }
        };

        let file_len = file_content.len();
        let block_size = 16;
        file_content.resize(file_len + block_size, 0);

        let mut iv = [0u8; 16];
        thread_rng().fill(&mut iv);

        let key = GenericArray::from_slice(&conn.key);
        let iv = GenericArray::from_slice(&iv);
        let cipher = Aes256CbcEnc::new(&key, &iv);

        let encrypt_data = cipher
            .encrypt_padded_mut::<Pkcs7>(&mut file_content, file_len)
            .unwrap();

        let mut final_packet = Vec::with_capacity(16 + encrypt_data.len());
        final_packet.extend_from_slice(&iv);
        final_packet.extend_from_slice(encrypt_data);

        send_message(&mut conn.socket, &final_packet).await?;
        info!("File Transfered.");
    }

    Ok(())
}

async fn handshake(mut socket: TcpStream) -> AppResult<Connection> {
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
