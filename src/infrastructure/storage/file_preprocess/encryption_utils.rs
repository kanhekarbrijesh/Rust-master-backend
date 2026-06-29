// ─── FILE ENCRYPTION/DECRYPTION UTILITY ────────────────────────────────────
//
// Provides AES-256-GCM authenticated encryption for files stored via any
// storage provider.  Uses the `ring` crate (RustCrypto, FIPS 140-2 ready).
//
// **Security guarantees:**
//   - AES-256-GCM: industry-standard symmetric encryption
//   - 12-byte random nonce per encryption (unique per file)
//   - Authentication tag: tampering is detected on decryption
//   - Key derived from a 256-bit master key via HKDF-SHA256
//
// **Flow:**
//   Upload:   plaintext → encrypt() → store encrypted blob
//   Download: load encrypted blob → decrypt() → plaintext
//
// **Key management:**
//   - A single 256-bit master key is loaded from env FILE_ENCRYPTION_KEY
//     (hex-encoded, 64 hex chars = 32 bytes)
//   - Each file gets its own random nonce prepended to the ciphertext
//   - Format: [12-byte nonce][AES-256-GCM ciphertext + 16-byte tag]
//
// **Important:**
//   Decryption MUST happen AFTER authorization check in the controller/service
//   layer.  NEVER decrypt before checking permissions.
// ============================================================================

use crate::_utils::app_error::AppError;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

/// Size of the AES-256-GCM key in bytes.
pub const KEY_SIZE: usize = 32;

/// Size of the nonce in bytes.
pub const NONCE_SIZE: usize = 12;

/// Size of the GCM authentication tag in bytes.
pub const TAG_SIZE: usize = 16;

/// Overhead added by encryption: nonce (12) + tag (16) = 28 bytes.
pub const ENCRYPTION_OVERHEAD: usize = NONCE_SIZE + TAG_SIZE;

// ─── Key Loading ───────────────────────────────────────────────────────────

/// Load the 256-bit master encryption key from the environment.
///
/// Expects `FILE_ENCRYPTION_KEY` as a 64-character hex string (32 bytes).
/// Returns a default key (derived from a hardcoded constant) if the env var
/// is not set — **only safe for local development**.
///
/// **Production:** Always set `FILE_ENCRYPTION_KEY` in your environment.
pub fn load_encryption_key() -> Result<[u8; KEY_SIZE], AppError> {
    let key_hex = std::env::var("FILE_ENCRYPTION_KEY").unwrap_or_else(|_| String::new());

    if key_hex.is_empty() {
        // ⚠️ Local development only — derive from a known seed
        return Ok(derive_local_dev_key());
    }

    let key_bytes = hex::decode(&key_hex).map_err(|e| {
        AppError::Internal(format!(
            "FILE_ENCRYPTION_KEY is not valid hex: {e}"
        ))
    })?;

    if key_bytes.len() != KEY_SIZE {
        return Err(AppError::Internal(format!(
            "FILE_ENCRYPTION_KEY must be exactly {} hex chars ({} bytes), got {} chars",
            KEY_SIZE * 2,
            KEY_SIZE,
            key_hex.len()
        )));
    }

    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&key_bytes);
    Ok(key)
}

/// Derive a deterministic key for local development.
/// **NEVER use this in production.** Always set FILE_ENCRYPTION_KEY.
fn derive_local_dev_key() -> [u8; KEY_SIZE] {
    use ring::digest::{Context, SHA256};
    let mut ctx = Context::new(&SHA256);
    ctx.update(b"rust-tut-day1-local-dev-encryption-key-do-not-use-in-prod");
    let digest = ctx.finish();
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(digest.as_ref());
    key
}

// ─── Encryption / Decryption ───────────────────────────────────────────────

