use crate::filesystem::{self, Note};

#[tauri::command]
pub fn list_notes() -> Result<Vec<Note>, String> {
    filesystem::list_notes()
}

#[tauri::command]
pub fn read_note(filename: String) -> Result<String, String> {
    filesystem::read_note(&filename)
}

#[tauri::command]
pub fn write_note(
    title: String,
    content: String,
    is_ephemeral: bool,
    previous_filename: Option<String>,
) -> Result<Note, String> {
    filesystem::write_note(
        &title,
        &content,
        is_ephemeral,
        previous_filename.as_deref(),
    )
}

#[tauri::command]
pub fn delete_note(filename: String) -> Result<(), String> {
    filesystem::delete_note(&filename)
}

#[tauri::command]
pub fn archive_notes() -> Result<usize, String> {
    filesystem::archive_stale_ephemeral_notes()
}
