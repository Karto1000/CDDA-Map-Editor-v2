mod cdda_data;
mod editor_data;
mod events;
mod map;
mod tab;
mod tileset;
mod util;

use crate::cdda_data::io::{CDDADataLoader, DeserializedCDDAJsonData};
use crate::cdda_data::map_data::NeighborDirection;
use crate::editor_data::handlers::{
    cdda_installation_directory_picked, get_editor_data, save_editor_data, tileset_picked,
};
use crate::editor_data::{
    get_map_data_collection_live_viewer_data, EditorData, MapDataCollection, Project, ProjectType,
};
use crate::map::handlers::{
    close_project, get_current_project_data, get_project_cell_data, open_project,
};
use crate::map::importing::{NestedMapDataImporter, SingleMapDataImporter};
use crate::map::viewer::open_viewer;
use crate::tab::{Tab, TabType};
use crate::tileset::handlers::{download_spritesheet, get_info_of_current_tileset};
use crate::tileset::io::{TileConfigLoader, TilesheetLoader};
use crate::tileset::legacy_tileset::MappedCDDAIds;
use crate::tileset::TilesheetKind;
use crate::util::Load;
use anyhow::{anyhow, Error};
use async_once::AsyncOnce;
use directories::ProjectDirs;
use glam::{IVec3, UVec2};
use lazy_static::lazy_static;
use log::{error, info, warn, LevelFilter};
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tauri::async_runtime::Mutex;
use tauri::{App, AppHandle, Emitter, Manager, State, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};

pub static RANDOM_SEED: u64 = 1;

lazy_static! {
    pub static ref RANDOM: Arc<RwLock<StdRng>> =
        Arc::new(RwLock::new(StdRng::seed_from_u64(RANDOM_SEED)));
    static ref TEST_CDDA_DATA: AsyncOnce<DeserializedCDDAJsonData> = AsyncOnce::new(async {
        dotenv::dotenv().unwrap();
        env_logger::init();

        info!("Loading CDDA data");

        let cdda_path = std::env::var("CDDA_INSTALL_PATH").expect("CDDA_INSTALL_PATH not set");
        let cdda_json_path = std::env::var("CDDA_JSON_PATH").unwrap_or("data\\json\\".to_string());

        let json_data = load_cdda_json_data(cdda_path, cdda_json_path)
            .await
            .unwrap();

        info!("Successfully Loaded CDDA data");

        json_data
    });
}

#[tauri::command]
async fn frontend_ready(
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    tilesheet: State<'_, Mutex<Option<TilesheetKind>>>,
) -> Result<(), ()> {
    let mut editor_data_lock = editor_data.lock().await;
    let mut json_data_lock = json_data.lock().await;
    let mut tilesheet_lock = tilesheet.lock().await;

    match json_data_lock.deref() {
        None => match &editor_data_lock.config.cdda_path {
            None => {
                info!("No CDDA path set, skipping loading CDDA Json Data");
            }
            Some(cdda_path) => {
                info!("trying to load CDDA Json Data");
                match load_cdda_json_data(cdda_path, &editor_data_lock.config.json_data_path).await
                {
                    Ok(cdda_json_data) => {
                        json_data_lock.replace(cdda_json_data);
                    }
                    Err(e) => {
                        warn!("Failed to load editor data {}", e);
                    }
                };
            }
        },
        _ => {}
    };

    match json_data_lock.deref() {
        None => {}
        Some(json_data) => {
            for project in editor_data_lock.projects.iter_mut() {
                info!("Loading Project {}", &project.name);

                match &project.ty {
                    ProjectType::MapEditor(me) => unimplemented!(),
                    ProjectType::LiveViewer(lvd) => {
                        info!("Opening Live viewer {:?} at {:?}", lvd.om_terrain, lvd.path);

                        let mut map_data_collection =
                            get_map_data_collection_live_viewer_data(lvd).await;
                        map_data_collection.calculate_parameters(&json_data.palettes);

                        let mut maps = HashMap::new();
                        maps.insert(0, map_data_collection);
                        project.maps = maps;

                        app.emit(
                            events::TAB_CREATED,
                            Tab {
                                name: project.name.clone(),
                                tab_type: TabType::LiveViewer,
                            },
                        )
                        .unwrap()
                    }
                }
            }
        }
    }

    info!("Sent initial editor data change");
    app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .expect("Emit to not fail");

    info!("Loading tilesheet");
    let tilesheet = load_tilesheet(&editor_data_lock).await.map_err(|e| {})?;
    *tilesheet_lock = tilesheet;

    Ok(())
}

fn get_saved_editor_data() -> Result<EditorData, Error> {
    let project_dir = ProjectDirs::from("", "", "CDDA Map Editor");

    let directory_path = match project_dir {
        None => {
            warn!("No valid project directory found, creating data folder application directory instead");
            let app_dir = match std::env::current_dir() {
                Ok(d) => d,
                Err(e) => {
                    error!("{}", e);
                    panic!()
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
                    info!("Error while reading config.json file, recreating file");

                    let mut default_editor_data = EditorData::default();
                    default_editor_data.config.config_path = directory_path.clone();

                    let serialized = serde_json::to_string_pretty(&default_editor_data)
                        .expect("Serialization to not fail");
                    fs::write(&config_file_path, serialized)
                        .expect("Directory path to config to have been created");
                    default_editor_data
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
    cdda_path: impl Into<PathBuf>,
    json_data_path: impl Into<PathBuf>,
) -> Result<DeserializedCDDAJsonData, anyhow::Error> {
    let mut data_loader = CDDADataLoader {
        json_path: cdda_path.into().join(json_data_path.into()),
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
                .level(LevelFilter::Warn)
                .targets(vec![
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::Stdout),
                ])
                .build(),
        )
        .setup(|app| {
            info!("Loading Editor data config");
            let editor_data = get_saved_editor_data()?;

            app.manage(Mutex::new(editor_data));
            app.manage::<Mutex<HashMap<IVec3, MappedCDDAIds>>>(Mutex::new(HashMap::new()));
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
            frontend_ready,
            open_project,
            close_project,
            open_viewer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
