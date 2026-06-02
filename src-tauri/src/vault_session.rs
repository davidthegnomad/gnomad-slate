use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::crypto;

/// In-memory secret; heap buffers are zeroed on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretItem {
    pub name: String,
    pub path: String,
    pub value: String,
}

/// JSON wire shape for decrypt only (fields move into `SecretItem`, which zeroizes on drop).
#[derive(Deserialize)]
struct SecretItemWire {
    name: String,
    path: String,
    value: String,
}

impl From<SecretItemWire> for SecretItem {
    fn from(w: SecretItemWire) -> Self {
        Self {
            name: w.name,
            path: w.path,
            value: w.value,
        }
    }
}

/// Metadata returned to the frontend — never includes secret values.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecretMeta {
    pub index: usize,
    pub name: String,
    pub path: String,
}

/// In-memory vault session: encryption key + decrypted secrets (Rust only).
#[derive(ZeroizeOnDrop)]
pub struct VaultSession {
    key: Zeroizing<[u8; 32]>,
    secrets: Vec<SecretItem>,
}

impl VaultSession {
    pub fn from_password_and_secrets(password: &str, secrets: Vec<SecretItem>) -> Result<Self, String> {
        let key = crypto::derive_vault_file_key(password)?;
        Ok(Self {
            key: Zeroizing::new(key),
            secrets,
        })
    }

    pub fn from_password_and_json(password: &str, json: &[u8]) -> Result<Self, String> {
        let secrets: Vec<SecretItem> = if json.is_empty() || json == b"[]" {
            Vec::new()
        } else {
            let wire: Vec<SecretItemWire> =
                serde_json::from_slice(json).map_err(|e| format!("Invalid secrets JSON: {}", e))?;
            wire.into_iter().map(SecretItem::from).collect()
        };
        Self::from_password_and_secrets(password, secrets)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        crypto::encrypt_vault_with_key(plaintext, &self.key)
    }

    pub fn encrypt_secrets(&self) -> Result<Vec<u8>, String> {
        #[derive(Serialize)]
        struct SecretRef<'a> {
            name: &'a str,
            path: &'a str,
            value: &'a str,
        }

        let refs: Vec<SecretRef<'_>> = self
            .secrets
            .iter()
            .map(|s| SecretRef {
                name: &s.name,
                path: &s.path,
                value: &s.value,
            })
            .collect();

        let json =
            serde_json::to_string(&refs).map_err(|e| format!("Failed to serialize secrets: {}", e))?;
        self.encrypt(json.as_bytes())
    }

    pub fn list_metadata(&self) -> Vec<SecretMeta> {
        self.secrets
            .iter()
            .enumerate()
            .map(|(index, s)| SecretMeta {
                index,
                name: s.name.clone(),
                path: s.path.clone(),
            })
            .collect()
    }

    /// Returns a zeroizing copy of the secret value (cleared when the caller drops it).
    pub fn get_value(&self, index: usize) -> Result<Zeroizing<String>, String> {
        self.secrets
            .get(index)
            .map(|s| Zeroizing::new(s.value.clone()))
            .ok_or_else(|| "Secret not found".to_string())
    }

    pub fn add_secret(&mut self, name: String, path: String, value: String) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Secret name cannot be empty".to_string());
        }
        if value.is_empty() {
            return Err("Secret value cannot be empty".to_string());
        }
        self.secrets.push(SecretItem { name, path, value });
        Ok(())
    }

    pub fn delete_secret(&mut self, index: usize) -> Result<(), String> {
        if index >= self.secrets.len() {
            return Err("Secret not found".to_string());
        }
        self.secrets.remove(index);
        Ok(())
    }
}

pub type VaultSessionState = Mutex<Option<VaultSession>>;

pub fn open_session_with_json(
    state: &VaultSessionState,
    password: &str,
    json: &[u8],
) -> Result<(), String> {
    let session = VaultSession::from_password_and_json(password, json)?;
    let mut guard = state
        .lock()
        .map_err(|_| "Vault session lock poisoned".to_string())?;
    *guard = Some(session);
    Ok(())
}

pub fn close_session(state: &VaultSessionState) {
    if let Ok(mut guard) = state.lock() {
        *guard = None;
    }
}

pub fn is_unlocked(state: &VaultSessionState) -> bool {
    state.lock().map(|g| g.is_some()).unwrap_or(false)
}

pub fn with_session<F, T>(state: &VaultSessionState, f: F) -> Result<T, String>
where
    F: FnOnce(&VaultSession) -> Result<T, String>,
{
    let guard = state
        .lock()
        .map_err(|_| "Vault session lock poisoned".to_string())?;
    let session = guard
        .as_ref()
        .ok_or_else(|| "Vault is locked".to_string())?;
    f(session)
}

pub fn with_session_mut<F, T>(state: &VaultSessionState, f: F) -> Result<T, String>
where
    F: FnOnce(&mut VaultSession) -> Result<T, String>,
{
    let mut guard = state
        .lock()
        .map_err(|_| "Vault session lock poisoned".to_string())?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "Vault is locked".to_string())?;
    f(session)
}
