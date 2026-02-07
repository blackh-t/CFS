use std::error::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

/// Reads bytes from the commected TCP stream.
///
/// # Arguments
/// * `socket` - A mutable reference to the connected `TcpStream`.
///
/// # Returns
/// * `Ok(Vec<u8>)` - The payload in raw bytes.
/// * `Err(std::io::Error)` - If the connection is closed or an I/O error occurs.
pub async fn recv_message(socket: &mut TcpStream) -> AppResult<Vec<u8>> {
    // Extract data len from the header.
    let mut len_bytes = [0u8; 4];
    socket.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut buffer = vec![0u8; len];
    socket.read_exact(&mut buffer).await?;

    Ok(buffer)
}

/// Writes bytes to the commected TCP stream.
///
/// # Arguments
/// * `socket` - A mutable reference to the connected `TcpStream`.
/// * `data` - Must be raw bytes.
///
/// # Returns
/// * `Ok(())` - Success.
/// * `Err(std::io::Error)` - If the connection is closed or an I/O error occurs.
pub async fn send_message(socket: &mut TcpStream, data: &[u8]) -> AppResult<()> {
    // Define data len.
    let len = data.len() as u32;
    let len_bytes = len.to_be_bytes();

    // Write length + data
    socket.write_all(&len_bytes).await?;
    socket.write_all(data).await?;
    Ok(())
}

pub struct Connection {
    pub socket: TcpStream,
    pub key: Vec<u8>,
}

pub type AppResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
