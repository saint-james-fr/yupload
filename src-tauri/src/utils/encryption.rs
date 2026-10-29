use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};

/// Encryption key length (32 bytes for AES-256)
const KEY_LENGTH: usize = 32;

/// Generate an encryption key from app-specific data
///
/// This derives a key from the app name and a fixed salt.
/// For a desktop app, this provides basic protection while keeping
/// the implementation simple. The key is deterministic per app instance.
fn get_encryption_key() -> [u8; KEY_LENGTH] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Use app name and salt to derive key
    let app_name = "yupload";
    let salt = "yupload-encryption-salt-v1";
    let combined = format!("{}{}", app_name, salt);

    // Hash to get deterministic 32-byte key
    let mut hasher = DefaultHasher::new();
    combined.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert to 32-byte array (repeat hash bytes)
    let mut key = [0u8; KEY_LENGTH];
    let hash_bytes = hash.to_be_bytes();
    for i in 0..KEY_LENGTH {
        key[i] = hash_bytes[i % 8] ^ (i as u8);
    }

    key
}

/// Encrypt a token string before storing
///
/// Uses AES-256-GCM encryption with a random nonce.
/// Returns base64-encoded string containing nonce + ciphertext.
pub fn encrypt_token(token: &str) -> Result<String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new_from_slice(&key).context("Failed to create cipher")?;

    // Generate random nonce
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    // Encrypt
    let ciphertext = cipher
        .encrypt(&nonce, token.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Combine nonce + ciphertext and encode as base64
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    let encoded = general_purpose::STANDARD.encode(&combined);

    Ok(encoded)
}

/// Decrypt a token string after retrieving from storage
///
/// Expects base64-encoded string containing nonce + ciphertext.
/// Returns the decrypted token string.
pub fn decrypt_token(encrypted: &str) -> Result<String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new_from_slice(&key).context("Failed to create cipher")?;

    // Decode from base64
    let combined = general_purpose::STANDARD
        .decode(encrypted)
        .context("Failed to decode base64")?;

    // Extract nonce (first 12 bytes for GCM)
    if combined.len() < 12 {
        anyhow::bail!("Encrypted data too short");
    }

    let nonce = Nonce::from_slice(&combined[..12]);
    let ciphertext = &combined[12..];

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).context("Decrypted data is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let original = "test_token_12345";
        let encrypted = encrypt_token(original).unwrap();
        let decrypted = decrypt_token(&encrypted).unwrap();
        assert_eq!(original, decrypted);
    }
}
