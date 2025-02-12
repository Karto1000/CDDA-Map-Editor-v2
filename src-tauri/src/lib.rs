mod editor_data;
mod legacy_tileset;
mod map_data;
mod util;

use crate::editor_data::handlers::{
    cdda_installation_directory_picked, get_editor_data, save_editor_data, tileset_picked,
};
use crate::editor_data::tab::handlers::{close_tab, create_tab};
use crate::editor_data::EditorData;
use crate::legacy_tileset::handlers::{download_spritesheet, get_tileset_metadata};
use crate::map_data::handlers::{close_map, create_map, get_current_map_data};
use crate::map_data::handlers::{open_map, place};
use crate::map_data::{MapData, MapDataContainer};
use directories::ProjectDirs;
use image::GenericImageView;
use log::{error, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};

#[tauri::command]
async fn frontend_ready(
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), ()> {
    let lock = editor_data.lock().await;

    for tab in &lock.tabs {
        info!("Opened Tab {}", &tab.name);
        app.emit("tab_created", tab).expect("Emit to not fail");
    }

    info!("Sent inital editor data change");
    app.emit("editor_data_changed", lock.clone())
        .expect("Emit to not fail");

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> () {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new()
            .level(LevelFilter::Debug)
            .targets(vec![Target::new(TargetKind::Webview), Target::new(TargetKind::Stdout)])
            .build()
        )
        .setup(|app| {
            let project_dir = ProjectDirs::from("", "", "CDDA Map Editor");

            let directory_path = match project_dir {
                None => {
                    warn!("No valid project directory found, creating data folder application directory instead");
                    let app_dir = match std::env::current_dir() {
                        Ok(d) => d,
                        Err(e) => {
                            app.dialog()
                                .message(e.to_string())
                                .kind(MessageDialogKind::Error)
                                .title("Error")
                                .blocking_show();

                            app.app_handle().exit(1);
                            unreachable!();
                        }
                    };

                    app_dir
                }
                Some(dir) => {
                    let local_dir = dir.config_local_dir();
                    info!("Got Path for CDDA-Map-Editor config directory at {:?}", local_dir);
                    local_dir.to_path_buf()
                }
            };

            if !fs::exists(&directory_path).expect("IO Error to not occur") {
                info!("Created CDDA-Map-Editor config directory at {:?}", directory_path);
                fs::create_dir_all(&directory_path)?;
            }

            let config_file_path = directory_path.join("config.json");
            let config_exists = fs::exists(&config_file_path).expect("IO Error to not occur");
            let config = match config_exists {
                true => {
                    info!("Reading config.json file");
                    let contents = fs::read_to_string(&config_file_path).expect("File to be valid UTF-8");

                    let data = match serde_json::from_str::<EditorData>(contents.as_str()) {
                        Ok(d) => {
                            info!("config.json file successfully read and parsed");
                            d
                        }
                        Err(e) => {
                            error!("{}", e.to_string());

                            let full_message = format!(r#"
                               An error occurred while reading the config.json file at {:?}.
                               This is likely due to the file containing unexpected or invalid data.

                               To fix this, you can regenerate the file. However, this would delete
                               your current configuration and reset it to the default state.

                               Do you want to continue?
                            "#, config_file_path);

                            let answer = app.dialog()
                                .message(full_message)
                                .title("Failed to read config.json file")
                                .kind(MessageDialogKind::Warning)
                                .buttons(MessageDialogButtons::YesNo)
                                .blocking_show();

                            let data = match answer {
                                true => {
                                    fs::remove_file(&config_file_path).expect("File to have been deleted");
                                    let mut default_editor_data = EditorData::default();
                                    default_editor_data.config.config_path = directory_path.clone();

                                    let serialized = serde_json::to_string_pretty(&default_editor_data).expect("Serialization to not fail");
                                    fs::write(&config_file_path, serialized).expect("Directory path to config to have been created");
                                    default_editor_data
                                }
                                false => {
                                    app.app_handle().exit(1);
                                    unreachable!();
                                }
                            };

                            data
                        }
                    };

                    data
                }
                false => {
                    info!("config.json file does not exist");
                    info!("Creating config.json file with default data");

                    let mut default_editor_data = EditorData::default();
                    default_editor_data.config.config_path = directory_path.clone();

                    let serialized = serde_json::to_string_pretty(&default_editor_data).expect("Serialization to not fail");
                    fs::write(&config_file_path, serialized).expect("Directory path to config to have been created");
                    default_editor_data
                }
            };

            app.manage(Mutex::new(config));

            let mut map_data = MapDataContainer::default();
            // For Testing
            map_data.data.push(MapData::new("test".into()));

            app.manage(Mutex::new(map_data));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tileset_metadata,
            download_spritesheet,
            get_current_map_data,
            place,
            get_editor_data,
            cdda_installation_directory_picked,
            tileset_picked,
            save_editor_data,
            create_tab,
            close_tab,
            frontend_ready,
            create_map,
            open_map,
            close_map
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
