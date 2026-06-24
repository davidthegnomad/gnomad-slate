//! Vault file encryption formats and key derivation.
//!
//! - **v3** (`0x03`): Per-file random 16-byte KDF salt + AES-256-GCM payload (current).
//! - **v2** (`0x02`): Legacy fixed application salt — migrated to v3 on unlock.
//! - **v1**: Legacy per-blob PHC salt — migrated to v3 on unlock.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::{rngs::OsRng, RngCore};

const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 16;
const FORMAT_V2: u8 = 0x02;
const FORMAT_V3: u8 = 0x03;

/// Legacy application-wide salt for v2 vault files (migrated to v3 on next unlock/save).
const LEGACY_V2_KDF_SALT: &[u8] = b"com.gnomadstudio.gnomad-slate.vault.v2";

pub fn generate_vault_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Derives the vault-file AES key from the master password and a salt (v2 legacy or v3 per-file).
pub fn derive_vault_file_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    if salt.is_empty() {
        return Err("KDF salt cannot be empty".to_string());
    }
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

/// Legacy helper — derives using the fixed v2 salt (tests / migration only).
pub fn derive_vault_file_key_legacy(password: &str) -> Result<[u8; 32], String> {
    derive_vault_file_key(password, LEGACY_V2_KDF_SALT)
}

/// Encrypts vault payload (v3): `[0x03 | 16-byte salt | 12-byte nonce | ciphertext+tag]`
pub fn encrypt_vault(plaintext: &[u8], key: &[u8; 32], file_salt: &[u8; SALT_LEN]) -> Result<Vec<u8>, String> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;

    let mut packet = Vec::with_capacity(1 + SALT_LEN + NONCE_LEN + ciphertext.len());
    packet.push(FORMAT_V3);
    packet.extend_from_slice(file_salt);
    packet.extend_from_slice(&nonce_bytes);
    packet.extend_from_slice(&ciphertext);
    Ok(packet)
}

/// Decrypts vault file; supports v3 (per-file salt), v2 (legacy fixed salt), and v1.
pub fn decrypt_vault(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if packet.is_empty() {
        return Err("Empty cipher packet".to_string());
    }

    match packet[0] {
        FORMAT_V3 => decrypt_v3(packet, password),
        FORMAT_V2 => decrypt_v2(packet, password),
        _ => decrypt_v1(packet, password),
    }
}

/// Returns the KDF salt embedded in a v3 vault file, or the legacy v2 salt.
pub fn vault_file_kdf_salt(packet: &[u8]) -> Result<[u8; SALT_LEN], String> {
    if packet.is_empty() {
        return Err("Empty cipher packet".to_string());
    }
    if packet[0] == FORMAT_V3 {
        if packet.len() < 1 + SALT_LEN {
            return Err("Invalid v3 cipher packet length".to_string());
        }
        let mut salt = [0u8; SALT_LEN];
        salt.copy_from_slice(&packet[1..1 + SALT_LEN]);
        return Ok(salt);
    }
    if packet[0] == FORMAT_V2 {
        let mut salt = [0u8; SALT_LEN];
        let len = LEGACY_V2_KDF_SALT.len().min(SALT_LEN);
        salt[..len].copy_from_slice(&LEGACY_V2_KDF_SALT[..len]);
        return Ok(salt);
    }
    Err("Legacy v1 vault has per-blob salt".to_string())
}

fn decrypt_v3(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if packet.len() < 1 + SALT_LEN + NONCE_LEN {
        return Err("Invalid v3 cipher packet length".to_string());
    }

    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&packet[1..1 + SALT_LEN]);
    let key = derive_vault_file_key(password, &salt)?;
    decrypt_v3_with_key(packet, &key)
}

fn decrypt_v2(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if packet.len() < 1 + NONCE_LEN {
        return Err("Invalid v2 cipher packet length".to_string());
    }

    let key = derive_vault_file_key_legacy(password)?;
    decrypt_v2_with_key(packet, &key)
}

fn decrypt_v2_with_key(packet: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if packet.len() < 1 + NONCE_LEN || packet[0] != FORMAT_V2 {
        return Err("Invalid v2 cipher packet".to_string());
    }

    let nonce_bytes = &packet[1..1 + NONCE_LEN];
    let ciphertext = &packet[1 + NONCE_LEN..];

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed (incorrect password or corrupted data)".to_string())
}

