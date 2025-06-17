use crate::features::editor::data::ZLevels;
use crate::features::editor::MapSize;
use crate::features::program_data::{
    EditorSaveState, MapDataCollection, ProgramData, Project, ProjectType,
    SavedProject, Tab, TabType,
};
use crate::{events, impl_serialize_for_error};
use glam::{IVec2, UVec2};
use log::info;
use serde::{Deserialize, Deserializer, Serialize};
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
    editor_data: State<'_, Mutex<ProgramData>>,
    project_name: String,
    map_size: MapSize,
    z_levels: ZLevels,
    path: PathBuf,
) -> Result<(), NewMapEditorError> {
    info!("Creating new map editor");

    let mut new_project = Project::new(
        project_name.clone(),
        map_size.value(),
        ProjectType::MapEditor(EditorSaveState::Saved { path }),
    );

    for z in z_levels.value().0..=z_levels.value().1 {
        let collection = MapDataCollection::new(map_size.clone());
        new_project.maps.insert(z, collection);
    }

    let mut editor_data_lock = editor_data.lock().await;

    editor_data_lock
        .loaded_projects
        .insert(project_name.clone(), new_project);
    editor_data_lock.opened_project = Some(project_name.clone());

    let saved_project = SavedProject {
        path: editor_data_lock.config.config_path.clone(),
    };

    editor_data_lock
        .openable_projects
        .insert(project_name.clone(), saved_project.clone());

    editor_data_lock
        .recent_projects
        .insert(project_name.clone(), saved_project);

    app.emit(
        events::TAB_CREATED,
        Tab {
            name: project_name.clone(),
            tab_type: TabType::MapEditor,
        },
    )?;

    Ok(())
}
