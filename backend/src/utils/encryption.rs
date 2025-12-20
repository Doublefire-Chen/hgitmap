use aes_gcm::{
    aead::{Aead, KeyInit, rand_core::RngCore},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};

/// Encrypt a plaintext string using AES-256-GCM
/// Returns base64-encoded ciphertext with nonce prepended (nonce is first 12 bytes)
pub fn encrypt(plaintext: &str, key_base64: &str) -> Result<String> {
    // Decode the base64-encoded key
    let key_bytes = general_purpose::STANDARD
        .decode(key_base64)
        .map_err(|e| anyhow!("Invalid base64 encryption key: {}", e))?;

    if key_bytes.len() != 32 {
        return Err(anyhow!(
            "Encryption key must be 32 bytes (256 bits), got {} bytes",
            key_bytes.len()
        ));
    }

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    // Generate a random nonce (96 bits / 12 bytes for GCM)
    let mut nonce_bytes = [0u8; 12];
    aes_gcm::aead::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    // Prepend nonce to ciphertext and encode as base64
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);

    Ok(general_purpose::STANDARD.encode(&combined))
}

/// Decrypt a base64-encoded ciphertext using AES-256-GCM
/// Expects nonce to be prepended to ciphertext (first 12 bytes)
pub fn decrypt(ciphertext_base64: &str, key_base64: &str) -> Result<String> {
    // Decode the base64-encoded key
    let key_bytes = general_purpose::STANDARD
        .decode(key_base64)
        .map_err(|e| anyhow!("Invalid base64 encryption key: {}", e))?;

    if key_bytes.len() != 32 {
        return Err(anyhow!(
            "Encryption key must be 32 bytes (256 bits), got {} bytes",
            key_bytes.len()
        ));
    }

    // Decode the base64-encoded ciphertext
    let combined = general_purpose::STANDARD
        .decode(ciphertext_base64)
        .map_err(|e| anyhow!("Invalid base64 ciphertext: {}", e))?;

    if combined.len() < 12 {
        return Err(anyhow!("Ciphertext too short to contain nonce"));
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    // Decrypt the ciphertext
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8 in decrypted text: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        // Generate a random 32-byte key and encode it as base64
        let key = general_purpose::STANDARD.encode(&[0u8; 32]);
        let plaintext = "my-secret-access-token";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key1 = general_purpose::STANDARD.encode(&[0u8; 32]);
        let key2 = general_purpose::STANDARD.encode(&[1u8; 32]);
        let plaintext = "my-secret-access-token";

        let encrypted = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = general_purpose::STANDARD.encode(&[0u8; 16]); // Only 16 bytes
        let plaintext = "my-secret-access-token";

        let result = encrypt(plaintext, &short_key);
        assert!(result.is_err());
    }
}
