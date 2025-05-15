use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::editor_data::{
    EditorData, EditorDataSaver, LiveViewerData, OmTerrainType, Project, ProjectType,
};
use crate::map::importing::{NestedMapDataImporter, SingleMapDataImporter};
use crate::map::Serializer;
use crate::tab::{Tab, TabType};
use crate::util::{get_json_data, Load};
use crate::util::{CDDADataError, Save};
use crate::{events, impl_serialize_for_error};
use derive_more::Display;
use glam::UVec2;
use image::save_buffer;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenViewerData {
    pub file_path: PathBuf,
    pub project_name: String,
    pub om_terrain: OmTerrainType,
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
    info!(
        "Opening Live viewer for om terrain ids {:?} at {:?}",
        &data.om_terrain, &data.file_path
    );

    let mut editor_data_lock = editor_data.lock().await;
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;

    if editor_data_lock
        .projects
        .iter()
        .find(|p| p.name == data.project_name)
        .is_some()
    {
        return Err(OpenViewerError::ProjectAlreadyExists);
    }

    match data.om_terrain.clone() {
        OmTerrainType::Single { om_terrain_id } => {
            let mut map_data_importer = SingleMapDataImporter {
                path: data.file_path.clone(),
                om_terrain: om_terrain_id.clone(),
            };

            let mut collection = map_data_importer.load().await.unwrap();
            collection.calculate_parameters(&json_data.palettes);

            let mut new_project = Project::new(
                data.project_name.clone(),
                collection.global_map_size.clone(),
                ProjectType::LiveViewer(LiveViewerData {
                    path: data.file_path,
                    om_terrain: data.om_terrain,
                }),
            );
            new_project.maps.insert(0, collection);

            editor_data_lock.projects.push(new_project);
            editor_data_lock.opened_project = Some(data.project_name.clone());
        }
        OmTerrainType::Nested { om_terrain_ids } => {
            let mut om_terrain_id_hashmap = HashMap::new();

            for (y, id_list) in om_terrain_ids.into_iter().enumerate() {
                for (x, id) in id_list.into_iter().enumerate() {
                    om_terrain_id_hashmap.insert(id, UVec2::new(x as u32, y as u32));
                }
            }

            let mut nested_importer = NestedMapDataImporter {
                path: data.file_path.clone(),
                om_terrain_ids: om_terrain_id_hashmap,
            };

            let mut collection = nested_importer.load().await.unwrap();
            collection.calculate_parameters(&json_data.palettes);

            let mut new_project = Project::new(
                data.project_name.clone(),
                collection.global_map_size.clone(),
                ProjectType::LiveViewer(LiveViewerData {
                    path: data.file_path,
                    om_terrain: data.om_terrain,
                }),
            );
            new_project.maps.insert(0, collection);

            editor_data_lock.projects.push(new_project);
            editor_data_lock.opened_project = Some(data.project_name.clone());
        }
    }

    app.emit(
        events::TAB_CREATED,
        Tab {
            name: data.project_name.clone(),
            tab_type: TabType::LiveViewer,
        },
    )?;

    let saver = EditorDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(editor_data_lock.deref()).await.unwrap();

    Ok(())
}
