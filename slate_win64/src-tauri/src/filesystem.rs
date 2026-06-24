use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub filename: String,
    pub title: String,
    pub content: String,
    pub is_ephemeral: bool,
    pub last_modified: u64,
}

/// Legacy dev data path relative to the current user's home directory (not a fixed machine path).
fn get_legacy_slate_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(".gemini")
            .join("antigravity")
            .join("scratch")
            .join("slate_data")
    })
}

/// Portable app data directory:
/// Windows: `%APPDATA%/gnomad-slate` (via dirs::data_dir -> RoamingAppData)
fn platform_data_root() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Portable app data directory:
/// Windows: `%APPDATA%/gnomad-slate` (RoamingAppData)
pub fn get_slate_dir() -> PathBuf {
    let modern = platform_data_root().join("gnomad-slate");

    if modern.exists() {
        return modern;
    }

    if let Some(legacy) = get_legacy_slate_dir() {
        if legacy.exists() {
            if let Err(e) = migrate_legacy_dir(&legacy, &modern) {
                tracing::warn!("legacy data migration failed: {e}");
                return legacy;
            }
            return modern;
        }
    }

    if let Err(e) = fs::create_dir_all(&modern) {
        tracing::warn!("failed to create data directory: {e}");
    }
    modern
}

fn migrate_legacy_dir(legacy: &Path, modern: &Path) -> Result<(), String> {
    if let Some(parent) = modern.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::rename(legacy, modern).map_err(|e| {
        format!(
            "Could not move legacy data dir to {}: {}",
            modern.display(),
            e
        )
    })
}

pub fn get_notes_dir() -> PathBuf {
    let path = get_slate_dir().join("notes");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

pub fn get_archive_dir() -> PathBuf {
    let path = get_slate_dir().join("archive");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

/// Rejects path traversal in vault secret path labels (not filesystem paths).
pub fn validate_secret_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Secret path cannot be empty".to_string());
    }
    if path.contains('/') || path.contains('\\') || path.contains("..") {
        return Err("Invalid secret path".to_string());
    }
    if path.chars().any(|c| c.is_control()) {
        return Err("Invalid secret path".to_string());
    }
    Ok(())
}

/// Rejects path traversal and non-markdown basenames.
pub fn validate_note_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return Err("Invalid filename".to_string());
    }
    if !filename.ends_with(".md") {
        return Err("Note files must be .md".to_string());
    }
    Ok(())
}

/// Resolves a note path and ensures it stays inside the notes directory.
pub fn resolve_note_path(filename: &str) -> Result<PathBuf, String> {
    validate_note_filename(filename)?;

    let notes_dir = get_notes_dir();
    let file_path = notes_dir.join(filename);

    if !file_path.exists() {
        return Err(format!("Note does not exist: {}", filename));
    }

    let canonical_notes = fs::canonicalize(&notes_dir)
        .map_err(|e| format!("Notes directory error: {}", e))?;
    let canonical_file = fs::canonicalize(&file_path)
        .map_err(|e| format!("Note path error: {}", e))?;

    if !canonical_file.starts_with(&canonical_notes) {
        return Err("Access denied: path outside notes directory".to_string());
    }

    Ok(canonical_file)
}

fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn list_notes() -> Result<Vec<Note>, String> {
    let notes_dir = get_notes_dir();
    let mut notes = Vec::new();

    let entries = fs::read_dir(&notes_dir)
        .map_err(|e| format!("Failed to read notes directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();
            let content = fs::read_to_string(&path).unwrap_or_default();

            let title = path.file_stem().unwrap().to_string_lossy().into_owned();

            let is_ephemeral =
                filename.starts_with("temp_") || content.contains("<!-- ephemeral -->");

            let last_modified = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or_else(|_| SystemTime::now())
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            notes.push(Note {
                filename,
                title,
                content,
                is_ephemeral,
                last_modified,
            });
        }
    }

    notes.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(notes)
}

pub fn read_note(filename: &str) -> Result<String, String> {
    let file_path = resolve_note_path(filename)?;
    fs::read_to_string(file_path).map_err(|e| format!("Failed to read note: {}", e))
}

