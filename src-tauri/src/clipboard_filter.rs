//! Heuristic filtering for credential-like clipboard content.
//! Uses explicit patterns + Shannon entropy — avoids rigid mixed-case rules that false-positive on identifiers.

/// Minimum Shannon entropy (bits/char) to treat a token-like string as a secret.
const ENTROPY_THRESHOLD: f64 = 4.2;

/// Token-like secrets are typically at least this long.
const MIN_TOKEN_LEN: usize = 32;

/// Max length for high-entropy token detection.
const MAX_TOKEN_LEN: usize = 256;

/// Returns true if the clipboard text should be excluded from history.
pub fn is_sensitive(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }

    if text.len() > 20_000 {
        return true;
    }

    if matches_credential_pattern(trimmed) {
        return true;
    }

    if looks_like_benign_text(trimmed) {
        return false;
    }

    if looks_like_high_entropy_token(trimmed) {
        return true;
    }

    false
}

/// Emails, prose, markdown, paths, and identifiers should not be blocked.
fn looks_like_benign_text(s: &str) -> bool {
    if s.contains('@') && s.contains('.') {
        return true;
    }

    if s.contains(' ') || s.contains('\n') || s.contains('\t') {
        return true;
    }

    if s.contains("://")
        && !s.starts_with("mysql://")
        && !s.starts_with("postgresql://")
        && !s.starts_with("mongodb://")
        && !s.starts_with("amqp://")
    {
        return true;
    }

    if s.ends_with(".md") || s.ends_with(".ts") || s.ends_with(".tsx") || s.ends_with(".json") {
        return true;
    }

    // camelCase / PascalCase identifiers (no typical secret charset)
    if is_likely_identifier(s) {
        return true;
    }

    false
}

fn is_likely_identifier(s: &str) -> bool {
    if s.len() < 8 || s.len() > 55 {
        return false;
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return false;
    }
    let has_upper = s.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = s.chars().any(|c| c.is_ascii_lowercase());
    has_upper && has_lower && !s.contains('+') && !s.contains('=')
}

fn matches_credential_pattern(s: &str) -> bool {
    const NEEDLES: &[&str] = &[
        "-----BEGIN",
        "PRIVATE KEY",
        "ghp_",
        "github_pat_",
        "sk-proj-",
        "sk-live-",
        "sk-ant-",
        "xoxb-",
        "xoxp-",
        "AKIA",
        "ASIA",
        "AWS_SECRET_ACCESS_KEY",
        "DATABASE_URL",
        "mysql://",
        "postgresql://",
        "postgres://",
        "amqp://",
        "mongodb+srv://",
        "mongodb://",
        "redis://",
        "secret_key",
        "client_secret",
        "api_key",
        "Bearer ",
        "Basic ",
    ];

    let upper = s.to_ascii_uppercase();
    for needle in NEEDLES {
        if s.contains(needle) || upper.contains(&needle.to_ascii_uppercase()) {
            return true;
        }
    }

    false
}

fn looks_like_high_entropy_token(s: &str) -> bool {
    let len = s.len();
    if len < MIN_TOKEN_LEN || len > MAX_TOKEN_LEN {
        return false;
    }

    if s.chars().any(|c| c.is_whitespace() || c == '@') {
        return false;
    }

    let token_chars = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '+' | '/' | '='));

    if !token_chars {
        return false;
    }

    shannon_entropy(s) >= ENTROPY_THRESHOLD
}

/// Shannon entropy in bits per character.
pub fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    let mut counts = [0u32; 256];
    let mut len = 0usize;
    for &b in s.as_bytes() {
        counts[b as usize] += 1;
        len += 1;
    }

    let len_f = len as f64;
    let mut entropy = 0.0;
    for &count in &counts {
        if count == 0 {
            continue;
        }
        let p = count as f64 / len_f;
        entropy -= p * p.log2();
    }
    entropy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_camel_case_class_names() {
        assert!(!is_sensitive("MySuperCoolConfigurationHelperClassName"));
        assert!(!is_sensitive("MySuperCoolConfigurationHelperClassName123"));
    }

    #[test]
    fn allows_email_addresses() {
        assert!(!is_sensitive("Developer.John.Doe.1994@Gmail.com"));
    }

    #[test]
    fn allows_prose_with_spaces() {
        assert!(!is_sensitive("This is a normal sentence with Mixed Case words."));
    }

    #[test]
    fn blocks_github_pat_prefix() {
        assert!(is_sensitive(
            "ghp_1234567890abcdefghijklmnopqrstuvwxyz1234567890"
        ));
    }

    #[test]
    fn blocks_private_key_header() {
        assert!(is_sensitive("-----BEGIN OPENSSH PRIVATE KEY-----\nAAA"));
    }

    #[test]
    fn blocks_high_entropy_token() {
        let token = "aB3xK9mN2pQ7rT5vW8yZ1cD4fG6hJ0kL3nM5pR8sU2wX7zA9bC1dE4fG6";
        assert!(token.len() >= MIN_TOKEN_LEN);
        assert!(is_sensitive(token));
    }

    #[test]
    fn entropy_empty_is_zero() {
        assert_eq!(shannon_entropy(""), 0.0);
    }
}
