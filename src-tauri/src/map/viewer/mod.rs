use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::{LiveViewerData, TabType};
use crate::editor_data::{EditorData, Project, ProjectType};
use crate::map::importing::{MapDataImporter, NestedMapDataImporter, SingleMapDataImporter};
use crate::map::ProjectContainer;
use crate::util::Load;
use glam::UVec2;
use log::info;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum OmTerrainType {
    Single { om_terrain_id: String },
    Nested { om_terrain_ids: Vec<Vec<String>> },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenViewerData {
    pub file_path: PathBuf,
    pub om_terrain: OmTerrainType,
}

#[tauri::command]
pub async fn open_viewer(
    app: AppHandle,
    data: OpenViewerData,
    editor_data: State<'_, Mutex<EditorData>>,
    project_container: State<'_, Mutex<ProjectContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), ()> {
    info!(
        "Opening Live viewer for om terrain ids {:?} at {:?}",
        &data.om_terrain, &data.file_path
    );

    let mut project_container_lock = project_container.lock().await;
    let json_data_lock = json_data.lock().await;

    let json_data = match json_data_lock.deref() {
        None => return Err(()),
        Some(d) => d,
    };

    match data.om_terrain {
        OmTerrainType::Single { om_terrain_id } => {
            let mut map_data_importer = SingleMapDataImporter {
                path: data.file_path.clone(),
                om_terrain: om_terrain_id.clone(),
            };

            let mut collection = map_data_importer.load().await.unwrap();
            collection.calculate_parameters(&json_data.palettes);

            let mut new_project = Project::new(
                "awd".into(),
                collection.global_map_size.clone(),
                ProjectType::Viewer,
            );
            new_project.maps.insert(0, collection);
            project_container_lock.data.push(new_project);

            create_tab(
                om_terrain_id.clone(),
                TabType::LiveViewer(LiveViewerData {
                    path: data.file_path,
                    om_terrain: om_terrain_id,
                }),
                app,
                editor_data,
            )
            .await?;
        }
        OmTerrainType::Nested { om_terrain_ids } => {
            let mut om_terrain_id_hashmap = HashMap::new();

            for (y, id_list) in om_terrain_ids.into_iter().enumerate() {
                for (x, id) in id_list.into_iter().enumerate() {
                    om_terrain_id_hashmap.insert(id, UVec2::new(x as u32, y as u32));
                }
            }

            let mut nested_importer = NestedMapDataImporter {
                path: data.file_path,
                om_terrain_ids: om_terrain_id_hashmap,
            };

            todo!()
        }
    }

    Ok(())
}
