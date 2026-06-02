use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::{rngs::OsRng, RngCore};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

/// Derives a 32-byte key from a password and salt using Argon2id
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let mut key = [0u8; 32];
    let argon2 = Argon2::default();
    
    // Convert salt to standard SaltString format for argon2
    let salt_str = std::str::from_utf8(salt)
        .map_err(|_| "Invalid salt bytes".to_string())?;
    let salt_parsed = SaltString::new(salt_str)
        .map_err(|e| format!("Salt error: {}", e))?;

    let hash = argon2
        .hash_password(password.as_bytes(), &salt_parsed)
        .map_err(|e| format!("Argon2 hashing failed: {}", e))?;

    // Extract the raw hash bytes and copy them to key
    if let Some(hash_bytes) = hash.hash {
        let bytes = hash_bytes.as_bytes();
        let len = bytes.len().min(32);
        key[..len].copy_from_slice(&bytes[..len]);
        Ok(key)
    } else {
        Err("Could not extract hash bytes".to_string())
    }
}

/// Encrypts plaintext using AES-256-GCM with a key derived from the password
pub fn encrypt(plaintext: &[u8], password: &str) -> Result<Vec<u8>, String> {
    // 1. Generate random salt and nonce
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    // Encode salt to base64 to ensure it is valid utf-8 for SaltString
    let salt_base64 = base64_light_encode(&salt);
    let salt_bytes = salt_base64.as_bytes();
    
    // 2. Derive key from password and salt
    let key = derive_key(password, salt_bytes)?;
    
    // 3. Initialize AES-256-GCM cipher
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // 4. Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;
    
    // 5. Package output: [salt_len (1 byte) | salt bytes | nonce bytes | ciphertext]
    let mut packet = Vec::new();
    packet.push(salt_bytes.len() as u8);
    packet.extend_from_slice(salt_bytes);
    packet.extend_from_slice(&nonce_bytes);
    packet.extend_from_slice(&ciphertext);
    
    Ok(packet)
}

/// Decrypts ciphertext using AES-256-GCM with a key derived from the password
pub fn decrypt(packet: &[u8], password: &str) -> Result<Vec<u8>, String> {
    if packet.is_empty() {
        return Err("Empty cipher packet".to_string());
    }
    
    // 1. Extract layout
    let salt_len = packet[0] as usize;
    if packet.len() < 1 + salt_len + NONCE_LEN {
        return Err("Invalid cipher packet length".to_string());
    }
    
    let salt_bytes = &packet[1..1 + salt_len];
    let nonce_bytes = &packet[1 + salt_len..1 + salt_len + NONCE_LEN];
    let ciphertext = &packet[1 + salt_len + NONCE_LEN..];
    
    // 2. Derive key
    let key = derive_key(password, salt_bytes)?;
    
    // 3. Decrypt
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    
    let nonce = Nonce::from_slice(nonce_bytes);
    let decrypted = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed (incorrect password or corrupted data)".to_string())?;
        
    Ok(decrypted)
}

/// Extremely simple Base64 encoder for generating valid salt strings
fn base64_light_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in bytes.chunks(3) {
        let mut buffer = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            buffer |= (b as u32) << (16 - i * 8);
        }
        let num_encoded = match chunk.len() {
            1 => 2,
            2 => 3,
            _ => 4,
        };
        for i in 0..num_encoded {
            let val = (buffer >> (18 - i * 6)) & 0x3F;
            result.push(ALPHABET[val as usize] as char);
        }
        if chunk.len() == 1 {
            result.push_str("==");
        } else if chunk.len() == 2 {
            result.push('=');
        }
    }
    
    // Sanitize to be a valid argon2 salt (letters, digits, '.' and '/')
    result.replace("+", ".").replace("=", "")
}
