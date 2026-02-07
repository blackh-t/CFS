use aes::cipher::{
    BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7, generic_array::GenericArray,
};
use rand::{Rng, rngs::OsRng};

use crate::types::AppResult;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

/// Encrypts data using AES-256-CBC with PKCS#7 padding.
///
/// # Parameters:
/// - `buffer`: Plaintext data in bytes.
/// - `key`: AES-256 key (32 bytes).
///
/// # Returns:
/// - `Ok(Vec<u8>)` : `[ iv (16 bytes) || encrypted data ]`
/// - `Err`         : encryption fails.
///
/// # Notes:
/// - Uses PKCS#7 padding.
/// - A random IV is generated for every encryption call.
pub fn encrypt_data(mut buffer: &mut Vec<u8>, key: &[u8]) -> AppResult<Vec<u8>> {
    let buffer_len = buffer.len();
    let block_size = 16;
    buffer.resize(buffer_len + block_size, 0);

    let mut iv = [0u8; 16];
    OsRng.fill(&mut iv);

    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(&iv);
    let cipher = Aes256CbcEnc::new(&key, &iv);

    let encrypt_data = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, buffer_len)
        .unwrap();

    let mut ciphertext = Vec::with_capacity(16 + encrypt_data.len());
    ciphertext.extend_from_slice(&iv);
    ciphertext.extend_from_slice(encrypt_data);

    Ok(ciphertext)
}

/// Decrypts data encrypted with AES-256-CBC and PKCS#7 padding.
///
/// # Parameters:
/// - `buffer`: Encrypted data (ciphertext).
/// - `key`: AES-256 key (32 bytes).
/// - `iv`: Initialization vector (16 bytes).
///
/// # Returns:
/// - `Ok(Vec<u8>)` : decrypted plaintext.
/// - `Err`         : decryption or padding validation fails.
///
/// # Notes:
/// - The IV must match the one used during encryption.
/// - The buffer is modified in memory during decryption.
pub fn decrypt_data(buffer: &mut Vec<u8>, key: &[u8], iv: &[u8]) -> AppResult<Vec<u8>> {
    let key = GenericArray::from_slice(&key);
    let iv = GenericArray::from_slice(iv);

    let cipher = Aes256CbcDec::new(key, iv);

    let decrypted_data = cipher
        .decrypt_padded_mut::<Pkcs7>(buffer)
        .map_err(|e| format!("Decryption error: {:?}", e))?;

    Ok(decrypted_data.to_vec())
}