pub fn write_note(
    title: &str,
    content: &str,
    is_ephemeral: bool,
    previous_filename: Option<&str>,
) -> Result<Note, String> {
    let notes_dir = get_notes_dir();

    let safe_title = title.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    if safe_title.is_empty() {
        return Err("Note title cannot be empty".to_string());
    }

    let prefix = if is_ephemeral { "temp_" } else { "vault_" };
    let filename = format!("{}{}.md", prefix, safe_title);
    validate_note_filename(&filename)?;

    let file_path = notes_dir.join(&filename);
    if let Ok(canonical_notes) = fs::canonicalize(&notes_dir) {
        if let Some(parent) = file_path.parent() {
            if let Ok(canonical_parent) = fs::canonicalize(parent) {
                if !canonical_parent.starts_with(&canonical_notes) {
                    return Err("Access denied: path outside notes directory".to_string());
                }
            }
        }
    }

    let mut full_content = content.to_string();
    if is_ephemeral && !full_content.contains("<!-- ephemeral -->") {
        full_content = format!("<!-- ephemeral -->\n{}", full_content);
    }

    let mut file = File::create(&file_path)
        .map_err(|e| format!("Failed to create note: {}", e))?;

    file.write_all(full_content.as_bytes())
        .map_err(|e| format!("Failed to write note content: {}", e))?;

    if let Some(prev) = previous_filename {
        if prev != filename {
            if validate_note_filename(prev).is_ok() {
                let prev_path = notes_dir.join(prev);
                if prev_path.exists() {
                    let _ = fs::remove_file(prev_path);
                }
            }
        }
    }

    let last_modified = get_current_time();

    Ok(Note {
        filename,
        title: title.to_string(),
        content: full_content,
        is_ephemeral,
        last_modified,
    })
}

pub fn delete_note(filename: &str) -> Result<(), String> {
    let file_path = resolve_note_path(filename)?;
    fs::remove_file(file_path).map_err(|e| format!("Failed to delete note: {}", e))
}

pub fn archive_stale_ephemeral_notes() -> Result<usize, String> {
    let notes_dir = get_notes_dir();
    let archive_dir = get_archive_dir();
    let now = get_current_time();
    let forty_eight_hours = 172_800;
    let mut archived_count = 0;

    let entries = fs::read_dir(&notes_dir)
        .map_err(|e| format!("Failed to read notes directory for archive: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();

            if filename.starts_with("temp_") {
                let metadata = path.metadata().map_err(|e| e.to_string())?;
                let last_modified = metadata
                    .modified()
                    .unwrap_or_else(|_| SystemTime::now())
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if now - last_modified > forty_eight_hours {
                    let destination = archive_dir.join(&filename);
                    fs::rename(&path, &destination)
                        .map_err(|e| format!("Failed to archive file: {}", e))?;
                    archived_count += 1;
                }
            }
        }
    }

    Ok(archived_count)
}

#[cfg(test)]
mod tests {
    use super::*;

  #[test]
  fn rejects_invalid_secret_paths() {
    assert!(validate_secret_path("../etc/passwd").is_err());
    assert!(validate_secret_path("foo/bar").is_err());
    assert!(validate_secret_path("").is_err());
    assert!(validate_secret_path("api-key").is_ok());
  }

  #[test]
  fn rejects_path_traversal_filenames() {
        assert!(validate_note_filename("temp_../../../etc/passwd").is_err());
        assert!(validate_note_filename("temp_..\\config.json").is_err());
        assert!(validate_note_filename("../notes/secret.md").is_err());
        assert!(validate_note_filename("").is_err());
        assert!(validate_note_filename("not-markdown.txt").is_err());
    }

    #[test]
    fn accepts_valid_note_filenames() {
        assert!(validate_note_filename("temp_scratch.md").is_ok());
        assert!(validate_note_filename("vault_my-note.md").is_ok());
    }

    #[test]
    fn legacy_dir_is_under_user_home() {
        let legacy = super::get_legacy_slate_dir();
        if let Some(path) = legacy {
            let home = dirs::home_dir().expect("home dir");
            assert!(path.starts_with(&home));
            assert!(path.to_string_lossy().contains(".gemini"));
        }
    }
}
