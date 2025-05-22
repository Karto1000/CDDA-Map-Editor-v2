use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::overmap::OvermapSpecialOvermap;
use crate::editor_data::{
    EditorData, EditorDataSaver, LiveViewerData, MapDataCollection, Project,
    ProjectType,
};
use crate::map::importing::{OvermapSpecialImporter, SingleMapDataImporter};
use crate::map::{Serializer, DEFAULT_MAP_DATA_SIZE};
use crate::tab::{Tab, TabType};
use crate::util::{get_json_data, get_size, CDDAIdentifier, Load};
use crate::util::{CDDADataError, Save};
use crate::{events, impl_serialize_for_error};
use anyhow::Error;
use glam::UVec2;
use log::info;
use notify::Watcher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::sync::Mutex;

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
}

impl_serialize_for_error!(OpenViewerError);

#[tauri::command]
pub async fn open_viewer(
    app: AppHandle,
    data: OpenViewerData,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), OpenViewerError> {
    info!("Opening Live viewer");

    let mut editor_data_lock = editor_data.lock().await;
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;

    match data {
        OpenViewerData::Terrain {
            project_name,
            mapgen_file_paths,
            om_id,
        } => {
            if editor_data_lock
                .projects
                .iter()
                .find(|p| p.name == project_name)
                .is_some()
            {
                return Err(OpenViewerError::ProjectAlreadyExists);
            }

            let mut overmap_terrain_importer = SingleMapDataImporter {
                om_terrain: om_id.clone(),
                paths: mapgen_file_paths.clone(),
            };

            let mut collection = overmap_terrain_importer.load().await.unwrap();
            collection.calculate_parameters(&json_data.palettes);

            let mut new_project = Project::new(
                project_name.clone(),
                DEFAULT_MAP_DATA_SIZE,
                ProjectType::LiveViewer(LiveViewerData::Terrain {
                    mapgen_file_paths,
                    project_name: project_name.clone(),
                    om_id,
                }),
            );

            new_project.maps.insert(0, collection);
            editor_data_lock.projects.push(new_project);

            editor_data_lock.opened_project = Some(project_name.clone());
            app.emit(
                events::TAB_CREATED,
                Tab {
                    name: project_name.clone(),
                    tab_type: TabType::LiveViewer,
                },
            )?;
        },
        OpenViewerData::Special {
            project_name,
            mapgen_file_paths,
            om_file_paths,
            om_id,
        } => {
            if editor_data_lock
                .projects
                .iter()
                .find(|p| p.name == project_name)
                .is_some()
            {
                return Err(OpenViewerError::ProjectAlreadyExists);
            }

            let mut overmap_special_importer = OvermapSpecialImporter {
                om_special_id: om_id.clone(),
                overmap_special_paths: om_file_paths.clone(),
                mapgen_entry_paths: mapgen_file_paths.clone(),
            };

            let mut maps = overmap_special_importer.load().await.unwrap();

            maps.iter_mut()
                .for_each(|(_, m)| m.calculate_parameters(&json_data.palettes));

            let mut new_project = Project::new(
                project_name.clone(),
                get_size(&maps),
                ProjectType::LiveViewer(LiveViewerData::Special {
                    mapgen_file_paths,
                    om_file_paths,
                    project_name: project_name.clone(),
                    om_id,
                }),
            );

            new_project.maps = maps;
            editor_data_lock.projects.push(new_project);

            editor_data_lock.opened_project = Some(project_name.clone());
            app.emit(
                events::TAB_CREATED,
                Tab {
                    name: project_name.clone(),
                    tab_type: TabType::LiveViewer,
                },
            )?;
        },
    }

    let saver = EditorDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(editor_data_lock.deref()).await.unwrap();

    Ok(())
}
