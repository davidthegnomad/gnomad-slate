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

const POLL_ACTIVE_MS: u64 = 1000;
const POLL_IDLE_MS: u64 = 3000;
const IDLE_POLLS_BEFORE_SLOWDOWN: u32 = 5;

/// Starts a background thread to monitor the system clipboard.
pub fn start_clipboard_monitor(
    history: Arc<Mutex<ClipboardHistory>>,
    filter_enabled: Arc<Mutex<bool>>,
) {
    thread::spawn(move || {
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("clipboard monitor: failed to initialize arboard: {e}");
                return;
            }
        };

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

            let Ok(text) = clipboard.get_text() else {
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

            let filter_on = filter_enabled.lock().map(|g| *g).unwrap_or(true);

            if let Ok(mut hist) = history.lock() {
                hist.add(trimmed, filter_on);
            }
        }
    });
}
