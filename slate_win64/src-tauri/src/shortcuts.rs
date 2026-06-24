//! Platform summon shortcut — single source of truth for registration and UI labels.
//! Windows edition: Alt+Space (matches Windows PowerToys Run style).

pub const SUMMON_SHORTCUT: &str = "Alt+Space";

pub fn summon_shortcut() -> &'static str {
    SUMMON_SHORTCUT
}
