use arboard::Clipboard;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};

use crate::clipboard_filter;

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

    pub fn add(&mut self, text: String, filter_enabled: bool) -> bool {
        if filter_enabled && clipboard_filter::is_sensitive(&text) {
            return false;
        }

        if let Some(last) = self.entries.first() {
            if last.text == text {
                return false;
            }
        }

        self.entries.retain(|e| e.text != text);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.entries.insert(0, ClipboardEntry { text, timestamp });

        if self.entries.len() > self.max_size {
            self.entries.truncate(self.max_size);
        }

        true
    }

    pub fn get_entries(&self) -> Vec<ClipboardEntry> {
        self.entries.clone()
    }
}

/// Read-only snapshot. Does not record history and does not replace the pasteboard.
pub fn read_clipboard_text() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .get_text()
        .map_err(|e| format!("Failed to read clipboard: {}", e))
}

const POLL_ACTIVE_MS: u64 = 1000;
const POLL_IDLE_MS: u64 = 3000;
const IDLE_POLLS_BEFORE_SLOWDOWN: u32 = 5;
const SECRET_CLIPBOARD_CLEAR_SECS: u64 = 30;

static HISTORY_SUPPRESS: Mutex<Option<String>> = Mutex::new(None);

fn suppress_history(text: &str) {
    if let Ok(mut slot) = HISTORY_SUPPRESS.lock() {
        *slot = Some(text.trim().to_string());
    }
}

fn history_suppressed(text: &str) -> bool {
    HISTORY_SUPPRESS
        .lock()
        .ok()
        .and_then(|slot| slot.as_ref().map(|secret| secret == text))
        .unwrap_or(false)
}

pub fn copy_plain(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .set_text(text)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

/// Copy text and clear the OS clipboard after a short timeout only if it is still the same text.
/// The copy is also kept out of clipboard history.
pub fn copy_and_clear_after(text: String) -> Result<(), String> {
    suppress_history(&text);
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard unavailable: {}", e))?;
    clipboard
        .set_text(text.clone())
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    thread::spawn(move || {
        thread::sleep(Duration::from_secs(SECRET_CLIPBOARD_CLEAR_SECS));
        if let Ok(mut clip) = Clipboard::new() {
            if let Ok(current) = clip.get_text() {
                if current == text {
                    let _ = clip.clear();
                }
            }
        }
    });
    Ok(())
}

/// Starts a background thread to monitor the system clipboard.
pub fn start_clipboard_monitor(
    history: Arc<Mutex<ClipboardHistory>>,
    filter_enabled: Arc<Mutex<bool>>,
) {
    thread::spawn(move || {
        let mut last_copied = String::new();
        let mut sleep_ms = POLL_ACTIVE_MS;
        let mut idle_polls = 0u32;
        let mut last_change_count: Option<isize> = crate::pasteboard_macos::general_change_count();

        loop {
            thread::sleep(Duration::from_millis(sleep_ms));

            if let Some(count) = crate::pasteboard_macos::general_change_count() {
                if last_change_count == Some(count) {
                    idle_polls += 1;
                    if idle_polls >= IDLE_POLLS_BEFORE_SLOWDOWN {
                        sleep_ms = POLL_IDLE_MS;
                    }
                    continue;
                }
                last_change_count = Some(count);
            }

            idle_polls = 0;
            sleep_ms = POLL_ACTIVE_MS;

            // Read with a short-lived clipboard and never write back. A long-lived
            // pasteboard handle in this process was swallowing Cmd+V into the note.
            let Ok(text) = read_clipboard_text() else {
                continue;
            };

            let trimmed = text.trim().to_string();
            if trimmed.is_empty() || trimmed == last_copied {
                idle_polls += 1;
                if idle_polls >= IDLE_POLLS_BEFORE_SLOWDOWN {
                    sleep_ms = POLL_IDLE_MS;
                }
                continue;
            }

            last_copied = trimmed.clone();

            if history_suppressed(&trimmed) {
                continue;
            }

            let filter_on = filter_enabled.lock().map(|g| *g).unwrap_or(true);

            if let Ok(mut hist) = history.lock() {
                hist.add(trimmed, filter_on);
            }
        }
    });
}
