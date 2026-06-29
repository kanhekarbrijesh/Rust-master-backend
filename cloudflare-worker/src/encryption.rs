// ─── ENCRYPTION UTILITY — Cloudflare Worker ────────────────────────────────
//
// AES-256-GCM authenticated encryption for files stored via R2.
// Uses `aes-gcm` crate (pure Rust, WASM-compatible).
//
// **Key management:**
//   - Key loaded from env `FILE_ENCRYPTION_KEY` (64 hex chars = 32 bytes)
//   - Each file gets a random 12-byte nonce prepended to ciphertext
//   - Format: [12-byte nonce][ciphertext + 16-byte tag]
//
// **Important:** Decryption MUST happen after authorization.
// ============================================================================

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use serde::Serialize;

/// Size of the AES-256-GCM key in bytes.
pub const KEY_SIZE: usize = 32;

/// Size of the nonce in bytes.
pub const NONCE_SIZE: usize = 12;

/// Overhead: nonce (12) + GCM tag (16) = 28 bytes.
pub const ENCRYPTION_OVERHEAD: usize = 28;

/// Load the encryption key from environment variable.
pub fn load_encryption_key() -> Result<[u8; KEY_SIZE], String> {
    let key_hex = std::env::var("FILE_ENCRYPTION_KEY")
        .map_err(|_| "FILE_ENCRYPTION_KEY environment variable not set".to_string())?;

    let key_bytes = hex::decode(&key_hex)
        .map_err(|e| format!("FILE_ENCRYPTION_KEY is not valid hex: {e}"))?;

    if key_bytes.len() != KEY_SIZE {
        return Err(format!(
            "FILE_ENCRYPTION_KEY must be exactly {} hex chars, got {}",
            KEY_SIZE * 2,
            key_hex.len()
        ));
    }

    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

/// Encrypt plaintext bytes using AES-256-GCM.
/// Returns `nonce || ciphertext || tag`.
pub fn encrypt_file(plaintext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {e}"))?;

    let nonce_bytes: [u8; NONCE_SIZE] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {e}"))?;

    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt ciphertext previously encrypted with `encrypt_file`.
pub fn decrypt_file(ciphertext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < ENCRYPTION_OVERHEAD {
        return Err("Ciphertext too short".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {e}"))?;

    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| "Decryption failed: file corrupt or wrong key".to_string())?;

    Ok(plaintext)
}
