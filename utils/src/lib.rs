pub mod cipher;
pub mod logger;
pub mod protocols;
pub mod types;

#[cfg(test)]
mod tests {

    use tokio::net::{TcpListener, TcpStream};

    use crate::{
        cipher::{decrypt_data, encrypt_data},
        protocols::{
            handshake::Handshake,
            lvf::{recv_message, send_message},
        },
    };

    #[test]
    fn test_cipher() {
        let key = [0u8; 32];
        let original_data = b"secret message".to_vec();
        let mut buffer = original_data.clone();

        let cipher_data = encrypt_data(&mut buffer, &key).expect("Encryption failed");

        // Extract IV and Ciphertext
        let (iv, ciphertext) = cipher_data.split_at(16);
        let mut ciphertext_vec = ciphertext.to_vec();

        let decrypted_data =
            decrypt_data(&mut ciphertext_vec, &key, iv).expect("Decryption failed");

        assert_eq!(original_data, decrypted_data);
    }

    #[tokio::test]
    async fn test_handshake() {
        let _ = env_logger::builder().is_test(true).try_init();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // spawn server thrd
        let server_handler = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            Handshake::server(socket).await
        });

        // run client on current process.
        let client_socket = TcpStream::connect(addr).await.unwrap();
        let client_result = Handshake::client(client_socket).await;
        let server_result = server_handler.await.unwrap();

        assert!(
            client_result.is_ok(),
            "Client handshake failed: {:?}",
            client_result.err()
        );
        assert!(
            server_result.is_ok(),
            "Server handshake failed: {:?}",
            server_result.err()
        );

        let client_conn = client_result.unwrap();
        let server_conn = server_result.unwrap();

        // critical check
        assert_eq!(client_conn.key, server_conn.key, "Mismatch Share key");
        assert_eq!(
            client_conn.key.len(),
            32,
            "Expected AES-256 key to be [u8; 32]"
        );
    }

    #[tokio::test]
    async fn test_length_value_framing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let payload = b"Lenght Value Framing".to_vec();

        let server_handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            recv_message(&mut socket).await
        });

        let mut client_socket = TcpStream::connect(addr).await.unwrap();
        send_message(&mut client_socket, &payload)
            .await
            .expect("Failed to send");

        let received_payload = server_handle.await.unwrap().expect("Failed to receive");
        assert_eq!(payload, received_payload);
    }
}
