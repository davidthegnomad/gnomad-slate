//! Platform summon shortcut — single source of truth for registration and UI labels.

#[cfg(target_os = "linux")]
pub const SUMMON_SHORTCUT: &str = "Super+Shift+Space";

#[cfg(not(target_os = "linux"))]
pub const SUMMON_SHORTCUT: &str = "Alt+Space";

pub fn summon_shortcut() -> &'static str {
    SUMMON_SHORTCUT
}
