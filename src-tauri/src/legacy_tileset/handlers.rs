use crate::legacy_tileset::{TilesetConfig, TilesetReader};
use image::{ImageError, ImageFormat, ImageReader};
use log::info;
use serde::Serialize;
use std::io::Cursor;
use std::path::PathBuf;
use tauri::ipc::Response;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tileset_metadata(name: String) -> Option<TilesetConfig> {
    let reader = TilesetReader::new(PathBuf::from(name));
    reader.read().ok()
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum DownloadSpritesheetError {
    #[error("The selected spritesheet could not be found")]
    SpritesheetNotFound,

    #[error("Failed to decode image")]
    DecodeError,

    #[error("Failed to Encode image")]
    EncodeError,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_spritesheet(
    tileset: String,
    name: String,
) -> Result<Response, DownloadSpritesheetError> {
    info!("Loading spritesheet {}", name);

    let mut path = PathBuf::new();
    path.push(tileset);
    path.push(name);

    let image = ImageReader::open(path)
        .map_err(|_| DownloadSpritesheetError::SpritesheetNotFound)?
        .decode()
        .map_err(|_| DownloadSpritesheetError::DecodeError)?;

    let mut image_data: Vec<u8> = Vec::new();

    image
        .write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)
        .map_err(|_| DownloadSpritesheetError::EncodeError)?;

    Ok(Response::new(image_data))
}
