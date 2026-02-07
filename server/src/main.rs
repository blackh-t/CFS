use log::{debug, error, info, warn};
use std::{env, io};
use tokio::{
    fs,
    net::{TcpListener, TcpStream},
};
use utils::{
    cipher::encrypt_data,
    logger::init_logger,
    protocols::{
        handshake::Handshake,
        lvf::{recv_message, send_message},
    },
    types::AppResult,
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
    let mut conn = Handshake::server(socket).await?;
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

        let packet = encrypt_data(&mut file_content, &conn.key)?;
        send_message(&mut conn.socket, &packet).await?;
        info!("File Transfered.");
    }

    Ok(())
}
