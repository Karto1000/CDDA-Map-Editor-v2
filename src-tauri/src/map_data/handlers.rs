use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::TabType;
use crate::editor_data::EditorData;
use crate::legacy_tileset::{Sprite, Tilesheet};
use crate::map_data::{Cell, Identifier, MapData, MapDataContainer};
use crate::util::JSONSerializableUVec2;
use glam::UVec2;
use image::imageops::tile;
use log::info;
use serde::{Deserialize, Deserializer, Serialize};
use std::ops::Index;
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
    info!("Placing {} at {:?}", command.character, command.position);

    let mut lock = map_data.lock().await;
    let data = get_current_map_mut(&mut lock)?;

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(PlaceError::NoTilesheet),
        Some(t) => t
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

    let sprite = tilesheet.id_map.get("t_grass").unwrap();
    let fg = match sprite {
        Sprite::Single { .. } => unreachable!(),
        Sprite::Open { .. } => unreachable!(),
        Sprite::Broken { .. } => unreachable!(),
        Sprite::Explosion { .. } => unreachable!(),
        Sprite::Multitile { ids, .. } => {
            ids.fg.clone().unwrap().get(0).unwrap().sprite.get(0).unwrap().clone()
        }
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

    lock.data.push(MapData {
        name: data.name.clone(),
        cells: Default::default(),
    });

    create_tab(data.name, TabType::MapEditor, app, editor_data)
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
) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;
    lock.current_map = Some(index);

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(()),
        Some(t) => t
    };

    app.emit("opened_map", index).expect("Function to not fail");

    let map_data = lock.data.get(index).unwrap();

    let sprite = tilesheet.id_map.get("t_grass").unwrap();
    let fg = match sprite {
        Sprite::Single { .. } => unreachable!(),
        Sprite::Open { .. } => unreachable!(),
        Sprite::Broken { .. } => unreachable!(),
        Sprite::Explosion { .. } => unreachable!(),
        Sprite::Multitile { ids, .. } => {
            ids.fg.clone().unwrap().get(0).unwrap().sprite.get(0).unwrap().clone()
        }
    };

    let positions: Vec<JSONSerializableUVec2> = map_data.cells
        .iter()
        .map(|(pos, _)| { JSONSerializableUVec2(pos.clone()) })
        .collect();

    let indexes: Vec<u32> = vec![fg; positions.len()];

    app.emit(
        "place_sprites",
        PlaceSpritesEvent {
            positions,
            indexes,
        },
    ).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn close_map(map_data_container: State<'_, Mutex<MapDataContainer>>) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;
    lock.current_map = None;
    Ok(())
}
