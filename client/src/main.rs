use std::{
    env,
    io::{self, Write},
};

use log::{debug, info, warn};
use tokio::{fs, net::TcpStream};
use utils::{
    cipher::decrypt_data,
    logger::init_logger,
    protocols::{
        handshake::Handshake,
        lvf::{recv_message, send_message},
    },
    types::AppResult,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    init_logger();
    let port = env::var("CFS_PORT").expect("CFS_PORT not found");
    let ip = env::var("CFS_IP").expect("CFS_IP not found");

    let server_addr = format!("{}:{}", ip, port);
    let socket = TcpStream::connect(&server_addr).await?;
    info!("Connecting to {}...", server_addr);

    let connection = Handshake::client(socket).await?;
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

async fn download_from(mut socket: TcpStream, key: Vec<u8>, filename: &str) -> AppResult<()> {
    send_message(&mut socket, filename.as_bytes()).await?;
    debug!("Share key: {:?}", key);

    let mut cipher_data = recv_message(&mut socket).await?;
    if cipher_data.is_empty() {
        warn!("File not found");
        return Ok(());
    }

    let (iv, ciphertext) = cipher_data.split_at_mut(16);
    let data = decrypt_data(&mut ciphertext.to_vec(), &key, iv)?;

    let file_path = format!("client/data/{}", filename);
    fs::write(file_path, data).await?;
    info!("stored in client/data/{}", filename);

    Ok(())
}
