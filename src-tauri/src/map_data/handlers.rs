use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::TabType;
use crate::editor_data::EditorData;
use crate::map_data::{Cell, Identifier, MapData, MapDataContainer};
use crate::util::JSONSerializableUVec2;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::MutexGuard;

#[derive(Debug, thiserror::Error, Serialize)]
pub enum GetCurrentMapDataError {
    #[error("No map has been opened")]
    NoMapOpened,
    #[error("Invalid map index")]
    InvalidMapIndex,
}

fn get_current_map<'a>(
    map_data_container: &'a MutexGuard<MapDataContainer>,
) -> Result<&'a MapData, GetCurrentMapDataError> {
    let map_index = match map_data_container.current_map {
        None => return Err(GetCurrentMapDataError::NoMapOpened),
        Some(i) => i,
    };

    let data = match map_data_container.data.get(map_index) {
        None => return Err(GetCurrentMapDataError::InvalidMapIndex),
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
        None => return Err(GetCurrentMapDataError::InvalidMapIndex),
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
pub struct PlaceTerrainEvent {
    position: JSONSerializableUVec2,
    identifier: Identifier,
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum PlaceError {
    #[error(transparent)]
    MapError(#[from] GetCurrentMapDataError),
}

#[tauri::command]
pub async fn place(
    app: AppHandle,
    map_data: State<'_, Mutex<MapDataContainer>>,
    command: PlaceCommand,
) -> Result<(), PlaceError> {
    let mut lock = map_data.lock().await;
    let data = get_current_map_mut(&mut lock)?;

    data.cells.insert(
        command.position.0.clone(),
        Cell {
            character: command.character,
        },
    );

    app.emit(
        "place_terrain",
        PlaceTerrainEvent {
            position: command.position.clone(),
            identifier: "t_grass".to_string(),
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
    map_data_container: State<'_, Mutex<MapDataContainer>>,
) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;
    lock.current_map = Some(index);
    Ok(())
}

#[tauri::command]
pub async fn close_map(map_data_container: State<'_, Mutex<MapDataContainer>>) -> Result<(), ()> {
    let mut lock = map_data_container.lock().await;
    lock.current_map = None;
    Ok(())
}
