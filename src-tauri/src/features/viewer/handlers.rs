use crate::data::io::DeserializedCDDAJsonData;
use crate::data::replace_region_setting;
use crate::data::TileLayer;
use crate::events::UPDATE_LIVE_VIEWER;
use crate::features::map::importing::{
    OvermapSpecialImporter, SingleMapDataImporter,
};
use crate::features::map::{
    CalculateParametersError, InstantiatedOvermapStack, MAX_MAP_DATA_SIZE,
};
use crate::features::map::{CalculateRandomParameters, MappedCDDAId};
use crate::features::program_data::handlers::{
    save_program_data, SaveEditorDataError,
};
use crate::features::program_data::io::{ProgramDataSaver, ProjectSaver};
use crate::features::program_data::Project;
use crate::features::program_data::ProjectType;
use crate::features::program_data::ZLevel;
use crate::features::program_data::{
    get_map_data_collection_from_map_viewer, Tab, TabType,
};
use crate::features::program_data::{GetLiveViewerDataError, LoadedProjects};
use crate::features::program_data::{ProgramData, SavedProject};
use crate::features::sprites::{
    InstancedFallbackSprite, InstancedSprite, InstancedSprites,
};
use crate::features::tileset::legacy_tileset::Tilesheet;
use crate::features::tileset::legacy_tileset::TilesheetCDDAId;
use crate::features::tileset::GetSprite;
use crate::features::viewer::{LiveViewerData, MapViewer};
use crate::util;
use crate::util::GetCurrentProjectError;
use crate::util::Save;
use crate::util::{get_current_project, get_json_data, get_json_data_mut};
use crate::util::{get_current_project_mut, get_size, Load};
use crate::util::{CDDADataError, SaveError};
use crate::{events, load_projects};
use crate::{impl_serialize_for_error, InvalidProjectType};
use cdda_lib::serde_vec::serialize_hashmap_with_ivec3_keys;
use cdda_lib::types::{CDDAIdentifier, ParameterIdentifier};
use cdda_lib::MapgenCellCoordinates;
use cdda_lib::{DEFAULT_CELL_CHARACTER, DEFAULT_EMPTY_CHAR_ROW};
use cdda_lib::{DEFAULT_MAP_ROWS, MAX_MAPGEN_HEIGHT};
use comfy_bounded_ints::prelude::Bound_u32;
use comfy_bounded_ints::types::Bound_usize;
use glam::IVec3;
use glam::UVec2;
use indexmap::IndexMap;
use log::error;
use log::info;
use log::warn;
use notify::{recommended_watcher, Watcher};
use notify_debouncer_full::new_debouncer;
use rand::rng;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde_json::{json, to_value};
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use strum::IntoEnumIterator;
use tauri::async_runtime::Mutex;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::State;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::MutexGuard;
use tokio_test::block_on;

#[derive(Debug, Error, Serialize)]
pub enum GetMapViewerError {
    #[error("The map was not found")]
    NotFound,

