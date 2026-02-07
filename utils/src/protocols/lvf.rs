use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::types::AppResult;

/// Reads bytes from the commected TCP stream.
///
/// # Arguments
/// * `socket` - A mutable reference to the connected `TcpStream`.
///
/// # Returns
/// * `Ok(Vec<u8>)` - The payload in raw bytes.
/// * `Err(std::io::Error)` - If the connection is closed or an I/O error occurs.
pub async fn recv_message(socket: &mut TcpStream) -> AppResult<Vec<u8>> {
    let mut len_bytes = [0u8; 4]; // Packet Header: data len.
    socket.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    if len > 100 * 1024 * 1024 {
        // 100 MB limit
        return Err("Message too large".into());
    }

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
    let len = data.len() as u32; // define data len.
    let len_bytes = len.to_be_bytes();

    // Write length + data
    socket.write_all(&len_bytes).await?;
    socket.write_all(data).await?;
    Ok(())
}
