use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardEntry {
    pub text: String,
    pub timestamp: u64,
}

pub struct ClipboardHistory {
    entries: Vec<ClipboardEntry>,
    max_size: usize,
}

impl ClipboardHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
        }
    }

    pub fn add(&mut self, text: String) -> bool {
        // Check if text is sensitive (API keys, passwords, private keys)
        if is_sensitive(&text) {
            return false;
        }

        // Avoid duplicate consecutive entries
        if let Some(last) = self.entries.first() {
            if last.text == text {
                return false;
            }
        }

        // Remove any prior identical entries to move this one to the top
        self.entries.retain(|e| e.text != text);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.insert(0, ClipboardEntry { text, timestamp });

        // Enforce max size limit
        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }

        true
    }

    pub fn get_entries(&self) -> Vec<ClipboardEntry> {
        self.entries.clone()
    }
}

/// Simple scanner to filter out sensitive credentials from clipboard history
fn is_sensitive(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Reject extremely large text blocks (> 20KB) in clipboard history to avoid memory bloat
    if text.len() > 20_000 {
        return true;
    }

    // Standard credential headers or key prefixes
    let patterns = [
        "-----BEGIN",
        "PRIVATE KEY",
        "ghp_", // GitHub Personal Access Token
        "sk-proj-", // OpenAI API key
        "xoxb-", // Slack Token
        "AWS_SECRET_ACCESS_KEY",
        "DATABASE_URL",
        "mysql://",
        "postgresql://",
        "amqp://",
        "mongodb://",
        "secret_key",
        "client_secret",
    ];

    for pattern in patterns {
        if trimmed.contains(pattern) {
            return true;
        }
    }

    // Check if it's a raw high-entropy string without whitespace (common API key pattern)
    // E.g., length between 24 and 128 characters, with no spaces
    if trimmed.len() >= 24 && trimmed.len() <= 128 && !trimmed.contains(' ') && !trimmed.contains('\n') {
        // Simple heuristic: if it has high variation of upper/lower/numbers
        let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
        let has_lowercase = trimmed.chars().any(|c| c.is_ascii_lowercase());
        let has_uppercase = trimmed.chars().any(|c| c.is_ascii_uppercase());
        if has_digit && has_lowercase && has_uppercase {
            return true;
        }
    }

    false
}

/// Starts a background thread to poll the system clipboard and update the shared history
pub fn start_clipboard_monitor(history: Arc<Mutex<ClipboardHistory>>) {
    thread::spawn(move || {
        // Initialize clipboard. Fallback gracefully if clipboard access is denied
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Clipboard Monitor Error: Failed to initialize arboard clipboard: {}", e);
                return;
            }
        };

        let mut last_copied = String::new();

        loop {
            // Poll every 1 second
            thread::sleep(Duration::from_millis(1000));

            if let Ok(text) = clipboard.get_text() {
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() && trimmed != last_copied {
                    last_copied = trimmed.clone();
                    
                    // Scope block to unlock mutex and push new entry
                    {
                        if let Ok(mut hist) = history.lock() {
                            hist.add(trimmed);
                        }
                    }
                }
            }
        }
    });
}