fn decrypt_v3_with_key(packet: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if packet.len() < 1 + SALT_LEN + NONCE_LEN || packet[0] != FORMAT_V3 {
        return Err("Invalid v3 cipher packet".to_string());
    }

    let nonce_bytes = &packet[1 + SALT_LEN..1 + SALT_LEN + NONCE_LEN];
    let ciphertext = &packet[1 + SALT_LEN + NONCE_LEN..];

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed (incorrect password or corrupted data)".to_string())
}

/// Encrypt using v3 format with an already-derived session key and file salt.
pub fn encrypt_vault_with_key(
    plaintext: &[u8],
    key: &[u8; 32],
    file_salt: &[u8; SALT_LEN],
) -> Result<Vec<u8>, String> {
    encrypt_vault(plaintext, key, file_salt)
}

pub fn decrypt_vault_with_key(packet: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if packet.is_empty() {
        return Err("Empty cipher packet".to_string());
    }
    match packet[0] {
        FORMAT_V3 => decrypt_v3_with_key(packet, key),
        FORMAT_V2 => decrypt_v2_with_key(packet, key),
        _ => Err("Legacy vault format requires password re-entry".to_string()),
    }
}

/// Returns true if the on-disk blob is legacy v1 (needs migration on next save).
pub fn is_legacy_vault_format(packet: &[u8]) -> bool {
    !packet.is_empty() && packet[0] != FORMAT_V2 && packet[0] != FORMAT_V3
}

/// Returns true if the vault uses the legacy v2 fixed-salt format (migrate to v3 on save).
pub fn is_v2_vault_format(packet: &[u8]) -> bool {
    !packet.is_empty() && packet[0] == FORMAT_V2
}

// --- Legacy v1 format (per-encryption random salt, PHC-style derivation) ---

fn derive_key_v1(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    let argon2 = Argon2::default();

    let salt_str = std::str::from_utf8(salt).map_err(|_| "Invalid salt bytes".to_string())?;
    let salt_parsed = SaltString::new(salt_str).map_err(|e| format!("Salt error: {}", e))?;

    let hash = argon2
        .hash_password(password.as_bytes(), &salt_parsed)
        .map_err(|e| format!("Argon2 hashing failed: {}", e))?;

    if let Some(hash_bytes) = hash.hash {
        let bytes = hash_bytes.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Ok(key)
    } else {
        Err("Could not extract hash bytes".to_string())
    }
}

fn decrypt_v1(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    // v1 packet: [salt_len: u8 | salt (salt_len bytes) | 12-byte nonce | ciphertext]
    // The first byte is the salt length (unusual vs v2/v3 fixed headers).
    let salt_len = packet[0] as usize;
    if packet.len() < 1 + salt_len + NONCE_LEN {
        return Err("Invalid v1 cipher packet length".to_string());
    }

    let salt_bytes = &packet[1..1 + salt_len];
    let nonce_bytes = &packet[1 + salt_len..1 + salt_len + NONCE_LEN];
    let ciphertext = &packet[1 + salt_len + NONCE_LEN..];

    let key = derive_key_v1(password, salt_bytes)?;

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed (incorrect password or corrupted data)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v3_roundtrip() {
        let salt = generate_vault_salt();
        let key = derive_vault_file_key("test-password-12345", &salt).unwrap();
        let plain = b"[]";
        let enc = encrypt_vault(plain, &key, &salt).unwrap();
        assert_eq!(enc[0], FORMAT_V3);
        let dec = decrypt_v3_with_key(&enc, &key).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn v2_legacy_roundtrip() {
        let key = derive_vault_file_key_legacy("test-password-12345").unwrap();
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce_bytes), &b"[]"[..])
            .unwrap();
        let mut packet = vec![FORMAT_V2];
        packet.extend_from_slice(&nonce_bytes);
        packet.extend_from_slice(&ciphertext);
        let dec = decrypt_v2_with_key(&packet, &key).unwrap();
        assert_eq!(dec, b"[]");
    }
}
