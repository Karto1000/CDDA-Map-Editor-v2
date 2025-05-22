pub(crate) mod handlers;

use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::palettes::Palettes;
use crate::impl_serialize_for_error;
use crate::map::importing::{
    OvermapSpecialImporter, OvermapSpecialImporterError, SingleMapDataImporter,
    SingleMapDataImporterError,
};
use crate::map::{
    CalculateParametersError, CellRepresentation, GetMappedCDDAIdsError,
    MapData, DEFAULT_MAP_DATA_SIZE,
};
use crate::tileset::legacy_tileset::MappedCDDAIds;
use crate::util::{CDDAIdentifier, Load, Save, SaveError};
use futures_lite::StreamExt;
use glam::{IVec3, UVec2};
use log::info;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tauri::Theme;
use thiserror::Error;

pub const DEFAULT_CDDA_DATA_JSON_PATH: &'static str = "data/json";

pub type ZLevel = i32;
pub type MapCoordinates = UVec2;

#[derive(Debug, Error)]
pub enum GetLiveViewerDataError {
    #[error(transparent)]
    SingleImporterError(#[from] SingleMapDataImporterError),

    #[error(transparent)]
    OvermapSpecialImporterError(#[from] OvermapSpecialImporterError),
}

impl_serialize_for_error!(GetLiveViewerDataError);

pub async fn get_map_data_collection_live_viewer_data(
    data: &LiveViewerData,
) -> Result<HashMap<ZLevel, MapDataCollection>, GetLiveViewerDataError> {
    info!("Opening Live viewer");

    let map_data_collection = match &data {
        LiveViewerData::Terrain {
            om_id,
            mapgen_file_paths,
            ..
        } => {
            let mut overmap_terrain_importer = SingleMapDataImporter {
                om_terrain: om_id.clone(),
                paths: mapgen_file_paths.clone(),
            };

            let collection = overmap_terrain_importer.load().await?;
            let mut map_data_collection = HashMap::new();
            map_data_collection.insert(0, collection);
            map_data_collection
        },
        LiveViewerData::Special {
            om_id,
            om_file_paths,
            mapgen_file_paths,
            ..
        } => {
            let mut om_special_importer = OvermapSpecialImporter {
                om_special_id: om_id.clone(),
                overmap_special_paths: om_file_paths.clone(),
                mapgen_entry_paths: mapgen_file_paths.clone(),
            };

            om_special_importer.load().await?
        },
    };

    Ok(map_data_collection)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProjectType {
    MapEditor(ProjectSaveState),
    LiveViewer(LiveViewerData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveViewerData {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum ProjectSaveState {
    #[default]
    Unsaved,
    Saved {
        path: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,

    #[serde(skip)]
    pub maps: HashMap<ZLevel, MapDataCollection>,

    pub size: UVec2,
    pub ty: ProjectType,
}

impl Project {
    pub fn new(name: String, size: UVec2, ty: ProjectType) -> Self {
        let mut maps = HashMap::new();
        let map_collection = MapDataCollection::default();
        maps.insert(0, map_collection);

        Self {
            name,
            maps,
            size,
            ty,
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        let mut maps = HashMap::new();
        let map_collection = MapDataCollection::default();
        maps.insert(0, map_collection);

        Self {
            name: "Unnamed".to_string(),
            maps,
            size: DEFAULT_MAP_DATA_SIZE,
            ty: ProjectType::MapEditor(ProjectSaveState::Unsaved),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDataCollection {
    pub maps: HashMap<MapCoordinates, MapData>,
}

impl MapDataCollection {
    pub fn map_to_global_cell_coords(
        map_coordinates: &MapCoordinates,
        cell_coordinates: &UVec2,
        z: ZLevel,
    ) -> IVec3 {
        IVec3::new(
            (cell_coordinates.x + map_coordinates.x * DEFAULT_MAP_DATA_SIZE.x)
                as i32,
            (cell_coordinates.y + map_coordinates.y * DEFAULT_MAP_DATA_SIZE.y)
                as i32,
            z,
        )
    }

    pub fn calculate_predecessor_parameters(
        &mut self,
        json_data: &mut DeserializedCDDAJsonData,
    ) {
        for (_, map) in self.maps.iter_mut() {
            match &map.predecessor {
                None => {},
                Some(predecessor_id) => {
                    let predecessor = json_data
                        .overmap_terrains
                        .get_mut(predecessor_id)
                        .expect(
                            format!(
                                "Overmap terrain for Predecessor {} to exist",
                                predecessor_id
                            )
                            .as_str(),
                        );

                    let predecessor_map_data = match &predecessor
                        .mapgen
                        .clone()
                        .unwrap_or_default()
                        .first_mut()
                    {
                        None => {
                            // This terrain is defined in a json file, so we can just search for it
                            json_data.map_data.get_mut(predecessor_id).expect(
                                format!(
                                    "Mapdata for Predecessor {} to exist",
                                    predecessor_id
                                )
                                    .as_str(),
                            )
                        },
                        Some(omtm) => json_data.map_data.get_mut(&omtm.name).expect(
                            format!(
                                "Hardcoded Map data for predecessor {} to exist",
                                omtm.name
                            )
                                .as_str(),
                        ),
                    };

                    predecessor_map_data
                        .calculate_parameters(&json_data.palettes);
                },
            }
        }
    }

    pub fn get_mapped_cdda_ids(
        &self,
        json_data: &DeserializedCDDAJsonData,
        z: ZLevel,
    ) -> Result<HashMap<IVec3, MappedCDDAIds>, GetMappedCDDAIdsError> {
        let mut mapped_cdda_ids = HashMap::new();

        for (map_coords, map_data) in self.maps.iter() {
            let mut ids = map_data.get_mapped_cdda_ids(json_data, z)?;

            // Transform every coordinate in the hashmap
            let mut new_ids = HashMap::new();

            for (cell_coords, cdda_ids) in ids.drain() {
                let new_cell_coords = Self::map_to_global_cell_coords(
                    map_coords,
                    &UVec2::new(cell_coords.x as u32, cell_coords.y as u32),
                    z,
                );
                new_ids.insert(new_cell_coords, cdda_ids);
            }

            mapped_cdda_ids.extend(new_ids);
        }

        Ok(mapped_cdda_ids)
    }

    pub fn get_representations(
        &self,
        json_data: &DeserializedCDDAJsonData,
    ) -> HashMap<UVec2, CellRepresentation> {
        let mut cell_repr: HashMap<UVec2, CellRepresentation> = HashMap::new();

        for (map_coords, map_data) in self.maps.iter() {
            let mut repr = map_data.get_representations(json_data);

            let mut new_repr = HashMap::new();

            for (cell_coords, cell_repr) in repr.drain() {
                let new_cell_coords = Self::map_to_global_cell_coords(
                    map_coords,
                    &cell_coords,
                    0,
                )
                .as_uvec3();
                new_repr.insert(
                    UVec2::new(new_cell_coords.x, new_cell_coords.y),
                    cell_repr,
                );
            }

            cell_repr.extend(new_repr);
        }

        cell_repr
    }

    pub fn calculate_parameters(
        &mut self,
        all_palettes: &Palettes,
    ) -> Result<(), CalculateParametersError> {
        for (_, map_data) in self.maps.iter_mut() {
            map_data.calculate_parameters(all_palettes)?;
        }

        Ok(())
    }
}

impl Default for MapDataCollection {
    fn default() -> Self {
        let mut maps = HashMap::new();
        maps.insert(MapCoordinates::ZERO, MapData::default());
        Self { maps }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub cdda_path: Option<PathBuf>,
    pub json_data_path: PathBuf,
    pub config_path: PathBuf,
    pub selected_tileset: Option<String>,
    pub theme: Theme,
}

#[derive(Debug, Serialize, Error)]
pub enum CDDAPathError {
    #[error("There was no CDDA path that was set")]
    NoCDDAPathSet,
}

#[derive(Debug, Serialize, Error)]
pub enum SelectedTilesetError {
    #[error("No Tileset was selected")]
    NoTilesetSelected,
}

impl EditorConfig {
    pub fn get_cdda_path(&self) -> Result<PathBuf, CDDAPathError> {
        self.cdda_path
            .as_ref()
            .ok_or(CDDAPathError::NoCDDAPathSet)
            .map(Clone::clone)
    }

    pub fn get_selected_tileset(&self) -> Result<String, SelectedTilesetError> {
        self.selected_tileset
            .as_ref()
            .ok_or(SelectedTilesetError::NoTilesetSelected)
            .map(Clone::clone)
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            cdda_path: None,
            config_path: Default::default(),
            selected_tileset: None,
            json_data_path: DEFAULT_CDDA_DATA_JSON_PATH.into(),
            theme: Theme::Dark,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorData {
    pub config: EditorConfig,
    pub projects: Vec<Project>,
    pub opened_project: Option<String>,
    pub available_tilesets: Option<Vec<String>>,
}

pub struct EditorDataSaver {
    pub path: PathBuf,
}

impl Save<EditorData> for EditorDataSaver {
    async fn save(&self, data: &EditorData) -> Result<(), SaveError> {
        let serialized = serde_json::to_string_pretty(data)
            .expect("Serialization to not fail");
        fs::write(self.path.join("config.json"), serialized)?;
        info!("Saved EditorData to {}", self.path.display());
        Ok(())
    }
}
