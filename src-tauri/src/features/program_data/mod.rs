pub mod handlers;
pub mod io;
mod keybinds;

use crate::data::io::DeserializedCDDAJsonData;
use crate::data::palettes::Palettes;
use crate::data::TileLayer;
use crate::features::map::importing::{
    OvermapSpecialImporter, OvermapSpecialImporterError, SingleMapDataImporter,
    SingleMapDataImporterError,
};
use crate::features::map::{
    CalculateParametersError, GetMappedCDDAIdsError, MapData,
    MappedCDDAIdsForTile, DEFAULT_MAP_DATA_SIZE,
};
use crate::features::program_data::keybinds::{Keybind, KeybindAction};
use crate::impl_serialize_for_error;
use crate::util::{IVec3JsonKey, Load, Save, SaveError};
use cdda_lib::types::CDDAIdentifier;
use futures_lite::StreamExt;
use glam::{IVec3, UVec2};
use log::info;
use serde::ser::SerializeMap;
use serde::Serializer;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use tauri::Theme;
use thiserror::Error;

pub const DEFAULT_CDDA_DATA_JSON_PATH: &'static str = "data/json";

pub type ZLevel = i32;
pub type MapCoordinates = UVec2;
pub type ProjectName = String;

#[derive(Debug, Error)]
pub enum GetLiveViewerDataError {
    #[error(transparent)]
    SingleImporterError(#[from] SingleMapDataImporterError),

    #[error(transparent)]
    OvermapSpecialImporterError(#[from] OvermapSpecialImporterError),
}

impl_serialize_for_error!(GetLiveViewerDataError);

pub async fn get_map_data_collection_from_live_viewer_data(
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

#[derive(Debug, Clone)]
pub struct MappedCDDAIdContainer {
    pub ids: HashMap<IVec3, MappedCDDAIdsForTile>,
}

impl Serialize for MappedCDDAIdContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_serializer =
            serializer.serialize_map(Some(self.ids.len()))?;
        for (key, value) in &self.ids {
            let key_wrapper = IVec3JsonKey(key.clone());
            map_serializer.serialize_entry(&key_wrapper, value)?;
        }
        map_serializer.end()
    }
}

impl MappedCDDAIdContainer {
    fn get_id_from_mapped_sprites(
        &self,
        cords: &IVec3,
        layer: &TileLayer,
    ) -> Option<CDDAIdentifier> {
        self.ids
            .get(cords)
            .map(|v| match layer {
                TileLayer::Terrain => {
                    v.terrain.clone().map(|v| v.tilesheet_id.id)
                },
                TileLayer::Furniture => {
                    v.furniture.clone().map(|v| v.tilesheet_id.id)
                },
                TileLayer::Monster => {
                    v.monster.clone().map(|v| v.tilesheet_id.id)
                },
                TileLayer::Field => v.field.clone().map(|v| v.tilesheet_id.id),
            })
            .flatten()
    }

    pub fn get_adjacent_identifiers(
        &self,
        coordinates: IVec3,
        layer: &TileLayer,
    ) -> AdjacentSprites {
        let top_cords = coordinates + IVec3::new(0, 1, 0);
        let top = self.get_id_from_mapped_sprites(&top_cords, &layer);

        let right_cords = coordinates + IVec3::new(1, 0, 0);
        let right = self.get_id_from_mapped_sprites(&right_cords, &layer);

        let bottom_cords = coordinates - IVec3::new(0, 1, 0);
        let bottom = self.get_id_from_mapped_sprites(&bottom_cords, &layer);

        let left_cords = coordinates - IVec3::new(1, 0, 0);
        let left = self.get_id_from_mapped_sprites(&left_cords, &layer);

        AdjacentSprites {
            top,
            right,
            bottom,
            left,
        }
    }
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
            name: "New Project".to_string(),
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
            cell_coordinates.x as i32
                + map_coordinates.x as i32 * DEFAULT_MAP_DATA_SIZE.x as i32,
            cell_coordinates.y as i32
                + map_coordinates.y as i32 * DEFAULT_MAP_DATA_SIZE.y as i32,
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
                        }
                        Some(omtm) => json_data.map_data.get_mut(&omtm.builtin).expect(
                            format!(
                                "Hardcoded Map data for predecessor {} to exist",
                                omtm.builtin
                            )
                                .as_str(),
                        ),
                    };

                    predecessor_map_data
                        .calculate_parameters(&json_data.palettes)
                        .unwrap();
                },
            }
        }
    }

    pub fn get_mapped_cdda_ids(
        &self,
        json_data: &DeserializedCDDAJsonData,
        z: ZLevel,
    ) -> Result<MappedCDDAIdContainer, GetMappedCDDAIdsError> {
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

        Ok(MappedCDDAIdContainer {
            ids: mapped_cdda_ids,
        })
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
    pub keybinds: HashSet<Keybind>,
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
            keybinds: HashSet::from_iter(vec![
                Keybind::with_ctrl("n").action(KeybindAction::NewProject),
                Keybind::with_ctrl("o").action(KeybindAction::OpenProject),
                Keybind::with_ctrl("s").action(KeybindAction::SaveProject),
                Keybind::with_ctrl("w").action(KeybindAction::CloseTab),
                Keybind::with_ctrl_alt("w").action(KeybindAction::CloseAllTabs),
                Keybind::with_ctrl("i").action(KeybindAction::ImportMap),
                Keybind::with_ctrl("e").action(KeybindAction::ExportMap),
                Keybind::with_ctrl_alt("s").action(KeybindAction::OpenSettings),
            ]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: PathBuf,
    pub name: String,
}

impl Hash for RecentProject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq<Self> for RecentProject {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for RecentProject {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorData {
    pub config: EditorConfig,

    #[serde(skip)]
    pub loaded_projects: HashMap<ProjectName, Project>,

    pub openable_projects: HashSet<ProjectName>,
    pub opened_project: Option<ProjectName>,
    pub recent_projects: HashSet<RecentProject>,

    pub available_tilesets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TabType {
    MapEditor,
    LiveViewer,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tab {
    pub name: String,
    pub tab_type: TabType,
}

#[derive(Debug)]
pub struct AdjacentSprites {
    pub top: Option<CDDAIdentifier>,
    pub right: Option<CDDAIdentifier>,
    pub bottom: Option<CDDAIdentifier>,
    pub left: Option<CDDAIdentifier>,
}
