mod editor_data;
mod legacy_tileset;
mod map_data;
mod util;

use crate::editor_data::handlers::{
    cdda_installation_directory_picked, get_editor_data, save_editor_data, tileset_picked,
};
use crate::editor_data::tab::handlers::{close_tab, create_tab};
use crate::editor_data::{EditorConfig, EditorData};
use crate::legacy_tileset::handlers::{download_spritesheet, get_info_of_current_tileset};
use crate::legacy_tileset::tile_config::reader::TileConfigReader;
use crate::legacy_tileset::tile_config::{AdditionalTile, AdditionalTileId, Spritesheet, Tile, TileConfig};
use crate::map_data::handlers::{close_map, create_map, get_current_map_data};
use crate::map_data::handlers::{open_map, place};
use crate::map_data::{MapData, MapDataContainer};
use crate::util::{MeabyVec, MeabyWeighted, Weighted};
use anyhow::anyhow;
use directories::ProjectDirs;
use image::GenericImageView;
use log::{error, info, warn, LevelFilter};
use serde::{Deserialize, Serialize};
use std::arch::x86_64::_mm256_insert_epi8;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_log::{Target, TargetKind};

pub type FinalIds = Option<Vec<Weighted<Vec<u32>>>>;

#[derive(Debug)]
pub struct ForeBackIds {
    fg: FinalIds,
    bg: FinalIds,
}

impl ForeBackIds {
    pub fn new(fg: FinalIds, bg: FinalIds) -> Self {
        Self { fg, bg }
    }
}

#[derive(Debug)]
pub enum Sprite {
    Single {
        ids: ForeBackIds,
    },
    Open {
        ids: ForeBackIds,
        open: ForeBackIds,
    },
    Broken {
        ids: ForeBackIds,
        broken: ForeBackIds,
    },
    Explosion {
        ids: ForeBackIds,
        center: ForeBackIds,
        edge: ForeBackIds,
        corner: ForeBackIds,
    },
    Multitile {
        ids: ForeBackIds,

        edge: Option<ForeBackIds>,
        corner: Option<ForeBackIds>,
        center: Option<ForeBackIds>,
        t_connection: Option<ForeBackIds>,
        end_piece: Option<ForeBackIds>,
        unconnected: Option<ForeBackIds>,
    },
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum ItemID {
    Terrain(String),
    Furniture(String),
    VehiclePart(String),
    Monster(String),
    Overlay(String),
    Art(String),
    Field(String),
    Bionic(String),
    OverlayEffect(String),
    OverlayMutation(String),
    OverlayMutationActive(String),
    OverlayWorn(String),
    OverlayWielded(String),
    Corpse(String),
    Explosion(String),
    // TODO: Verify if this is actually correct
    Unknown(String),
}

impl From<String> for ItemID {
    fn from(value: String) -> Self {
        let (left, _) = match value.split_once("_") {
            None => return ItemID::Unknown(value),
            Some((left, right)) => (left, right)
        };

        // TODO: Handle _male and _female
        let item_id = match left {
            "t" => ItemID::Terrain(value),
            "f" => ItemID::Furniture(value),
            "vp" => ItemID::VehiclePart(value),
            "overlay" => ItemID::Overlay(value),
            "mon" => ItemID::Monster(value),
            "bio" => ItemID::Bionic(value),
            "art" => ItemID::Art(value),
            "fd" => ItemID::Field(value),
            _ => ItemID::Unknown(value),
        };

        item_id
    }
}

fn to_weighted_vec<T>(indices: Option<MeabyVec<MeabyWeighted<MeabyVec<T>>>>) -> Option<Vec<Weighted<Vec<T>>>> {
    indices.map(|fg| fg.map(|mw| {
        let weighted = mw.weighted();
        let weight = weighted.weight;
        let vec = weighted.sprite.vec();
        Weighted::new(vec, weight)
    }))
}

fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, anyhow::Error> {
    let mut additional_tile_ids = HashMap::new();

    for additional_tile in additional_tiles {
        let fg = to_weighted_vec(additional_tile.fg.clone());
        let bg = to_weighted_vec(additional_tile.bg.clone());

        additional_tile_ids.insert(
            additional_tile.id.clone(),
            ForeBackIds::new(fg, bg),
        );
    }

    let fg = to_weighted_vec(tile.fg.clone());
    let bg = to_weighted_vec(tile.bg.clone());

    match additional_tile_ids.remove(&AdditionalTileId::Broken) {
        None => {}
        Some(ids) => {
            return Ok(Sprite::Broken {
                ids: ForeBackIds::new(fg, bg),
                broken: ids,
            })
        }
    }

    match additional_tile_ids.remove(&AdditionalTileId::Open) {
        None => {}
        Some(ids) => {
            return Ok(Sprite::Open {
                ids: ForeBackIds::new(fg, bg),
                open: ids,
            })
        }
    }

    Ok(Sprite::Multitile {
        ids: ForeBackIds::new(fg, bg),
        center: additional_tile_ids.remove(&AdditionalTileId::Center),
        corner: additional_tile_ids.remove(&AdditionalTileId::Corner),
        edge: additional_tile_ids.remove(&AdditionalTileId::Edge),
        t_connection: additional_tile_ids.remove(&AdditionalTileId::TConnection),
        unconnected: additional_tile_ids.remove(&AdditionalTileId::Unconnected),
        end_piece: additional_tile_ids.remove(&AdditionalTileId::EndPiece),
    })
}

fn get_id_map_from_config(config: TileConfig) {
    let mut id_map = HashMap::new();

    let mut normal_spritesheets = vec![];
    for spritesheet in config.spritesheets.iter() {
        match spritesheet {
            Spritesheet::Normal(n) => normal_spritesheets.push(n),
            Spritesheet::Fallback(_) => {}
        }
    }

    for spritesheet in normal_spritesheets {
        for tile in spritesheet.tiles.iter() {
            let is_multitile = tile.multitile
                .unwrap_or_else(|| false) && tile.additional_tiles.is_some();

            if !is_multitile {
                let fg = to_weighted_vec(tile.fg.clone());
                let bg = to_weighted_vec(tile.bg.clone());

                tile.id.for_each(|id| {
                    id_map.insert(
                        ItemID::from(id.clone()),
                        Sprite::Single {
                            ids: ForeBackIds::new(fg.clone(), bg.clone()),
                        },
                    );
                });
            }

            if is_multitile {
                let additional_tiles = match &tile.additional_tiles {
                    None => unreachable!(),
                    Some(t) => t
                };

                tile.id.for_each(|id| {
                    id_map.insert(
                        ItemID::from(id.clone()),
                        get_multitile_sprite_from_additional_tiles(tile, additional_tiles).unwrap(),
                    );
                });
            }
        }
    }
}

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

    info!("Sent initial editor data change");
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

            let tile_config_reader = TileConfigReader {
                path: r"C:\DEV\SelfDEV\Rust\CDDA-Map-Editor-2\src-tauri\MSX++UnDeadPeopleEdition\tile_config.json".into(),
            };

            let config = tile_config_reader.read().unwrap();
            get_id_map_from_config(config);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            download_spritesheet,
            get_info_of_current_tileset,
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
