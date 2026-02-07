use aes::Aes256;
use std::error::Error;

pub type AppResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub type Aes256CbcEnc = cbc::Encryptor<Aes256>;
pub type Aes256CbcDec = cbc::Decryptor<Aes256>;
