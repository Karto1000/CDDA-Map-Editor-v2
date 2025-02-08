use crate::editor_data::{EditorData, EditorDataSaver};
use crate::util::Save;
use log::{error, info};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::async_runtime::Mutex;
use tauri::State;

#[tauri::command]
pub async fn get_editor_data(editor_data: State<'_, Mutex<EditorData>>) -> Result<EditorData, ()> {
    Ok(editor_data.lock().await.clone())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum InstallationPickedError {
    #[error("Picked directory is not a valid CDDA Directory, reason: `{0}`")]
    InvalidCDDADirectory(String),

    #[error("IO Error, `{0}`")]
    Io(String),
}

#[tauri::command]
pub async fn cdda_installation_directory_picked(
    path: PathBuf,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), InstallationPickedError> {
    let gfx_dir = fs::read_dir(&path.join("gfx")).map_err(|_| {
        InstallationPickedError::InvalidCDDADirectory("Missing 'gfx' directory".into())
    })?;

    let mut available_tilesets = vec![];

    for entry in gfx_dir {
        let entry = entry.map_err(|e| InstallationPickedError::Io(e.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| InstallationPickedError::Io(e.to_string()))?;

        if file_type.is_file() {
            continue;
        }

        available_tilesets.push(entry.file_name().to_string_lossy().into_owned());
    }

    let mut lock = editor_data.lock().await;
    lock.available_tilesets = Some(available_tilesets);
    lock.config.cdda_path = Some(path);

    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum TilesetPickedError {
    #[error("The selected tileset does not exist")]
    NotATileset,

    #[error("No CDDA game directory has been picked")]
    NoCDDADirPicked,
}

#[tauri::command]
pub async fn tileset_picked(
    tileset: String,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), TilesetPickedError> {
    let mut lock = editor_data.lock().await;

    let tilesets = match &lock.available_tilesets {
        None => return Err(TilesetPickedError::NoCDDADirPicked),
        Some(t) => t,
    };

    // This is the default tileset
    if tileset == "None" {
        lock.config.selected_tileset = None;
        return Ok(());
    }

    match tilesets.iter().find(|t| **t == tileset) {
        None => return Err(TilesetPickedError::NotATileset),
        Some(_) => {}
    }

    lock.config.selected_tileset = Some(tileset.clone());

    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum SaveEditorDataError {
    #[error("Failed to save editor data, `{0}`")]
    SaveFailed(String),
}

#[tauri::command]
pub async fn save_editor_data(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), SaveEditorDataError> {
    let lock = editor_data.lock().await;

    let saver = EditorDataSaver {
        path: lock.config.config_path.clone(),
    };

    saver.save(&lock).map_err(|e| {
        error!("Failed to save editor data, `{0}`", e);
        SaveEditorDataError::SaveFailed(e.to_string())
    })?;

    info!("Saved EditorData to {:?}", &lock.config.config_path);

    Ok(())
}
