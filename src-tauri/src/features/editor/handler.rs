use crate::features::editor::data::ZLevels;
use crate::features::editor::{MapEditor, MapSize};
use crate::features::program_data::{
    MapDataCollection, ProgramData, Project, ProjectType, SavedProject, Tab,
    TabType,
};
use crate::util::get_size;
use crate::{events, impl_serialize_for_error};
use glam::{IVec2, UVec2};
use log::info;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum NewMapEditorError {
    #[error(transparent)]
    TauriError(#[from] tauri::Error),
}

impl_serialize_for_error!(NewMapEditorError);

#[tauri::command(rename_all = "camelCase")]
pub async fn new_map_editor(
    app: AppHandle,
    program_data: State<'_, Mutex<ProgramData>>,
    project_name: String,
    map_size: MapSize,
    z_levels: ZLevels,
    path: PathBuf,
) -> Result<(), NewMapEditorError> {
    info!("Creating new map editor");

    let mut map_collection = HashMap::new();
    for z in z_levels.value().0..=z_levels.value().1 {
        let collection = MapDataCollection::new(map_size.clone());
        map_collection.insert(z, collection);
    }

    let map_editor = MapEditor {
        maps: map_collection,
        size: map_size.value(),
    };

    let new_project =
        Project::new(project_name.clone(), ProjectType::MapEditor(map_editor));

    let mut editor_data_lock = program_data.lock().await;
    editor_data_lock.create_and_open_project(new_project, path);

    app.emit(
        events::TAB_CREATED,
        Tab {
            name: project_name.clone(),
            tab_type: TabType::MapEditor,
        },
    )?;

    Ok(())
}
