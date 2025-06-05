use crate::features::program_data::{
    CDDAPathError, EditorData, SelectedTilesetError,
};
use crate::features::tileset::legacy_tileset::fallback::{
    get_fallback_config, FALLBACK_TILESHEET_IMAGE,
};
use crate::features::tileset::legacy_tileset::io::LegacyTilesheetConfigLoader;
use log::info;
use serde::Serialize;
use serde_json::Value;
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

    let selected_tileset = match lock.config.get_selected_tileset() {
        Ok(s) => s,
        Err(_) => {
            let config = get_fallback_config();
            return Ok(serde_json::to_value(config).unwrap());
        },
    };

    let cdda_path = lock.config.get_cdda_path()?;

    let tileset_path = cdda_path.join("gfx").join(selected_tileset);

    let mut config_reader = LegacyTilesheetConfigLoader::new(tileset_path);
    let info = config_reader.load_value().await.unwrap();

    Ok(info)
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum DownloadSpritesheetError {
    #[error("No Spritesheet has been selected")]
    NoSpritesheetSelected,

    #[error("Failed to read image")]
    ReadError,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_spritesheet(
    name: String,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<Response, DownloadSpritesheetError> {
    info!("Loading spritesheet {}", &name);

    let lock = editor_data.lock().await;
    let selected_tileset = match &lock.config.selected_tileset {
        None => {
            return Ok(Response::new(FALLBACK_TILESHEET_IMAGE.to_vec()));
        },
        Some(s) => s.clone(),
    };

    let path = lock
        .config
        .cdda_path
        .clone()
        .ok_or(DownloadSpritesheetError::NoSpritesheetSelected)?
        .join("gfx")
        .join(selected_tileset)
        .join(name);

    let image_bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| DownloadSpritesheetError::ReadError)?;

    Ok(Response::new(image_bytes))
}