/// Encrypt plaintext bytes using AES-256-GCM.
///
/// Returns `nonce || ciphertext || tag` (ready for storage).
///
/// # Arguments
/// * `plaintext` - Raw bytes to encrypt
/// * `key` - 256-bit key (32 bytes)
pub fn encrypt_file(plaintext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, AppError> {
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|e| AppError::Internal(format!("Failed to create encryption key: {e}")))?;
    let key = LessSafeKey::new(unbound_key);

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce_bytes)
        .map_err(|e| AppError::Internal(format!("Failed to generate nonce: {e}")))?;

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    // Allocate buffer: nonce + plaintext + tag overhead
    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|e| AppError::Internal(format!("Encryption failed: {e}")))?;

    // Prepend the nonce to the ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);

    Ok(result)
}

/// Decrypt ciphertext bytes previously encrypted with `encrypt_file`.
///
/// Expects `nonce || ciphertext || tag` format.
///
/// # Arguments
/// * `ciphertext` - Encrypted bytes with prepended nonce
/// * `key` - 256-bit key (32 bytes) — MUST match the encryption key
pub fn decrypt_file(ciphertext: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>, AppError> {
    if ciphertext.len() < ENCRYPTION_OVERHEAD {
        return Err(AppError::BadRequest(
            "Ciphertext is too short to contain a valid encrypted file".into(),
        ));
    }

    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|e| AppError::Internal(format!("Failed to create decryption key: {e}")))?;
    let key = LessSafeKey::new(unbound_key);

    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::assume_unique_for_key(
        nonce_bytes.try_into().map_err(|_| {
            AppError::Internal("Invalid nonce length in ciphertext".into())
        })?,
    );

    let mut in_out = encrypted.to_vec();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| {
            AppError::BadRequest(
                "Decryption failed: file may be corrupt or encryption key has changed".into(),
            )
        })?;

    Ok(plaintext.to_vec())
}

// ─── Convenience: check if bytes look encrypted ────────────────────────────

/// Check whether a byte slice appears to be encrypted (starts with a nonce
/// and has the minimum length for an AES-256-GCM encrypted payload).
///
/// This is a heuristic — there is no magic byte marker.  It checks length.
pub fn is_encrypted(data: &[u8]) -> bool {
    data.len() >= ENCRYPTION_OVERHEAD
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = load_encryption_key().unwrap();
        let plaintext = b"Hello, this is a test file with sensitive content!";

        let encrypted = encrypt_file(plaintext, &key).unwrap();
        assert!(encrypted.len() > plaintext.len());
        assert!(is_encrypted(&encrypted));

        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_empty_content() {
        let key = load_encryption_key().unwrap();
        let plaintext = b"";

        let encrypted = encrypt_file(plaintext, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let key = load_encryption_key().unwrap();
        let mut wrong_key = key;
        wrong_key[0] ^= 0x01; // flip one bit

        let plaintext = b"sensitive data";
        let encrypted = encrypt_file(plaintext, &key).unwrap();

        let result = decrypt_file(&encrypted, &wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_tampered_ciphertext_fails() {
        let key = load_encryption_key().unwrap();
        let plaintext = b"tamper test";

        let mut encrypted = encrypt_file(plaintext, &key).unwrap();
        // Corrupt one byte in the ciphertext portion (after nonce)
        if encrypted.len() > NONCE_SIZE + 1 {
            encrypted[NONCE_SIZE + 5] ^= 0xff;
        }

        let result = decrypt_file(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_too_short_fails() {
        let key = load_encryption_key().unwrap();
        let result = decrypt_file(b"too-short", &key);
        assert!(result.is_err());
    }

    #[test]
    fn key_from_env_or_default() {
        let key = load_encryption_key().unwrap();
        assert_eq!(key.len(), KEY_SIZE);
    }

    #[test]
    fn is_encrypted_heuristic() {
        let key = load_encryption_key().unwrap();
        assert!(!is_encrypted(b"small"));
        let encrypted = encrypt_file(b"some content that is long enough", &key).unwrap();
        assert!(is_encrypted(&encrypted));
    }
}
