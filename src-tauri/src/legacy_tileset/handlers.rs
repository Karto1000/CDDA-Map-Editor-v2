use crate::legacy_tileset::{TilesetConfig, TilesetReader};
use image::{ImageFormat, ImageReader};
use std::io::Cursor;
use std::path::PathBuf;
use tauri::ipc::Response;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tileset_metadata(name: String) -> Option<TilesetConfig> {
    let reader = TilesetReader::new(PathBuf::from(name));
    Some(reader.read().unwrap())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_spritesheet(tileset: String, name: String) -> Response {
    let mut path = PathBuf::new();
    path.push(tileset);
    path.push(name);

    let image = ImageReader::open(path).unwrap().decode().unwrap();

    let mut image_data: Vec<u8> = Vec::new();

    image
        .write_to(&mut Cursor::new(&mut image_data), ImageFormat::Png)
        .unwrap();

    Response::new(image_data)
}
