use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::clipboard::ClipboardHistory;
use crate::unlock_limiter::UnlockRateLimiterState;
use crate::vault_session::VaultSessionState;

/// Window close-to-tray and quit lifecycle flags (replaces global statics in `window.rs`).
pub struct WindowLifecycle {
    intentionally_hidden: AtomicBool,
    quit_in_progress: AtomicBool,
}

impl WindowLifecycle {
    pub fn new() -> Self {
        Self {
            intentionally_hidden: AtomicBool::new(false),
            quit_in_progress: AtomicBool::new(false),
        }
    }

    pub fn is_quit_in_progress(&self) -> bool {
        self.quit_in_progress.load(Ordering::SeqCst)
    }

    pub fn mark_quit_in_progress(&self) -> bool {
        self.quit_in_progress.swap(true, Ordering::SeqCst)
    }

    pub fn set_intentionally_hidden(&self, hidden: bool) {
        self.intentionally_hidden.store(hidden, Ordering::SeqCst);
    }

    pub fn is_intentionally_hidden(&self) -> bool {
        self.intentionally_hidden.load(Ordering::SeqCst)
    }
}

/// Shared Tauri managed state for the desktop shell.
pub struct AppState {
    pub clipboard_history: Arc<Mutex<ClipboardHistory>>,
    pub clipboard_filter_enabled: Arc<Mutex<bool>>,
    pub vault_session: Arc<VaultSessionState>,
    pub unlock_limiter: Arc<UnlockRateLimiterState>,
    pub window_lifecycle: WindowLifecycle,
}

#[derive(serde::Serialize)]
pub struct AppVersionInfo {
    pub version: String,
    pub channel: String,
    pub display_version: String,
}
