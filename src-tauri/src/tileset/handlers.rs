use crate::editor_data::{CDDAPathError, EditorData, SelectedTilesetError};
use crate::tileset::io::TilesheetConfigLoader;
use image::{ImageFormat, ImageReader};
use log::info;
use serde::Serialize;
use serde_json::Value;
use std::io::Cursor;
use std::path::PathBuf;
use tauri::ipc::Response;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum GetSpritesheetsError {
    #[error(transparent)]
    CDDAPathError(#[from] CDDAPathError),

    #[error(transparent)]
    TilesetError(#[from] SelectedTilesetError),
}
#[tauri::command]
pub async fn get_info_of_current_tileset(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<Value, GetSpritesheetsError> {
    let lock = editor_data.lock().await;

    let selected_tileset = lock.config.get_selected_tileset()?;
    let cdda_path = lock.config.get_cdda_path()?;

    let tileset_path = cdda_path.join("gfx").join(selected_tileset);

    let config_reader = TilesheetConfigLoader::new(tileset_path);
    let info = config_reader.load_serde_value().unwrap();

    Ok(info)
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum DownloadSpritesheetError {
    #[error("No Spritesheet has been selected")]
    NoSpritesheetSelected,

    #[error("Failed to decode image")]
    DecodeError,

    #[error("Failed to Encode image")]
    EncodeError,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_spritesheet(
    name: String,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<Response, DownloadSpritesheetError> {
    info!("Loading spritesheet {}", &name);

    let lock = editor_data.lock().await;
    let selected_tileset = match &lock.config.selected_tileset {
        None => return Err(DownloadSpritesheetError::NoSpritesheetSelected),
        Some(s) => s.clone(),
    };

    let mut path = PathBuf::new();
    path.push(selected_tileset);
    path.push(name);

    let image = ImageReader::open(path)
        .map_err(|_| DownloadSpritesheetError::NoSpritesheetSelected)?
        .decode()
        .map_err(|_| DownloadSpritesheetError::DecodeError)?;

    let mut image_data: Vec<u8> = Vec::new();

    image
        .write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)
        .map_err(|_| DownloadSpritesheetError::EncodeError)?;

    Ok(Response::new(image_data))
}
