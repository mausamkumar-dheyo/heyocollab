//! AES-256-GCM decryption with PBKDF2 key derivation for storyboard data.
//!
//! Matches TypeScript implementation in storyboardCrypto.ts

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const HARDCODED_SALT: &str = "dheyo-storyboard-salt-v1";
const PBKDF2_ITERATIONS: u32 = 100_000;
const IV_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32; // 256 bits

/// Parameters needed for key derivation
#[derive(Debug, Clone)]
pub struct KeyParams {
    pub email: String,
    pub created_at: i64,
}

/// Crypto errors
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Decryption failed: {0}")]
    Decryption(String),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<aes_gcm::Error> for CryptoError {
    fn from(e: aes_gcm::Error) -> Self {
        CryptoError::Decryption(e.to_string())
    }
}

/// Derive encryption key using PBKDF2
///
/// Key material format: "{email}:{salt}:{createdAt}"
fn derive_key(params: &KeyParams) -> [u8; KEY_LENGTH] {
    let key_material = format!(
        "{}:{}:{}",
        params.email, HARDCODED_SALT, params.created_at
    );
    let salt = HARDCODED_SALT.as_bytes();

    let mut derived_key = [0u8; KEY_LENGTH];
    pbkdf2_hmac::<Sha256>(
        key_material.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut derived_key,
    );

    derived_key
}

/// Decrypt storyboard data from encrypted format
///
/// Input format: base64([12-byte IV][ciphertext with auth tag])
pub fn decrypt_data(encrypted_base64: &str, params: &KeyParams) -> Result<String, CryptoError> {
    // Decode base64
    let combined = BASE64.decode(encrypted_base64)?;

    if combined.len() < IV_LENGTH {
        return Err(CryptoError::InvalidData(format!(
            "Data too short for IV: {} bytes",
            combined.len()
        )));
    }

    // Extract IV and ciphertext
    let iv = &combined[..IV_LENGTH];
    let ciphertext = &combined[IV_LENGTH..];

    // Derive key
    let key = derive_key(params);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Nonce::from_slice(iv);

    // Decrypt
    let plaintext = cipher.decrypt(nonce, ciphertext)?;

    String::from_utf8(plaintext).map_err(CryptoError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_deterministic() {
        let params = KeyParams {
            email: "test@example.com".to_string(),
            created_at: 1700000000000,
        };
        let key1 = derive_key(&params);
        let key2 = derive_key(&params);
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_key_derivation_different_emails() {
        let params1 = KeyParams {
            email: "test1@example.com".to_string(),
            created_at: 1700000000000,
        };
        let params2 = KeyParams {
            email: "test2@example.com".to_string(),
            created_at: 1700000000000,
        };
        let key1 = derive_key(&params1);
        let key2 = derive_key(&params2);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let params = KeyParams {
            email: "test@example.com".to_string(),
            created_at: 1700000000000,
        };
        let result = decrypt_data("not-valid-base64!!!", &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_data_too_short() {
        let params = KeyParams {
            email: "test@example.com".to_string(),
            created_at: 1700000000000,
        };
        // Only 8 bytes (less than IV_LENGTH of 12)
        let short_data = BASE64.encode(&[0u8; 8]);
        let result = decrypt_data(&short_data, &params);
        assert!(matches!(result, Err(CryptoError::InvalidData(_))));
    }
}
