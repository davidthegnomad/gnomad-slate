use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub filename: String,
    pub title: String,
    pub content: String,
    pub is_ephemeral: bool,
    pub last_modified: u64,
}

/// Returns the base directory for Slate data: `/Users/gnomadstudio/.gemini/antigravity/scratch/slate_data`
pub fn get_slate_dir() -> PathBuf {
    let path = PathBuf::from("/Users/gnomadstudio/.gemini/antigravity/scratch/slate_data");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

/// Returns the notes directory path: `<slate_dir>/notes`
pub fn get_notes_dir() -> PathBuf {
    let path = get_slate_dir().join("notes");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

/// Returns the archive directory path: `<slate_dir>/archive`
pub fn get_archive_dir() -> PathBuf {
    let path = get_slate_dir().join("archive");
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }
    path
}

/// Helper to get current Unix timestamp in seconds
fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Lists all notes in the active notes directory
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
            
            // Extract title from filename (remove .md)
            let title = path.file_stem().unwrap().to_string_lossy().into_owned();
            
            // Check metadata (if note starts with is_ephemeral comments or has ephemeral name suffix)
            let is_ephemeral = filename.starts_with("temp_") || content.contains("<!-- ephemeral -->");
            
            let last_modified = path.metadata()
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

    // Sort notes by last modified descending
    notes.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(notes)
}

/// Reads a specific note by filename
pub fn read_note(filename: &str) -> Result<String, String> {
    let file_path = get_notes_dir().join(filename);
    if !file_path.exists() {
        return Err(format!("Note does not exist: {}", filename));
    }
    fs::read_to_string(file_path).map_err(|e| format!("Failed to read note: {}", e))
}

/// Saves a note (writes to disk)
pub fn write_note(title: &str, content: &str, is_ephemeral: bool) -> Result<Note, String> {
    let notes_dir = get_notes_dir();
    
    // Sanitize title for filename
    let safe_title = title.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
    let prefix = if is_ephemeral { "temp_" } else { "vault_" };
    let filename = format!("{}{}.md", prefix, safe_title);
    let file_path = notes_dir.join(&filename);
    
    // Add optional markdown indicator for ephemerality
    let mut full_content = content.to_string();
    if is_ephemeral && !full_content.contains("<!-- ephemeral -->") {
        full_content = format!("<!-- ephemeral -->\n{}", full_content);
    }

    let mut file = File::create(&file_path)
        .map_err(|e| format!("Failed to create note: {}", e))?;
    
    file.write_all(full_content.as_bytes())
        .map_err(|e| format!("Failed to write note content: {}", e))?;

    let last_modified = get_current_time();

    Ok(Note {
        filename,
        title: title.to_string(),
        content: full_content,
        is_ephemeral,
        last_modified,
    })
}

/// Deletes a specific note
pub fn delete_note(filename: &str) -> Result<(), String> {
    let file_path = get_notes_dir().join(filename);
    if file_path.exists() {
        fs::remove_file(file_path).map_err(|e| format!("Failed to delete note: {}", e))?;
    }
    Ok(())
}

/// Automatically sweeps the notes directory and archives ephemeral notes older than 48 hours (in seconds: 172800)
pub fn archive_stale_ephemeral_notes() -> Result<usize, String> {
    let notes_dir = get_notes_dir();
    let archive_dir = get_archive_dir();
    let now = get_current_time();
    let forty_eight_hours = 172_800; // 48 hours in seconds
    let mut archived_count = 0;

    let entries = fs::read_dir(&notes_dir)
        .map_err(|e| format!("Failed to read notes directory for archive: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_string_lossy().into_owned();
            
            // Check if it is an ephemeral note
            if filename.starts_with("temp_") {
                let metadata = path.metadata().map_err(|e| e.to_string())?;
                let last_modified = metadata.modified()
                    .unwrap_or_else(|_| SystemTime::now())
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if now - last_modified > forty_eight_hours {
                    // Move to archive folder
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
