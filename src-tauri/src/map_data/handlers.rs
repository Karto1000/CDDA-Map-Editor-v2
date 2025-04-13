use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::palettes::Palettes;
use crate::cdda_data::region_settings::RegionIdentifier;
use crate::cdda_data::TileLayer;
use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::MapDataState::Saved;
use crate::editor_data::tab::{MapDataState, TabType};
use crate::editor_data::{EditorData, EditorDataSaver};
use crate::legacy_tileset::{
    FinalIds, GetRandom, MappedSprite, Sprite, SpriteIndex, SpriteLayer, Tilesheet,
};
use crate::map_data::io::MapDataSaver;
use crate::map_data::{Cell, MapData, MapDataContainer, SPECIAL_EMPTY_CHAR};
use crate::util::{CDDAIdentifier, DistributionInner, GetIdentifier, JSONSerializableUVec2, Save};
use glam::{UVec2, Vec2};
use image::imageops::{index_colors, tile};
use log::{debug, error, info, warn};
use rand::fill;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::convert::identity;
use std::mem::discriminant;
use std::ops::{Deref, Index};
use std::path::PathBuf;
use tauri::async_runtime::{set, Mutex};
use tauri::utils::display_path;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::MutexGuard;
use tokio::task::id;

mod events {
    pub const OPENED_MAP: &'static str = "opened_map";
    pub const PLACE_SPRITES: &'static str = "place_sprites";
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum GetCurrentMapDataError {
    #[error("No map has been opened")]
    NoMapOpened,
    #[error("Invalid map index")]
    InvalidMapIndex(usize),
}

fn get_current_map<'a>(
    map_data_container: &'a MutexGuard<MapDataContainer>,
) -> Result<&'a MapData, GetCurrentMapDataError> {
    let map_index = match map_data_container.current_map {
        None => return Err(GetCurrentMapDataError::NoMapOpened),
        Some(i) => i,
    };

    let data = match map_data_container.data.get(map_index) {
        None => return Err(GetCurrentMapDataError::InvalidMapIndex(map_index)),
        Some(d) => d,
    };

    Ok(data)
}

fn get_current_map_mut<'a>(
    map_data_container: &'a mut MutexGuard<MapDataContainer>,
) -> Result<&'a mut MapData, GetCurrentMapDataError> {
    let map_index = match map_data_container.current_map {
        None => return Err(GetCurrentMapDataError::NoMapOpened),
        Some(i) => i,
    };

    let data = match map_data_container.data.get_mut(map_index) {
        None => return Err(GetCurrentMapDataError::InvalidMapIndex(map_index)),
        Some(d) => d,
    };

    Ok(data)
}

