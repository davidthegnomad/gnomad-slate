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

/// Portable app data directory: XDG `~/.local/share/gnomad-slate` on Linux.
fn platform_data_root() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .unwrap_or_else(|| PathBuf::from("."))
    }
    #[cfg(not(target_os = "linux"))]
    {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
    }
}

/// Portable app data directory: `~/Library/Application Support/gnomad-slate` (macOS) or XDG equivalent.
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

fn sanitize_note_stem(title: &str) -> Result<String, String> {
    let safe_title = title.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    if safe_title.is_empty() {
        return Err("Note title cannot be empty".to_string());
    }
    Ok(safe_title)
}

fn filename_taken(name: &str, existing: &[String], previous: Option<&str>) -> bool {
    if previous == Some(name) {
        return false;
    }
    existing.iter().any(|item| item == name)
}

/// Picks `prefix + stem.md`, or `prefix + stem-N.md` when that file belongs to a different note.
pub fn allocate_note_filename(
    existing: &[String],
    is_ephemeral: bool,
    title: &str,
    previous_filename: Option<&str>,
) -> Result<String, String> {
    let safe_title = sanitize_note_stem(title)?;
    let prefix = if is_ephemeral { "temp_" } else { "vault_" };
    let previous = previous_filename.filter(|name| validate_note_filename(name).is_ok());
    let primary = format!("{prefix}{safe_title}.md");
    validate_note_filename(&primary)?;
    if !filename_taken(&primary, existing, previous) {
        return Ok(primary);
    }
    for n in 2..10_000 {
        let candidate = format!("{prefix}{safe_title}-{n}.md");
        validate_note_filename(&candidate)?;
        if !filename_taken(&candidate, existing, previous) {
            return Ok(candidate);
        }
    }
    Err("Could not choose a unique note name".to_string())
}

fn markdown_filenames(notes_dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(notes_dir) else {
        return names;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            if let Some(name) = path.file_name() {
                names.push(name.to_string_lossy().into_owned());
            }
        }
    }
    names
}

fn ensure_note_parent(notes_dir: &Path, file_path: &Path) -> Result<(), String> {
    if let Ok(canonical_notes) = fs::canonicalize(notes_dir) {
        if let Some(parent) = file_path.parent() {
            if let Ok(canonical_parent) = fs::canonicalize(parent) {
                if !canonical_parent.starts_with(&canonical_notes) {
                    return Err("Access denied: path outside notes directory".to_string());
                }
            }
        }
    }
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create note directory: {}", e))?;
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| "Invalid note path".to_string())?;
    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(".tmp");
    let tmp = path.with_file_name(&tmp_name);
    {
        let mut file = File::create(&tmp).map_err(|e| format!("Failed to create note: {}", e))?;
        file.write_all(bytes)
            .map_err(|e| format!("Failed to write note content: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync note: {}", e))?;
    }
    replace_file(&tmp, path)
}

fn replace_file(tmp: &Path, dest: &Path) -> Result<(), String> {
    match fs::rename(tmp, dest) {
        Ok(()) => Ok(()),
        Err(err) => {
            if !dest.exists() {
                return Err(format!("Failed to finalize note: {err}"));
            }
            let mut backup_name = dest
                .file_name()
                .ok_or_else(|| "Invalid note path".to_string())?
                .to_os_string();
            backup_name.push(".bak");
            let backup = dest.with_file_name(backup_name);
            if let Err(e) = fs::rename(dest, &backup) {
                let _ = fs::remove_file(tmp);
                return Err(format!("Failed to finalize note: {e}"));
            }
            if let Err(e) = fs::rename(tmp, dest) {
                let _ = fs::rename(&backup, dest);
                return Err(format!("Failed to finalize note: {e}"));
            }
            let _ = fs::remove_file(&backup);
            Ok(())
        }
    }
}

fn strip_ephemeral_marker(content: &str) -> String {
    content
        .strip_prefix("<!-- ephemeral -->\n")
        .unwrap_or(content)
        .to_string()
}

/// Joins a new daily-note block onto the existing body. An empty note gets `empty_header` first.
pub fn combine_daily_note(existing: &str, block: &str, empty_header: &str) -> String {
    let existing = existing.trim_end();
    let block = block.trim();
    if existing.is_empty() {
        if empty_header.is_empty() || block.is_empty() {
            if block.is_empty() {
                return empty_header.to_string();
            }
            return block.to_string();
        }
        return format!("{empty_header}\n\n{block}");
    }
    if block.is_empty() {
        return existing.to_string();
    }
    format!("{existing}\n\n{block}")
}

fn daily_header_for_title(title: &str) -> String {
    let date = title.strip_prefix("Daily_Note_").unwrap_or(title);
    format!("## Log: {date}")
}

pub fn write_note(
    title: &str,
    content: &str,
    is_ephemeral: bool,
    previous_filename: Option<&str>,
) -> Result<Note, String> {
    write_note_in(
        &get_notes_dir(),
        title,
        content,
        is_ephemeral,
        previous_filename,
    )
}

