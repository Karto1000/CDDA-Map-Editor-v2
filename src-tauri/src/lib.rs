mod cdda_data;
mod editor_data;
mod map;
mod tileset;
mod util;

use crate::cdda_data::io::{CDDADataLoader, DeserializedCDDAJsonData};
use crate::editor_data::handlers::{
    cdda_installation_directory_picked, get_editor_data, save_editor_data, tileset_picked,
};
use crate::editor_data::tab::handlers::{close_tab, create_tab};
use crate::editor_data::EditorData;
use crate::map::handlers::{
    close_project, create_project, get_current_project_data, save_current_project,
};
use crate::map::handlers::{get_project_cell_data, open_project};
use crate::map::ProjectContainer;
use crate::tileset::handlers::{download_spritesheet, get_info_of_current_tileset};
use crate::tileset::io::{TileConfigLoader, TilesheetLoader};
use crate::tileset::legacy_tileset::MappedSprite;
use crate::tileset::TilesheetKind;
use crate::util::Load;
use anyhow::{anyhow, Error};
use directories::ProjectDirs;
use glam::IVec3;
use lazy_static::lazy_static;
use log::{error, info, warn, LevelFilter};
use map::importing::MapDataImporter;
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use tauri::async_runtime::Mutex;
use tauri::{App, AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};

mod events {
    pub const EDITOR_DATA_CHANGED: &'static str = "editor_data_changed";
    pub const CDDA_DATA: &'static str = "cdda_data";
}

pub static RANDOM_SEED: u64 = 1;

lazy_static! {
    pub static ref RANDOM: Arc<RwLock<StdRng>> =
        Arc::new(RwLock::new(StdRng::seed_from_u64(RANDOM_SEED)));
}

#[tauri::command]
async fn frontend_ready(
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    map_data: State<'_, Mutex<ProjectContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    tilesheet: State<'_, Mutex<Option<TilesheetKind>>>,
) -> Result<(), ()> {
    let editor_data_lock = editor_data.lock().await;
    let mut map_data_lock = map_data.lock().await;
    let mut json_data_lock = json_data.lock().await;
    let mut tilesheet_lock = tilesheet.lock().await;

    for tab in &editor_data_lock.tabs {
        info!("Opened Tab {}", &tab.name);
        app.emit("tab_created", tab).expect("Emit to not fail");
    }

    info!("Sent initial editor data change");
    app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .expect("Emit to not fail");

    info!("Loading tilesheet");
    let tilesheet = load_tilesheet(&editor_data_lock).await.map_err(|e| {})?;
    *tilesheet_lock = tilesheet;

    info!("trying to load CDDA Json Data");
    match load_cdda_json_data(&editor_data_lock).await {
        Ok(cdda_json_data) => {
            info!("Loading testing map data");

            let mut importer = MapDataImporter {
                path: r"C:\CDDA\testing\data\json\mapgen\mansion.json".into(),
                om_terrain: "mansion_t2d".into(),
            };
            let mut loaded = importer.load().await.unwrap();
            loaded
                .maps
                .get_mut(&0)
                .unwrap()
                .calculate_parameters(&cdda_json_data.palettes);

            map_data_lock.data.push(loaded.clone());
            map_data_lock.current_project = Some(0);

            info!("Sending cdda data");
            app.emit(events::CDDA_DATA, loaded).unwrap();

            json_data_lock.replace(cdda_json_data);
        }
        Err(e) => {
            warn!("Failed to load editor data");
        }
    };

    Ok(())
}

fn get_saved_editor_data(app: &mut App) -> Result<EditorData, Error> {
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
            info!(
                "Got Path for CDDA-Map-Editor config directory at {:?}",
                local_dir
            );
            local_dir.to_path_buf()
        }
    };

    if !fs::exists(&directory_path).expect("IO Error to not occur") {
        info!(
            "Created CDDA-Map-Editor config directory at {:?}",
            directory_path
        );
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

                    let full_message = format!(
                        r#"
                               An error occurred while reading the config.json file at {:?}.
                               This is likely due to the file containing unexpected or invalid data.

                               To fix this, you can regenerate the file. However, this would delete
                               your current configuration and reset it to the default state.

                               Do you want to continue?
                            "#,
                        config_file_path
                    );

                    let answer = app
                        .dialog()
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

                            let serialized = serde_json::to_string_pretty(&default_editor_data)
                                .expect("Serialization to not fail");
                            fs::write(&config_file_path, serialized)
                                .expect("Directory path to config to have been created");
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

            let serialized = serde_json::to_string_pretty(&default_editor_data)
                .expect("Serialization to not fail");
            fs::write(&config_file_path, serialized)
                .expect("Directory path to config to have been created");
            default_editor_data
        }
    };

    Ok(config)
}

fn get_map_data(editor_data: &EditorData) -> Result<ProjectContainer, anyhow::Error> {
    let map_data = ProjectContainer::default();

    // map_data.data.push(loaded);

    // for tab in editor_data.tabs.iter() {
    //     match &tab.tab_type {
    //         TabType::MapEditor(state) => match state {
    //             MapDataState::Saved { path } => {
    //                 let loader = MapDataLoader { path: path.clone() };
    //
    //                 info!("Loading map data from {:?}", path);
    //                 map_data.data.push(loader.load()?)
    //             }
    //             _ => {}
    //         },
    //         TabType::LiveViewer => todo!(),
    //         _ => {}
    //     }
    // }

    Ok(map_data)
}

async fn load_tilesheet(editor_data: &EditorData) -> Result<Option<TilesheetKind>, Error> {
    let tileset = match &editor_data.config.selected_tileset {
        None => return Ok(None),
        Some(t) => t.clone(),
    };

    let cdda_path = match &editor_data.config.cdda_path {
        None => return Ok(None),
        Some(p) => p.clone(),
    };

    let config_path = cdda_path
        .join("gfx")
        .join(&tileset)
        .join("tile_config.json");

    let mut tile_config_loader = TileConfigLoader::new(config_path);
    let config = tile_config_loader.load().await?;

    let mut tilesheet_loader = TilesheetLoader::new(config);
    let tilesheet = tilesheet_loader.load().await?;

    Ok(Some(TilesheetKind::Legacy(tilesheet)))
}

pub async fn load_cdda_json_data(
    editor_data: &EditorData,
) -> Result<DeserializedCDDAJsonData, anyhow::Error> {
    let cdda_path = editor_data
        .config
        .cdda_path
        .clone()
        .ok_or(anyhow!("No CDDA Path supplied"))?
        .clone();

    let mut data_loader = CDDADataLoader {
        json_path: cdda_path.join(&editor_data.config.json_data_path),
    };

    data_loader.load().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> () {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(LevelFilter::Info)
                .targets(vec![
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::Stdout),
                ])
                .build(),
        )
        .setup(|app| {
            info!("Loading Editor data config");
            let editor_data = get_saved_editor_data(app)?;

            info!("Loading map data");
            let map_data = get_map_data(&editor_data)?;

            app.manage(Mutex::new(editor_data));
            app.manage(Mutex::new(map_data));
            app.manage::<Mutex<HashMap<IVec3, MappedSprite>>>(Mutex::new(HashMap::new()));
            app.manage::<Mutex<Option<DeserializedCDDAJsonData>>>(Mutex::new(None));
            app.manage::<Mutex<Option<TilesheetKind>>>(Mutex::new(None));

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
            save_editor_data,
            create_tab,
            close_tab,
            frontend_ready,
            create_project,
            open_project,
            close_project,
            save_current_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
