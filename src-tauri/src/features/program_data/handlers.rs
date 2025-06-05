use crate::data::io::{load_cdda_json_data, DeserializedCDDAJsonData};
use crate::events;
use crate::events::EDITOR_DATA_CHANGED;
use crate::features::program_data::{EditorData, EditorDataSaver};
use crate::features::tileset::legacy_tileset::{
    load_tilesheet, LegacyTilesheet,
};
use crate::util::Save;
use log::{error, warn};
use serde::Serialize;
use std::fs;
use std::ops::DerefMut;
use std::path::PathBuf;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn get_editor_data(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<EditorData, ()> {
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
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), InstallationPickedError> {
    let gfx_dir = fs::read_dir(&path.join("gfx")).map_err(|_| {
        InstallationPickedError::InvalidCDDADirectory(
            "Missing 'gfx' directory".into(),
        )
    })?;

    let mut available_tilesets = vec![];

    for entry in gfx_dir {
        let entry =
            entry.map_err(|e| InstallationPickedError::Io(e.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| InstallationPickedError::Io(e.to_string()))?;

        if file_type.is_file() {
            continue;
        }

        available_tilesets
            .push(entry.file_name().to_string_lossy().into_owned());
    }

    let mut editor_data_lock = editor_data.lock().await;
    editor_data_lock.available_tilesets = Some(available_tilesets);
    editor_data_lock.config.cdda_path = Some(path);

    match load_cdda_json_data(
        &editor_data_lock.config.cdda_path.clone().unwrap(),
        &editor_data_lock.config.json_data_path,
    )
    .await
    {
        Ok(data) => {
            let mut json_data_lock = json_data.lock().await;
            json_data_lock.replace(data);
        },
        Err(e) => {
            warn!("{}", e);
            return Err(InstallationPickedError::InvalidCDDADirectory(
                "Failed to load json data".into(),
            ));
        },
    }

    app.emit(EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .unwrap();

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
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    tilesheet: State<'_, Mutex<Option<LegacyTilesheet>>>,
) -> Result<(), TilesetPickedError> {
    let mut editor_data_lock = editor_data.lock().await;
    let mut tilesheet_lock = tilesheet.lock().await;

    let tilesets = match &editor_data_lock.available_tilesets {
        None => return Err(TilesetPickedError::NoCDDADirPicked),
        Some(t) => t,
    };

    // This is the default tileset
    if tileset == "None" {
        editor_data_lock.config.selected_tileset = None;
    } else {
        match tilesets.iter().find(|t| **t == tileset) {
            None => return Err(TilesetPickedError::NotATileset),
            Some(_) => {},
        }

        editor_data_lock.config.selected_tileset = Some(tileset.clone());
        *tilesheet_lock =
            load_tilesheet(&editor_data_lock).await.map_err(|e| {
                error!("Failed to load tilesheet, `{0}`", e);
                TilesetPickedError::NotATileset
            })?;
    }

    let saver = EditorDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(&editor_data_lock).await.unwrap();

    app.emit(EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .unwrap();

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

    saver.save(&lock).await.map_err(|e| {
        error!("Failed to save editor data, `{0}`", e);
        SaveEditorDataError::SaveFailed(e.to_string())
    })?;

    Ok(())
}
