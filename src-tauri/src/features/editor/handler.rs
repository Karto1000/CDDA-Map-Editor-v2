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
use cdda_lib::types::{CDDAIdentifier, MapGenValue};
use glam::{IVec2, IVec3, UVec2};
use log::info;
use rayon::max_num_threads;
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

    #[error("Mapgen at coordinates {0} (x,y,z) does not exist")]
    MissingMapgen(IVec3),
}

impl_serialize_for_error!(AddPaletteError);

#[derive(Debug, Clone, Deserialize)]
#[serde(
    rename_all = "camelCase",
    tag = "type",
    rename_all_fields = "camelCase"
)]
pub enum ModifyPaletteAction {
    AddPalette { palette_name: CDDAIdentifier },
    RemovePalette { index: usize },
}

#[tauri::command(rename_all = "camelCase")]
pub async fn modify_palette(
    app: AppHandle,
    coordinates: IVec3,
    action: ModifyPaletteAction,
    program_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), AddPaletteError> {
    let program_data_lock = program_data.lock().await;
    let mut loaded_projects_lock = loaded_projects.lock().await;

    let loaded_project =
        get_current_project_mut(&program_data_lock, &mut loaded_projects_lock)?;

    let maps = match &mut loaded_project.project_type {
        ProjectType::MapEditor(me) => &mut me.maps,
        ProjectType::MapViewer(_) => Err(InvalidProjectType::NotAMapEditor)?,
    };

    let map_data = maps
        .get_mut(&coordinates.z)
        .ok_or(AddPaletteError::MissingMapgen(coordinates.clone()))?
        .maps
        .get_mut(&UVec2::new(coordinates.x as u32, coordinates.y as u32).into())
        .ok_or(AddPaletteError::MissingMapgen(coordinates.clone()))?;

    match action {
        ModifyPaletteAction::AddPalette { palette_name } => {
            info!("Trying to add palette {} to project", palette_name);
            map_data.palettes.push(MapGenValue::String(palette_name));
        },
        ModifyPaletteAction::RemovePalette { index } => {
            info!("Trying to remove palette at index {} from project", index);
            map_data.palettes.remove(index);
        },
    }

    let path = match program_data_lock
        .openable_projects
        .get(&loaded_project.name)
    {
        None => unreachable!(),
        Some(p) => p,
    };

    let project_saver = ProjectSaver { path: path.clone() };
    project_saver.save(&loaded_project).await?;

    app.emit(events::CURRENT_PROJECT_CHANGED, loaded_project.clone())
        .unwrap();

    Ok(())
}

// #[tauri::command(rename_all = "camelCase")]
// pub async fn update_editor() -> Result<(), ()> {}