#[tauri::command]
pub async fn get_current_map_data(
    map_data: State<'_, Mutex<MapDataContainer>>,
) -> Result<MapData, GetCurrentMapDataError> {
    let lock = map_data.lock().await;
    let data = get_current_map(&lock)?;

    Ok(data.clone())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceCommand {
    position: JSONSerializableUVec2,
    character: char,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapChangEvent {
    kind: MapChangeEventKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MapChangeEventKind {
    Place(PlaceCommand),
    Delete(JSONSerializableUVec2),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceSpriteEvent {
    position: JSONSerializableUVec2,
    index: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceSpritesEvent {
    static_sprites: Vec<StaticSprite>,
    animated_sprites: Vec<AnimatedSprite>,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum PlaceError {
    #[error(transparent)]
    MapError(#[from] GetCurrentMapDataError),

    #[error("No Tilesheet selected")]
    NoTilesheet,
}

#[tauri::command]
pub async fn place(
    app: AppHandle,
    map_data: State<'_, Mutex<MapDataContainer>>,
    tilesheet: State<'_, Mutex<Option<Tilesheet>>>,
    command: PlaceCommand,
) -> Result<(), PlaceError> {
    let mut lock = map_data.lock().await;
    let data = get_current_map_mut(&mut lock)?;

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(PlaceError::NoTilesheet),
        Some(t) => t,
    };

    if data.cells.get(&command.position.0).is_some() {
        return Ok(());
    }

    data.cells.insert(
        command.position.0.clone(),
        Cell {
            character: command.character,
        },
    );

    // let sprite = tilesheet
    //     .id_map
    //     .get(&CDDAIdentifier("t_grass".into()))
    //     .unwrap();
    // let fg = match sprite {
    //     Sprite::Single { .. } => unreachable!(),
    //     Sprite::Open { .. } => unreachable!(),
    //     Sprite::Broken { .. } => unreachable!(),
    //     Sprite::Explosion { .. } => unreachable!(),
    //     Sprite::Multitile { ids, .. } => ids.fg.clone().unwrap().get(0).unwrap().sprite,
    // };
    //
    // app.emit(
    //     "place_sprite",
    //     PlaceSpriteEvent {
    //         position: command.position.clone(),
    //         index: fg,
    //     },
    // )
    // .unwrap();

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMapData {
    name: String,
    size: JSONSerializableUVec2,
}

#[tauri::command]
pub async fn create_map(
    app: AppHandle,
    data: CreateMapData,
    map_data_container: State<'_, Mutex<MapDataContainer>>,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;

    lock.data.push(MapData::new(
        data.name.clone(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    ));

    create_tab(
        data.name,
        TabType::MapEditor(MapDataState::Unsaved),
        app,
        editor_data,
    )
    .await
    .expect("Function to not fail");

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StaticSprite {
    pub position: JSONSerializableUVec2,
    pub index: u32,
    pub layer: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AnimatedSprite {
    pub position: JSONSerializableUVec2,
    pub indices: Vec<u32>,
    pub layer: u32,
}

fn get_id_from_mapped_sprites(
    mapped_sprites_lock: &MutexGuard<HashMap<UVec2, MappedSprite>>,
    cords: &UVec2,
    layer: &TileLayer,
) -> Option<CDDAIdentifier> {
    mapped_sprites_lock
        .get(cords)
        .map(|v| match layer {
            TileLayer::Terrain => v.terrain.clone(),
            TileLayer::Furniture => v.furniture.clone(),
        })
        .flatten()
}

#[tauri::command]
pub async fn open_map(
    index: usize,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<Tilesheet>>>,
    map_data_container: State<'_, Mutex<MapDataContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    mapped_sprites: State<'_, Mutex<HashMap<UVec2, MappedSprite>>>,
) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;

    let guard = json_data.lock().await;
    let json_data = match guard.deref() {
        None => return Err(()),
        Some(d) => d,
    };

    lock.current_map = Some(index);

    let map_data = match lock.data.get(index) {
        None => {
            warn!("Could not find map at index {}", index);
            return Err(());
        }
        Some(d) => d,
    };

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(()),
        Some(t) => t,
    };

    app.emit(events::OPENED_MAP, index)
        .expect("Function to not fail");

    let region_settings = json_data
        .region_settings
        .get(&CDDAIdentifier("default".into()))
        .expect("Region settings to exist");

    let fill_terrain_sprite = match &map_data.fill {
        None => None,
        Some(id) => {
            let id = id.get_identifier(&map_data.calculated_parameters);

            Some(id.as_final_id(&region_settings, &json_data.terrain, &json_data.furniture))
        }
    };

    let mut mapped_sprites_lock = mapped_sprites.lock().await;

    let mut static_sprites = vec![];
    let mut animated_sprites = vec![];

    // Store the identifiers here since they use random numbers to determine them and if we used
    // this function in the next iteration through map_data, we would get other identifiers
    let mut identifiers = HashMap::new();

    // We need to insert the mapped_sprite before we get the fg and bg of this sprite since
    // the function relies on the mapped sprite of this sprite to already exist
    map_data.cells.iter().for_each(|(p, cell)| {
        let identifier_group = map_data.get_identifiers(&cell.character, &json_data.palettes);

        let mut mapped_sprite = MappedSprite::default();

        for (layer, o_id) in [
            (TileLayer::Terrain, &identifier_group.terrain),
            (TileLayer::Furniture, &identifier_group.furniture),
        ] {
            let id = match o_id {
                None => continue,
                Some(id) => {
                    id.as_final_id(region_settings, &json_data.terrain, &json_data.furniture)
                }
            };

            match layer {
                TileLayer::Terrain => mapped_sprite.terrain = Some(id.clone()),
                TileLayer::Furniture => mapped_sprite.furniture = Some(id.clone()),
            };
        }

        mapped_sprites_lock.insert(p.clone(), mapped_sprite);
        identifiers.insert(p, identifier_group);
    });

    map_data.cells.iter().for_each(|(p, cell)| {
        let mut identifier_group = identifiers.remove(p).expect("Identifier group to exist");

        match identifier_group.terrain {
            None => {
                debug!(
                    "No terrain found for {}, trying to use fill_sprite",
                    cell.character
                );

                match &fill_terrain_sprite {
                    None => {
                        warn!("terrain was not defined for {}", cell.character);
                        todo!();
                    }
                    Some(fill_sprite) => {
                        identifier_group.terrain = Some(fill_sprite.clone());
                    }
                };
            }
            Some(_) => {}
        }

        if identifier_group.terrain.is_none() && identifier_group.furniture.is_none() {
            warn!("No sprites found for char {:?}", cell);
            return;
        }

        // Layer here is done so furniture is above terrain
        for (layer, o_id) in [
            (TileLayer::Terrain, identifier_group.terrain),
            (TileLayer::Furniture, identifier_group.furniture),
        ] {
            let id = match o_id {
                None => continue,
                Some(id) => {
                    id.as_final_id(region_settings, &json_data.terrain, &json_data.furniture)
                }
            };

            let top_cords = p + UVec2::new(0, 1);
            let top = get_id_from_mapped_sprites(&mapped_sprites_lock, &top_cords, &layer);

            let right_cords = p + UVec2::new(1, 0);
            let right = get_id_from_mapped_sprites(&mapped_sprites_lock, &right_cords, &layer);

            // We need this to protect against underflow of coordinates since they're stored as a UVec2
            let bottom = match p.y > 0 {
                true => {
                    let bottom_cords = p - UVec2::new(0, 1);
                    get_id_from_mapped_sprites(&mapped_sprites_lock, &bottom_cords, &layer)
                }
                false => None,
            };

            let left = match p.x > 0 {
                true => {
                    let left_cords = p - UVec2::new(1, 0);
                    get_id_from_mapped_sprites(&mapped_sprites_lock, &left_cords, &layer)
                }
                false => None,
            };

            let sprite = tilesheet.get_sprite(&id, &json_data.terrain, &json_data.furniture);

            match sprite.get_fg_id(&id, json_data, &layer, top, right, bottom, left) {
                None => {}
                // TODO: Mapped Sprite
                Some(id) => match sprite.is_animated() {
                    true => {
                        let display_sprite = AnimatedSprite {
                            position: JSONSerializableUVec2(p.clone()),
                            layer: (layer.clone() as u32) * 2 + SpriteLayer::Fg as u32,
                            indices: id.into_vec(),
                        };

                        animated_sprites.push(display_sprite)
                    }
                    false => {
                        let display_sprite = StaticSprite {
                            position: JSONSerializableUVec2(p.clone()),
                            layer: (layer.clone() as u32) * 2 + SpriteLayer::Bg as u32,
                            index: id.into_single().unwrap(),
                        };

                        static_sprites.push(display_sprite);
                    }
                },
            }

            match sprite.get_bg_id() {
                None => {}
                Some(id) => match sprite.is_animated() {
                    true => {
                        let display_sprite = AnimatedSprite {
                            position: JSONSerializableUVec2(p.clone()),
                            layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                            indices: id.into_vec(),
                        };

                        animated_sprites.push(display_sprite);
                    }
                    false => {
                        let display_sprite = StaticSprite {
                            position: JSONSerializableUVec2(p.clone()),
                            layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                            index: id.into_single().unwrap(),
                        };

                        static_sprites.push(display_sprite);
                    }
                },
            }
        }
    });

    app.emit(
        events::PLACE_SPRITES,
        PlaceSpritesEvent {
            static_sprites,
            animated_sprites,
        },
    )
    .unwrap();

    Ok(())
}

#[tauri::command]
pub async fn close_map(map_data_container: State<'_, Mutex<MapDataContainer>>) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;
    lock.current_map = None;
    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum SaveError {
    #[error(transparent)]
    MapError(#[from] GetCurrentMapDataError),

    #[error("Failed to save map data")]
    SaveError,
}

#[tauri::command]
pub async fn save_current_map(
    map_data_container: State<'_, Mutex<MapDataContainer>>,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), SaveError> {
    let lock = map_data_container.lock().await;
    let map_data = get_current_map(&lock)?;

    let save_path = r"C:\Users\Kartoffelbauer\Downloads\test";
    let saver = MapDataSaver {
        path: save_path.into(),
    };

    saver.save(&map_data).map_err(|e| {
        error!("{}", e);
        SaveError::SaveError
    })?;

    let mut editor_data = editor_data.lock().await;
    let new_tab_type = TabType::MapEditor(Saved {
        path: PathBuf::from(save_path).join(&map_data.name),
    });
    editor_data
        .tabs
        .get_mut(lock.current_map.unwrap())
        .unwrap()
        .tab_type = new_tab_type;

    let editor_data_saver = EditorDataSaver {
        path: editor_data.config.config_path.clone(),
    };

    editor_data_saver
        .save(&editor_data)
        .expect("Saving to not fail");

    Ok(())
}