fn write_note_in(
    notes_dir: &Path,
    title: &str,
    content: &str,
    is_ephemeral: bool,
    previous_filename: Option<&str>,
) -> Result<Note, String> {
    fs::create_dir_all(notes_dir).map_err(|e| format!("Failed to create notes directory: {}", e))?;

    let existing = markdown_filenames(notes_dir);
    let filename = allocate_note_filename(&existing, is_ephemeral, title, previous_filename)?;
    let file_path = notes_dir.join(&filename);
    ensure_note_parent(notes_dir, &file_path)?;

    let mut full_content = content.to_string();
    if is_ephemeral && !full_content.contains("<!-- ephemeral -->") {
        full_content = format!("<!-- ephemeral -->\n{full_content}");
    }

    atomic_write(&file_path, full_content.as_bytes())?;

    if let Some(prev) = previous_filename {
        if prev != filename && validate_note_filename(prev).is_ok() {
            let prev_path = notes_dir.join(prev);
            if prev_path.exists() {
                let _ = fs::remove_file(prev_path);
            }
        }
    }

    Ok(Note {
        filename,
        title: title.to_string(),
        content: full_content,
        is_ephemeral,
        last_modified: get_current_time(),
    })
}

pub fn write_daily_note(title: &str, block: &str, append: bool) -> Result<Note, String> {
    write_daily_note_in(&get_notes_dir(), title, block, append)
}

fn write_daily_note_in(
    notes_dir: &Path,
    title: &str,
    block: &str,
    append: bool,
) -> Result<Note, String> {
    fs::create_dir_all(notes_dir).map_err(|e| format!("Failed to create notes directory: {}", e))?;
    let stem = sanitize_note_stem(title)?;
    let filename = format!("vault_{stem}.md");
    validate_note_filename(&filename)?;
    let path = notes_dir.join(&filename);
    let existing_raw = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("Failed to read note: {}", e))?
    } else {
        String::new()
    };
    let existing = strip_ephemeral_marker(&existing_raw);
    let content = if append {
        combine_daily_note(&existing, block, &daily_header_for_title(title))
    } else if !existing.trim().is_empty() {
        existing
    } else {
        block.to_string()
    };
    let previous = if path.exists() {
        Some(filename.as_str())
    } else {
        None
    };
    write_note_in(notes_dir, title, &content, false, previous)
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
    fn slate_dir_uses_xdg_data_home_on_linux() {
        #[cfg(target_os = "linux")]
        {
            let slate = super::get_slate_dir();
            if let Some(home) = dirs::home_dir() {
                assert!(
                    slate.starts_with(home.join(".local").join("share")),
                    "expected XDG data dir, got {}",
                    slate.display()
                );
            }
        }
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

    fn temp_notes_dir() -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("slate-notes-{}-{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).expect("temp notes dir");
        dir
    }

    #[test]
    fn distinct_sanitized_titles_do_not_share_a_filename() {
        let first = allocate_note_filename(&[], false, "My Note", None).expect("first");
        assert_eq!(first, "vault_My_Note.md");
        let second = allocate_note_filename(&[first.clone()], false, "My_Note", None).expect("second");
        assert_eq!(second, "vault_My_Note-2.md");
    }

    #[test]
    fn same_note_keeps_its_filename_when_previous_matches() {
        let name = allocate_note_filename(
            &["vault_My_Note.md".to_string()],
            false,
            "My Note",
            Some("vault_My_Note.md"),
        )
        .expect("name");
        assert_eq!(name, "vault_My_Note.md");
    }

    #[test]
    fn second_note_does_not_truncate_an_unrelated_file() {
        let dir = temp_notes_dir();
        let first = super::write_note_in(&dir, "My Note", "original-body", false, None).expect("first");
        let second = super::write_note_in(&dir, "My_Note", "other-body", false, None).expect("second");
        assert_ne!(first.filename, second.filename);
        let original = std::fs::read_to_string(dir.join(&first.filename)).expect("read");
        assert_eq!(original, "original-body");
        let other = std::fs::read_to_string(dir.join(&second.filename)).expect("read other");
        assert_eq!(other, "other-body");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn daily_append_keeps_the_existing_journal() {
        let dir = temp_notes_dir();
        let created = super::write_daily_note_in(
            &dir,
            "Daily_Note_2026-10-23",
            "## Log: 2026-10-23\n\n*   first",
            false,
        )
        .expect("create");
        assert_eq!(created.filename, "vault_Daily_Note_2026-10-23.md");
        let appended =
            super::write_daily_note_in(&dir, "Daily_Note_2026-10-23", "*   second", true).expect("append");
        assert_eq!(appended.filename, created.filename);
        assert!(appended.content.contains("first"));
        assert!(appended.content.contains("*   second"));
        let on_disk = std::fs::read_to_string(dir.join(&created.filename)).expect("disk");
        assert_eq!(on_disk, appended.content);
        let again = super::write_daily_note_in(
            &dir,
            "Daily_Note_2026-10-23",
            "## Log: 2026-10-23\n\n*   replaced",
            false,
        )
        .expect("create again");
        assert!(again.content.contains("first"));
        assert!(!again.content.contains("replaced"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn combine_daily_note_appends_instead_of_replacing() {
        let combined = combine_daily_note("## Log: 2026-10-23\n\n*   first", "*   second", "## Log: 2026-10-23");
        assert!(combined.contains("first"));
        assert!(combined.ends_with("*   second"));
        let fresh = combine_daily_note("", "*   second", "## Log: 2026-10-23");
        assert!(fresh.starts_with("## Log: 2026-10-23"));
        assert!(fresh.contains("*   second"));
    }
}
