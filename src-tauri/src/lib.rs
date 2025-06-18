mod data;
mod events;
mod features;
mod util;

use crate::data::io::{load_cdda_json_data, DeserializedCDDAJsonData};
use crate::features::editor::handler::new_map_editor;
use crate::features::editor::MapEditor;
use crate::features::program_data::handlers::{
    cdda_installation_directory_picked, close_project,
    get_current_project_data, get_editor_data, open_project,
    open_recent_project, save_program_data, tileset_picked,
};
use crate::features::program_data::{
    get_map_data_collection_from_map_viewer, MappedCDDAIdContainer, ProgramData, ProjectName, ProjectType,
    ZLevel,
};
use crate::features::tileset::handlers::{
    download_spritesheet, get_info_of_current_tileset,
};
use crate::features::tileset::legacy_tileset::fallback::get_fallback_tilesheet;
use crate::features::tileset::legacy_tileset::LegacyTilesheet;
use crate::features::viewer::handlers::{
    create_viewer, get_calculated_parameters, get_project_cell_data,
    get_sprites, new_nested_mapgen_viewer, new_single_mapgen_viewer,
    new_special_mapgen_viewer, reload_project,
};
use crate::features::viewer::MapViewer;
use crate::util::Load;
use anyhow::{anyhow, Error};
use async_once::AsyncOnce;
use data::io;
use features::program_data::{Tab, TabType};
use features::tileset::legacy_tileset;
use features::toast::ToastMessage;
use features::viewer::LiveViewerData;
use lazy_static::lazy_static;
use log::{info, warn, LevelFilter};
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use tauri::async_runtime::{block_on, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_log::{Target, TargetKind};
use tokio::task::JoinHandle;

lazy_static! {
    static ref TEST_CDDA_DATA: AsyncOnce<DeserializedCDDAJsonData> =
        AsyncOnce::new(async {
            dotenv::dotenv().unwrap();
            env_logger::init();

            info!("Loading CDDA data");

            let cdda_path = std::env::var("CDDA_INSTALL_PATH")
                .expect("CDDA_INSTALL_PATH not set");
            let cdda_json_path = std::env::var("CDDA_JSON_PATH")
                .unwrap_or("data\\json\\".to_string());

            let json_data = load_cdda_json_data(cdda_path, cdda_json_path)
                .await
                .unwrap();

            info!("Successfully Loaded CDDA data");

            json_data
        });
}

#[derive(Debug, Clone, Serialize)]
pub struct AboutInfo {
    pub version: &'static str,
    pub contributors: &'static str,
    pub description: &'static str,
}

#[tauri::command]
async fn about() -> AboutInfo {
    let version = env!("CARGO_PKG_VERSION");
    let contributors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    AboutInfo {
        version,
        contributors,
        description,
    }
}

#[tauri::command]
async fn frontend_ready(
    app: AppHandle,
    editor_data: State<'_, Mutex<ProgramData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    tilesheet: State<'_, Mutex<Option<LegacyTilesheet>>>,
) -> Result<(), ()> {
    let mut editor_data_lock = editor_data.lock().await;
    let mut json_data_lock = json_data.lock().await;
    let mut tilesheet_lock = tilesheet.lock().await;

    match json_data_lock.deref() {
        None => match &editor_data_lock.config.cdda_path {
            None => {
                info!("No CDDA path set, skipping loading CDDA Json Data");
            },
            Some(cdda_path) => {
                info!("trying to load CDDA Json Data");
                match load_cdda_json_data(
                    cdda_path,
                    &editor_data_lock.config.json_data_path,
                )
                .await
                {
                    Ok(cdda_json_data) => {
                        json_data_lock.replace(cdda_json_data);
                    },
                    Err(e) => {
                        warn!("Failed to load editor data {}", e);
                    },
                };
            },
        },
        _ => {},
    };

    match json_data_lock.deref() {
        None => {},
        Some(json_data) => {
            for (name, project) in editor_data_lock.loaded_projects.iter_mut() {
                info!("Loading Project {}", name);

                match &mut project.project_type {
                    ProjectType::MapEditor(map_editor) => {
                        info!("Opening Map Editor");

                        for (_, maps) in map_editor.maps.iter_mut() {
                            match maps.calculate_parameters(&json_data.palettes)
                            {
                                Ok(_) => {},
                                Err(e) => {
                                    warn!(
                                        "Failed to calculate parameters: {}",
                                        e
                                    );
                                    continue;
                                },
                            }
                        }

                        app.emit(
                            events::TAB_CREATED,
                            Tab {
                                name: project.name.clone(),
                                tab_type: TabType::MapEditor,
                            },
                        )
                        .unwrap();
                    },
                    ProjectType::MapViewer(map_viewer) => {
                        info!("Opening Live viewer");

                        let collection =
                            match get_map_data_collection_from_map_viewer(
                                &map_viewer,
                            )
                            .await
                            {
                                Ok(mut map_data_collection) => {
                                    for (_, maps) in
                                        map_data_collection.iter_mut()
                                    {
                                        match maps.calculate_parameters(
                                            &json_data.palettes,
                                        ) {
                                            Ok(_) => {},
                                            Err(e) => {
                                                warn!(
                                                    "Failed to calculate parameters: {}",
                                                    e
                                                );
                                                continue;
                                            },
                                        }
                                    }
                                    map_data_collection
                                },
                                Err(e) => {
                                    warn!("Failed to load map data {}", e);
                                    continue;
                                },
                            };

                        map_viewer.maps = collection;

                        app.emit(
                            events::TAB_CREATED,
                            Tab {
                                name: project.name.clone(),
                                tab_type: TabType::LiveViewer,
                            },
                        )
                        .unwrap()
                    },
                }
            }
        },
    }

    info!("Sent initial editor data change");
    app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .unwrap();

    info!("Loading tilesheet");
    let tilesheet = legacy_tileset::load_tilesheet(&editor_data_lock)
        .await
        .map_err(|e| {})?;
    *tilesheet_lock = tilesheet;

    app.emit(events::TILESET_CHANGED, ()).unwrap();

    Ok(())
}

#[tauri::command]
async fn close_app(app: AppHandle) {
    let windows = app.webview_windows();

    for (_, window) in windows {
        window.close().unwrap();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> () {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Warn)
                .targets(vec![Target::new(TargetKind::Stdout)])
                .build(),
        )
        .setup(|app| {
            info!("Loading Editor data config");
            let editor_data = block_on(io::get_saved_editor_data())?;

            info!("Getting fallback tilesheet");
            let fallback_tilesheet = get_fallback_tilesheet();

            app.manage(Arc::new(fallback_tilesheet));
            app.manage(Mutex::new(editor_data));
            app.manage::<Mutex<Option<DeserializedCDDAJsonData>>>(Mutex::new(
                None,
            ));
            app.manage::<Mutex<Option<LegacyTilesheet>>>(Mutex::new(None));
            app.manage::<Mutex<Option<JoinHandle<()>>>>(Mutex::new(None));
            app.manage::<Mutex<Option<HashMap<ZLevel, MappedCDDAIdContainer>>>>(Mutex::new(None));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            download_spritesheet,
            get_project_cell_data,
            get_info_of_current_tileset,
            get_current_project_data,
            get_editor_data,
            cdda_installation_directory_picked,
            tileset_picked,
            save_program_data,
            frontend_ready,
            open_project,
            close_project,
            create_viewer,
            get_sprites,
            reload_project,
            new_single_mapgen_viewer,
            new_special_mapgen_viewer,
            new_nested_mapgen_viewer,
            get_calculated_parameters,
            open_recent_project,
            about,
            close_app,
            new_map_editor
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
