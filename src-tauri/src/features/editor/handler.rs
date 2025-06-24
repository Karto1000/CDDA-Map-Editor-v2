use crate::features::editor::data::ZLevels;
use crate::features::editor::{MapEditor, MapSize};
use crate::features::program_data::io::{ProgramDataSaver, ProjectSaver};
use crate::features::program_data::{
    LoadedProjects, MapDataCollection, ProgramData, Project, ProjectType,
    SavedProject, Tab, TabType,
};
use crate::util::{
    get_current_project_mut, get_size, GetCurrentProjectError, Save, SaveError,
};
use crate::{events, impl_serialize_for_error, InvalidProjectType};
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

    #[error(transparent)]
    SaveError(#[from] SaveError),
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
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), NewMapEditorError> {
    info!("Creating new map editor");

    let mut program_data_lock = program_data.lock().await;

    let mut map_collection = HashMap::new();
    for z in z_levels.value().0..=z_levels.value().1 {
        let collection =
            MapDataCollection::new(map_size.clone(), project_name.clone(), z);
        map_collection.insert(z, collection);
    }

    let map_editor = MapEditor {
        maps: map_collection,
        size: map_size.value(),
    };

    let new_project =
        Project::new(project_name.clone(), ProjectType::MapEditor(map_editor));

    let project_saver = ProjectSaver { path: path.clone() };
    project_saver.save(&new_project).await?;

    program_data_lock.create_and_open_project(new_project.name.clone(), path);

    let mut loaded_projects_lock = loaded_projects.lock().await;
    loaded_projects_lock.insert(new_project.name.clone(), new_project);

    let program_data_saver = ProgramDataSaver {
        path: program_data_lock.config.config_path.clone(),
    };
    program_data_saver.save(&program_data_lock).await?;

    app.emit(
        events::CREATE_TAB,
        Tab {
            name: project_name.clone(),
            tab_type: TabType::MapEditor,
        },
    )?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum AddPaletteError {
    #[error(transparent)]
    SaveError(#[from] SaveError),

    #[error(transparent)]
    GetCurrentProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    InvalidProjectType(#[from] InvalidProjectType),
}

impl_serialize_for_error!(AddPaletteError);

#[tauri::command(rename_all = "camelCase")]
pub async fn add_palette(
    palette_name: String,
    program_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), AddPaletteError> {
    info!("Trying to add palette {} to project", palette_name);

    let program_data_lock = program_data.lock().await;
    let mut loaded_projects_lock = loaded_projects.lock().await;

    let mut loaded_project =
        get_current_project_mut(&program_data_lock, &mut loaded_projects_lock)?;

    let mut maps = match &mut loaded_project.project_type {
        ProjectType::MapEditor(me) => &mut me.maps,
        ProjectType::MapViewer(_) => Err(InvalidProjectType::NotAMapEditor)?,
    };

    Ok(())
}

// #[tauri::command(rename_all = "camelCase")]
// pub async fn update_editor() -> Result<(), ()> {}
