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

/// `vault.enc` is absent or zero bytes. A real empty secret list is non-empty ciphertext.
pub const VAULT_FILE_MISSING: &str =
    "Vault file is missing or empty. Unlock and recovery will not create a replacement.";

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

const RECOVERY_GROUP_LEN: usize = 4;
const RECOVERY_GROUPS: usize = 8;
const RECOVERY_RAW_LEN: usize = 20;
const WRAP_SALT_LEN: usize = 16;
const RECOVERY_ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// Wrapped vault file key. The recovery key itself is never stored.
#[derive(Clone)]
pub struct RecoveryWrapRecord {
    pub salt: [u8; WRAP_SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
    pub wrapped_key: Vec<u8>,
}

pub struct RotatedVault {
    pub plaintext: Vec<u8>,
    pub packet: Vec<u8>,
    pub wrap: RecoveryWrapRecord,
}

/// 8 groups of 4 lowercase base32 characters, hyphen-separated (160 bits).
pub fn generate_recovery_key() -> String {
    let mut raw = [0u8; RECOVERY_RAW_LEN];
    OsRng.fill_bytes(&mut raw);
    group_recovery_key(&encode_base32(&raw))
}

pub fn normalize_recovery_key(input: &str) -> Result<String, String> {
    let mut compact = String::with_capacity(RECOVERY_GROUP_LEN * RECOVERY_GROUPS);
    for c in input.chars() {
        if c.is_whitespace() || c == '-' {
            continue;
        }
        for lower in c.to_lowercase() {
            compact.push(lower);
        }
    }
    if compact.len() != RECOVERY_GROUP_LEN * RECOVERY_GROUPS
        || !compact.bytes().all(|b| RECOVERY_ALPHABET.contains(&b))
    {
        return Err("Invalid recovery key".to_string());
    }
    Ok(group_recovery_key(&compact))
}

/// Argon2id wrap key from the recovery key and a random salt (same Argon2 family as the password verifier).
pub fn derive_wrap_key(recovery_key: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    if salt.len() < 8 {
        return Err("Invalid recovery data".to_string());
    }
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(recovery_key.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

pub fn create_recovery_wrap(file_key: &[u8; 32], recovery_key: &str) -> Result<RecoveryWrapRecord, String> {
    let mut salt = [0u8; WRAP_SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    rewrap_file_key(file_key, recovery_key, &salt)
}

pub fn rewrap_file_key(
    file_key: &[u8; 32],
    recovery_key: &str,
    salt: &[u8; WRAP_SALT_LEN],
) -> Result<RecoveryWrapRecord, String> {
    let wrap_key = derive_wrap_key(recovery_key, salt)?;
    let (nonce, wrapped_key) = seal_file_key(&wrap_key, file_key)?;
    Ok(RecoveryWrapRecord {
        salt: *salt,
        nonce,
        wrapped_key,
    })
}

pub fn unwrap_with_recovery(
    record: &RecoveryWrapRecord,
    recovery_key: &str,
) -> Result<[u8; 32], String> {
    let wrap_key = derive_wrap_key(recovery_key, &record.salt)?;
    open_file_key(&wrap_key, &record.nonce, &record.wrapped_key)
}

/// Decrypt with the wrapped file key, then re-encrypt and re-wrap under a new password.
/// The recovery key and wrap salt stay the same. A wrong recovery key returns an error
/// and does not produce a replacement packet. An empty packet is rejected.
pub fn rotate_file_key_with_recovery(
    vault_packet: &[u8],
    record: &RecoveryWrapRecord,
    recovery_key: &str,
    new_password: &str,
) -> Result<RotatedVault, String> {
    if vault_packet.is_empty() {
        return Err(VAULT_FILE_MISSING.to_string());
    }
    let old_key = unwrap_with_recovery(record, recovery_key)?;
    let plaintext = decrypt_vault_with_key(vault_packet, &old_key)?;
    let new_key = derive_vault_file_key(new_password)?;
    let packet = encrypt_vault(&plaintext, &new_key)?;
    let wrap = rewrap_file_key(&new_key, recovery_key, &record.salt)?;
    Ok(RotatedVault {
        plaintext,
        packet,
        wrap,
    })
}

pub fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn hex_val(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err("Invalid recovery data".to_string()),
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("Invalid recovery data".to_string());
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_val(bytes[i])?;
        let lo = hex_val(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

pub fn decode_hex_array<const N: usize>(s: &str) -> Result<[u8; N], String> {
    let bytes = decode_hex(s)?;
    if bytes.len() != N {
        return Err("Invalid recovery data".to_string());
    }
    let mut arr = [0u8; N];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn seal_file_key(wrap_key: &[u8; 32], file_key: &[u8; 32]) -> Result<([u8; NONCE_LEN], Vec<u8>), String> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(wrap_key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let wrapped = cipher
        .encrypt(nonce, file_key.as_slice())
        .map_err(|e| format!("Encryption error: {}", e))?;
    Ok((nonce_bytes, wrapped))
}

fn open_file_key(wrap_key: &[u8; 32], nonce_bytes: &[u8], wrapped: &[u8]) -> Result<[u8; 32], String> {
    if nonce_bytes.len() != NONCE_LEN {
        return Err("Invalid recovery data".to_string());
    }
    let cipher = Aes256Gcm::new_from_slice(wrap_key)
        .map_err(|e| format!("Cipher init error: {}", e))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, wrapped)
        .map_err(|_| "Invalid recovery key".to_string())?;
    if plain.len() != 32 {
        return Err("Invalid recovery key".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&plain);
    Ok(key)
}

fn encode_base32(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() * 8 + 4) / 5);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for &byte in data {
        acc = (acc << 8) | byte as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((acc >> bits) & 31) as usize;
            out.push(RECOVERY_ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((acc << (5 - bits)) & 31) as usize;
        out.push(RECOVERY_ALPHABET[idx] as char);
    }
    out
}

fn group_recovery_key(compact: &str) -> String {
    compact
        .as_bytes()
        .chunks(RECOVERY_GROUP_LEN)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_recovery_key_format(key: &str) {
        let parts: Vec<&str> = key.split('-').collect();
        assert_eq!(parts.len(), RECOVERY_GROUPS);
        for part in parts {
            assert_eq!(part.len(), RECOVERY_GROUP_LEN);
            assert!(part.bytes().all(|b| RECOVERY_ALPHABET.contains(&b)));
        }
    }

    fn expect_ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected success: {err}"),
        }
    }

    #[test]
    fn v2_roundtrip() {
        let key = derive_vault_file_key("test-password-12345").unwrap();
        let plain = b"[]";
        let enc = encrypt_vault(plain, &key).unwrap();
        let dec = decrypt_v2_with_key(&enc, &key).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn recovery_key_format_and_normalize() {
        let key = generate_recovery_key();
        assert_recovery_key_format(&key);
        assert_eq!(normalize_recovery_key(&key.to_uppercase()).unwrap(), key);
        assert_eq!(normalize_recovery_key(&key.replace('-', " ")).unwrap(), key);
        assert_eq!(normalize_recovery_key(&key.replace('-', "")).unwrap(), key);
        assert!(normalize_recovery_key("not-a-key").is_err());
    }

    #[test]
    fn wrap_unwrap_roundtrip() {
        let file_key = derive_vault_file_key("test-password-12345").unwrap();
        let recovery = generate_recovery_key();
        let wrap = expect_ok(create_recovery_wrap(&file_key, &recovery));
        let opened = unwrap_with_recovery(&wrap, &recovery).unwrap();
        assert_eq!(opened, file_key);

        let wrong = generate_recovery_key();
        assert_eq!(
            unwrap_with_recovery(&wrap, &wrong).unwrap_err(),
            "Invalid recovery key"
        );
    }

    #[test]
    fn recover_roundtrip_keeps_plaintext() {
        let password = "test-password-12345";
        let new_password = "replacement-password";
        let recovery = generate_recovery_key();
        let file_key = derive_vault_file_key(password).unwrap();
        let plain = br#"[{"name":"token","path":"proj","value":"plain-value"}]"#;
        let packet = encrypt_vault(plain, &file_key).unwrap();
        let wrap = expect_ok(create_recovery_wrap(&file_key, &recovery));

        let wrong = generate_recovery_key();
        let err = match rotate_file_key_with_recovery(&packet, &wrap, &wrong, new_password) {
            Err(message) => message,
            Ok(_) => panic!("wrong recovery key must not rotate the vault"),
        };
        assert_eq!(err, "Invalid recovery key");
        assert_eq!(decrypt_vault_with_key(&packet, &file_key).unwrap(), plain);

        let rotated = expect_ok(rotate_file_key_with_recovery(
            &packet,
            &wrap,
            &recovery,
            new_password,
        ));
        assert_eq!(rotated.plaintext, plain);
        assert_eq!(rotated.wrap.salt, wrap.salt);

        let new_key = derive_vault_file_key(new_password).unwrap();
        assert_eq!(decrypt_vault_with_key(&rotated.packet, &new_key).unwrap(), plain);
        assert!(decrypt_vault(&rotated.packet, password).is_err());
        assert_eq!(unwrap_with_recovery(&rotated.wrap, &recovery).unwrap(), new_key);
    }

    #[test]
    fn rotate_rejects_empty_packet() {
        let record = RecoveryWrapRecord {
            salt: [1u8; 16],
            nonce: [2u8; 12],
            wrapped_key: vec![3, 4],
        };
        let err = match rotate_file_key_with_recovery(&[], &record, "unused", "unused") {
            Err(message) => message,
            Ok(_) => panic!("empty ciphertext must not rotate into an empty vault"),
        };
        assert_eq!(err, VAULT_FILE_MISSING);
    }
}