    #[error(transparent)]
    GetCurrentProjectError(#[from] GetCurrentProjectError),
}

#[derive(Debug, Error)]
pub enum GetSpritesError {
    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error(transparent)]
    GetCurrentProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    GetMapViewerError(#[from] GetMapViewerError),
}

impl_serialize_for_error!(GetSpritesError);

#[tauri::command]
pub async fn get_sprites(
    tilesheet: State<'_, Mutex<Option<Tilesheet>>>,
    fallback_tilesheet: State<'_, Arc<Tilesheet>>,
    editor_data: State<'_, Mutex<ProgramData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    existing_instantiation: State<'_, Mutex<Option<InstantiatedOvermapStack>>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<InstancedSprites, GetSpritesError> {
    let mut json_data_lock = json_data.lock().await;
    let json_data = get_json_data_mut(&mut json_data_lock)?;

    let mut loaded_projects_lock = loaded_projects.lock().await;

    let editor_data_lock = editor_data.lock().await;
    let project =
        get_current_project_mut(&editor_data_lock, &mut loaded_projects_lock)?;

    let tilesheet_lock = tilesheet.lock().await;

    let instantiated_overmap_stack = project
        .project_type
        .maps()
        .par_iter()
        .fold(
            || InstantiatedOvermapStack::default(),
            |mut acc, (z, overmap)| {
                let instantiated = overmap
                    .instantiate(CalculateRandomParameters, json_data)
                    .unwrap();
                acc.instantiated_overmaps.insert(z.clone(), instantiated);
                acc
            },
        )
        .reduce(
            || InstantiatedOvermapStack::default(),
            |mut acc, partial| {
                acc.instantiated_overmaps
                    .extend(partial.instantiated_overmaps);
                acc
            },
        );

    let instanced_sprites =
        crate::features::sprites::get_sprites_from_instantiated_overmaps_stack(
            &instantiated_overmap_stack,
            tilesheet_lock.as_ref(),
            &fallback_tilesheet,
            &json_data,
        );

    let mut existing_instantiation_lock = existing_instantiation.lock().await;
    existing_instantiation_lock.replace(instantiated_overmap_stack);

    Ok(instanced_sprites)
}

#[derive(Debug, Error)]
pub enum ReloadProjectError {
    #[error(transparent)]
    InvalidProjectType(#[from] InvalidProjectType),

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error(transparent)]
    ProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    GetLiveViewerError(#[from] GetLiveViewerDataError),

    #[error(transparent)]
    CalculateParametersError(#[from] CalculateParametersError),

    #[error(transparent)]
    GetMapViewerError(#[from] GetMapViewerError),
}

impl_serialize_for_error!(ReloadProjectError);

#[tauri::command]
pub async fn reload_project(
    editor_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), ReloadProjectError> {
    let mut loaded_projects_lock = loaded_projects.lock().await;
    let editor_data_lock = editor_data.lock().await;
    let project =
        get_current_project_mut(&editor_data_lock, &mut loaded_projects_lock)?;

    match &mut project.project_type {
        ProjectType::MapEditor(_) => {
            return Err(InvalidProjectType::NotAMapViewer)?;
        },
        ProjectType::MapViewer(map_viewer) => {
            let map_data_collection =
                get_map_data_collection_from_map_viewer(map_viewer).await?;

            map_viewer.maps = map_data_collection;
        },
    }

    Ok(())
}

#[derive(Debug, Error, Serialize)]
pub enum GetProjectCellDataError {
    #[error(transparent)]
    MapError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error("No map is opened")]
    NoMapOpened,
}

#[tauri::command]
pub async fn get_project_cell_data(
    instantiated_overmap_stack: State<
        '_,
        Mutex<Option<InstantiatedOvermapStack>>,
    >,
) -> Result<InstantiatedOvermapStack, GetProjectCellDataError> {
    let instantiated_overmap_stack_lock =
        instantiated_overmap_stack.lock().await;
    let existing_overmaps = match instantiated_overmap_stack_lock.deref() {
        None => return Err(GetProjectCellDataError::NoMapOpened),
        Some(m) => m,
    };

    Ok(existing_overmaps.clone())
}

#[derive(Debug, Error)]
pub enum NewMapgenViewerError {
    #[error(transparent)]
    OpenViewerError(#[from] OpenViewerError),
}

impl_serialize_for_error!(NewMapgenViewerError);

#[tauri::command]
pub async fn new_single_mapgen_viewer(
    path: PathBuf,
    project_save_path: PathBuf,
    om_terrain_name: String,
    project_name: String,
    app: AppHandle,
    editor_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), NewMapgenViewerError> {
    let data = serde_json::to_string_pretty(&json!(
        [
            {
                "type": "mapgen",
                "method": "json",
                "om_terrain": om_terrain_name,
                "object": {
                    "fill_ter": "t_region_groundcover",
                    "rows": DEFAULT_MAP_ROWS
                }
            }
        ]
    ))
    .unwrap();

    let mut file = File::create(&path).await.unwrap();

    file.write_all(data.as_bytes()).await.unwrap();

    create_viewer(
        app,
        project_save_path,
        OpenViewerData::Terrain {
            mapgen_file_paths: vec![path],
            project_name,
            om_id: CDDAIdentifier(om_terrain_name),
        },
        editor_data,
        loaded_projects,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn new_special_mapgen_viewer(
    path: PathBuf,
    project_save_path: PathBuf,
    om_terrain_name: String,
    project_name: String,
    special_width: Bound_u32<1, { u32::MAX }>,
    special_height: Bound_u32<1, { u32::MAX }>,
    special_z_from: i32,
    special_z_to: i32,
    app: AppHandle,
    editor_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), NewMapgenViewerError> {
    let mut data = Vec::new();

    let mut overmaps_list = Vec::new();

    for z in special_z_from..=special_z_to {
        for y in 0..special_height.get() {
            for x in 0..special_width.get() {
                let om_terrain_name =
                    format!("{}_{}_{}_{}", om_terrain_name, x, y, z);

                overmaps_list.push(json!({
                   "point": [x, y, z],
                    "overmap": om_terrain_name,
                }));
            }
        }
    }

    data.push(json!({
        "type": "overmap_special",
        "id": om_terrain_name,
        "overmaps": overmaps_list
    }));

    for z in special_z_from..=special_z_to {
        let mut z_om_terrain_names = Vec::new();
        z_om_terrain_names.reserve(special_height.get() as usize);

        for y in 0..special_height.get() {
            let mut y_om_terrain_names = Vec::new();
            y_om_terrain_names.reserve(special_width.get() as usize);

            for x in 0..special_width.get() {
                let om_terrain_name =
                    format!("{}_{}_{}_{}", om_terrain_name, x, y, z);
                y_om_terrain_names.push(om_terrain_name.clone());
            }

            z_om_terrain_names.push(y_om_terrain_names)
        }

        let mut rows = Vec::new();

        for _ in 0..special_height.get() * MAX_MAPGEN_HEIGHT {
            rows.push(
                DEFAULT_EMPTY_CHAR_ROW.repeat(special_width.get() as usize),
            );
        }

        data.push(json!(
            {
                "type": "mapgen",
                "method": "json",
                "om_terrain": z_om_terrain_names,
                "object": {
                    "fill_ter": "t_region_groundcover",
                    "rows": rows
                }
            }
        ));
    }

    let data_ser = serde_json::to_string_pretty(&data).unwrap();
    let mut file = File::create(&path).await.unwrap();
    file.write_all(data_ser.as_bytes()).await.unwrap();

    create_viewer(
        app,
        project_save_path,
        OpenViewerData::Special {
            mapgen_file_paths: vec![path.clone()],
            om_file_paths: vec![path.clone()],
            project_name,
            om_id: CDDAIdentifier(om_terrain_name),
        },
        editor_data,
        loaded_projects,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn new_nested_mapgen_viewer(
    path: PathBuf,
    project_save_path: PathBuf,
    om_terrain_name: String,
    project_name: String,
    nested_width: Bound_usize<1, 24>,
    nested_height: Bound_usize<1, 24>,
    app: AppHandle,
    editor_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), NewMapgenViewerError> {
    let mut rows = Vec::new();

    for _ in 0..nested_height.get() {
        rows.push(
            DEFAULT_CELL_CHARACTER
                .to_string()
                .repeat(nested_width.get()),
        );
    }

    let data = json!(
        [
            {
                "type": "mapgen",
                "method": "json",
                "nested_mapgen_id": om_terrain_name,
                "object": {
                    "mapgensize": [nested_width, nested_height],
                    "fill_ter": "t_region_groundcover",
                    "rows": rows
                }
            }
        ]
    );

    let data_ser = serde_json::to_string_pretty(&data).unwrap();
    let mut file = File::create(&path).await.unwrap();
    file.write_all(data_ser.as_bytes()).await.unwrap();

    create_viewer(
        app,
        project_save_path,
        OpenViewerData::Terrain {
            mapgen_file_paths: vec![path.clone()],
            project_name,
            om_id: CDDAIdentifier(om_terrain_name),
        },
        editor_data,
        loaded_projects,
    )
    .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum OpenViewerData {
    Terrain {
        mapgen_file_paths: Vec<PathBuf>,
        project_name: String,
        om_id: CDDAIdentifier,
    },
    Special {
        mapgen_file_paths: Vec<PathBuf>,
        om_file_paths: Vec<PathBuf>,
        project_name: String,
        om_id: CDDAIdentifier,
    },
}

#[derive(Debug, Error)]
pub enum OpenViewerError {
    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error(transparent)]
    TauriError(#[from] tauri::Error),

    #[error("Another project with the same name already exists")]
    ProjectAlreadyExists,

    #[error(transparent)]
    CalculateParametersError(#[from] CalculateParametersError),

    #[error(transparent)]
    SaveEditorDataError(#[from] SaveEditorDataError),

    #[error(transparent)]
    SaveProjectError(#[from] SaveError),
}
impl_serialize_for_error!(OpenViewerError);

#[tauri::command]
pub async fn create_viewer(
    app: AppHandle,
    project_save_path: PathBuf,
    data: OpenViewerData,
    editor_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), OpenViewerError> {
    info!("Creating Live viewer");

    let mut program_data_lock = editor_data.lock().await;
    let mut loaded_projects_lock = loaded_projects.lock().await;

    let project = match data {
        OpenViewerData::Terrain {
            project_name,
            mapgen_file_paths,
            om_id,
        } => {
            if loaded_projects_lock.get(&project_name).is_some() {
                return Err(OpenViewerError::ProjectAlreadyExists);
            }

            let mut overmap_terrain_importer = SingleMapDataImporter {
                om_terrain: om_id.clone(),
                paths: mapgen_file_paths.clone(),
            };

            let collection = overmap_terrain_importer.load().await.unwrap();
            let mut maps = HashMap::new();
            maps.insert(0, collection);

            let map_viewer = MapViewer {
                maps,
                data: LiveViewerData::Terrain {
                    mapgen_file_paths,
                    project_name: project_name.clone(),
                    om_id,
                },
                size: MAX_MAP_DATA_SIZE,
            };

            let project = Project::new(
                project_name.clone(),
                ProjectType::MapViewer(map_viewer),
            );

            project
        },
        OpenViewerData::Special {
            project_name,
            mapgen_file_paths,
            om_file_paths,
            om_id,
        } => {
            if loaded_projects_lock.get(&project_name).is_some() {
                return Err(OpenViewerError::ProjectAlreadyExists);
            }

            let mut overmap_special_importer = OvermapSpecialImporter {
                om_special_id: om_id.clone(),
                overmap_special_paths: om_file_paths.clone(),
                mapgen_entry_paths: mapgen_file_paths.clone(),
            };

            let mut maps = overmap_special_importer.load().await.unwrap();
            let map_size = get_size(&maps);

            let map_viewer = MapViewer {
                maps,
                data: LiveViewerData::Special {
                    mapgen_file_paths,
                    om_file_paths,
                    project_name: project_name.clone(),
                    om_id,
                },
                size: map_size,
            };

            let project = Project::new(
                project_name.clone(),
                ProjectType::MapViewer(map_viewer),
            );

            program_data_lock.opened_project = Some(project_name.clone());

            project
        },
    };

    let project_saver = ProjectSaver {
        path: project_save_path.clone(),
    };
    project_saver.save(&project).await?;

    app.emit(
        events::CREATE_TAB,
        Tab {
            name: project.name.clone(),
            tab_type: TabType::LiveViewer,
        },
    )?;

    program_data_lock
        .create_and_open_project(project.name.clone(), project_save_path);
    loaded_projects_lock.insert(project.name.clone(), project);

    app.emit(events::EDITOR_DATA_CHANGED, program_data_lock.clone())?;
    drop(program_data_lock);

    save_program_data(editor_data).await?;

    Ok(())
}
