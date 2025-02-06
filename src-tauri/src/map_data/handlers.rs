use crate::map_data::{Cell, Identifier, MapData};
use crate::util::JSONSerializableUVec2;
use serde::{Deserialize, Deserializer, Serialize};
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn get_map_data(map_data: State<'_, Mutex<MapData>>) -> Result<MapData, ()> {
    Ok(map_data.lock().await.clone())
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

#[tauri::command]
pub async fn place(
    app: AppHandle,
    map_data: State<'_, Mutex<MapData>>,
    command: PlaceCommand,
) -> Result<(), ()> {
    let mut lock = map_data.lock().await;

    lock.cells.insert(
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
