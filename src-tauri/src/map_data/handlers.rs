use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::palettes::Palettes;
use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::MapDataState::Saved;
use crate::editor_data::tab::{MapDataState, TabType};
use crate::editor_data::{EditorData, EditorDataSaver};
use crate::legacy_tileset::{FinalIds, GetRandom, Sprite, Tilesheet};
use crate::map_data::io::MapDataSaver;
use crate::map_data::{Cell, MapData, MapDataContainer};
use crate::util::{CDDAIdentifier, GetIdentifier, JSONSerializableUVec2, MeabyParam, Save};
use glam::UVec2;
use image::imageops::tile;
use log::{error, info, warn};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, Index};
use std::path::PathBuf;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::MutexGuard;

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
    positions: Vec<JSONSerializableUVec2>,
    indexes: Vec<u32>,
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

    let sprite = tilesheet
        .id_map
        .get(&CDDAIdentifier("t_grass".into()))
        .unwrap();
    let fg = match sprite {
        Sprite::Single { .. } => unreachable!(),
        Sprite::Open { .. } => unreachable!(),
        Sprite::Broken { .. } => unreachable!(),
        Sprite::Explosion { .. } => unreachable!(),
        Sprite::Multitile { ids, .. } => ids
            .fg
            .clone()
            .unwrap()
            .get(0)
            .unwrap()
            .sprite
            .get(0)
            .unwrap()
            .clone(),
    };

    app.emit(
        "place_sprite",
        PlaceSpriteEvent {
            position: command.position.clone(),
            index: fg,
        },
    )
    .unwrap();

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

#[tauri::command]
pub async fn open_map(
    index: usize,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<Tilesheet>>>,
    map_data_container: State<'_, Mutex<MapDataContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
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

    app.emit("opened_map", index).expect("Function to not fail");

    let mut indexes = vec![];
    let mut positions = vec![];

    let fill_sprite = match &map_data.fill {
        None => None,
        Some(id) => {
            let id = id.get_identifier(&map_data.calculated_parameters);

            match tilesheet.id_map.get(&id) {
                None => None,
                Some(s) => Some(s),
            }
        }
    };

    map_data.cells.iter().for_each(|(p, char)| {
        match fill_sprite {
            None => {}
            Some(fill_sprite) => {
                if char.character == ' ' {
                    let tilesheet_id = match fill_sprite {
                        Sprite::Single { ids } => match &ids.fg {
                            None => 0,
                            Some(v) => v.get_random().get(0).unwrap().clone(),
                        },
                        Sprite::Open { .. } => 0,
                        Sprite::Broken { .. } => 0,
                        Sprite::Explosion { .. } => 0,
                        Sprite::Multitile { .. } => 0,
                    };

                    positions.push(JSONSerializableUVec2(p.clone()));
                    indexes.push(tilesheet_id);
                    return;
                }
            }
        };

        let identifiers = map_data.get_identifiers(&char.character, &json_data.palettes);

        if identifiers.terrain.is_none() && identifiers.furniture.is_none() {
            warn!("No sprites found for char {:?}", char);
            return;
        }

        for o_id in [identifiers.terrain, identifiers.furniture] {
            let id = match o_id {
                None => continue,
                Some(id) => id,
            };

            let sprite = match tilesheet.id_map.get(&id) {
                None => {
                    warn!("Could not find {} in tilesheet ids", id);
                    positions.push(JSONSerializableUVec2(p.clone()));
                    indexes.push(0);
                    return;
                }
                Some(s) => s,
            };

            let tilesheet_id = match sprite {
                Sprite::Single { ids } => match &ids.fg {
                    None => 0,
                    Some(v) => v.get_random().get(0).unwrap().clone(),
                },
                Sprite::Open { .. } => 0,
                Sprite::Broken { .. } => 0,
                Sprite::Explosion { .. } => 0,
                Sprite::Multitile { .. } => 0,
            };

            positions.push(JSONSerializableUVec2(p.clone()));
            indexes.push(tilesheet_id);
        }
    });

    assert_eq!(indexes.len(), positions.len());

    app.emit("place_sprites", PlaceSpritesEvent { positions, indexes })
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
