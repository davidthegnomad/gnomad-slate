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
const FORMAT_V2: u8 = 0x02;

/// Application-wide salt for deriving the vault file encryption key (v2).
const VAULT_FILE_KDF_SALT: &[u8] = b"com.gnomadstudio.gnomad-slate.vault.v2";

/// Derives the stable vault-file AES key from the master password (Argon2id raw output).
pub fn derive_vault_file_key(password: &str) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), VAULT_FILE_KDF_SALT, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

/// Encrypts vault payload (v2): `[0x02 | 12-byte nonce | ciphertext+tag]`
pub fn encrypt_vault(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;

    let mut packet = Vec::with_capacity(1 + NONCE_LEN + ciphertext.len());
    packet.push(FORMAT_V2);
    packet.extend_from_slice(&nonce_bytes);
    packet.extend_from_slice(&ciphertext);
    Ok(packet)
}

/// Decrypts vault file; supports v2 (session key) and legacy v1 (password).
pub fn decrypt_vault(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if packet.is_empty() {
        return Err("Empty cipher packet".to_string());
    }

    if packet[0] == FORMAT_V2 {
        return decrypt_v2(packet, password);
    }

    decrypt_v1(packet, password)
}

fn decrypt_v2(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if packet.len() < 1 + NONCE_LEN {
        return Err("Invalid v2 cipher packet length".to_string());
    }

    let key = derive_vault_file_key(password)?;
    let nonce_bytes = &packet[1..1 + NONCE_LEN];
    let ciphertext = &packet[1 + NONCE_LEN..];

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed (incorrect password or corrupted data)".to_string())
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
        .map_err(|_| "Decryption failed (corrupted data)".to_string())
}

/// Encrypt using v2 format with an already-derived session key.
pub fn encrypt_vault_with_key(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    encrypt_vault(plaintext, key)
}

pub fn decrypt_vault_with_key(packet: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if packet.is_empty() {
        return Err("Empty cipher packet".to_string());
    }
    if packet[0] == FORMAT_V2 {
        return decrypt_v2_with_key(packet, key);
    }
    Err("Legacy vault format requires password re-entry".to_string())
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

/// Returns true if the on-disk blob is legacy v1 (needs migration on next save).
pub fn is_legacy_vault_format(packet: &[u8]) -> bool {
    !packet.is_empty() && packet[0] != FORMAT_V2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_roundtrip() {
        let key = derive_vault_file_key("test-password-12345").unwrap();
        let plain = b"[]";
        let enc = encrypt_vault(plain, &key).unwrap();
        let dec = decrypt_v2_with_key(&enc, &key).unwrap();
        assert_eq!(dec, plain);
    }
}
