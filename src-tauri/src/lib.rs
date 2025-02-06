mod legacy_tileset;
mod map_data;
mod util;

use crate::legacy_tileset::handlers::{download_spritesheet, get_tileset_metadata};
use crate::map_data::handlers::{get_map_data, place};
use crate::map_data::MapData;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> () {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.manage(Mutex::new(MapData::new("Unnamed".into())));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tileset_metadata,
            download_spritesheet,
            get_map_data,
            place
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
