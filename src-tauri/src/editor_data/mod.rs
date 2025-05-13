pub(crate) mod handlers;
pub(crate) mod tab;

use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::palettes::Palettes;
use crate::editor_data::tab::Tab;
use crate::map::{CellRepresentation, MapData, MappingKind, DEFAULT_MAP_DATA_SIZE};
use crate::tileset::legacy_tileset::MappedCDDAIds;
use crate::util::Save;
use glam::{IVec3, UVec2};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use tauri::Theme;
use thiserror::Error;

pub const DEFAULT_CDDA_DATA_JSON_PATH: &'static str = "data/json";

pub type ZLevel = i32;
pub type MapCoordinates = UVec2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
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
            ty: ProjectType::Editor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDataCollection {
    pub maps: HashMap<MapCoordinates, MapData>,
    pub global_map_size: UVec2,
}

impl MapDataCollection {
    pub fn map_to_global_cell_coords(
        map_coordinates: &MapCoordinates,
        cell_coordinates: &UVec2,
        z: ZLevel,
    ) -> IVec3 {
        IVec3::new(
            (cell_coordinates.x + map_coordinates.x * DEFAULT_MAP_DATA_SIZE.x) as i32,
            (cell_coordinates.y + map_coordinates.y * DEFAULT_MAP_DATA_SIZE.y) as i32,
            z,
        )
    }

    pub fn get_mapped_cdda_ids(
        &self,
        json_data: &DeserializedCDDAJsonData,
        z: ZLevel,
    ) -> HashMap<IVec3, MappedCDDAIds> {
        let mut mapped_cdda_ids = HashMap::new();

        for (map_coords, map_data) in self.maps.iter() {
            let mut ids = map_data.get_mapped_cdda_ids(json_data, z);

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

        mapped_cdda_ids
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
                let new_cell_coords =
                    Self::map_to_global_cell_coords(map_coords, &cell_coords, 0).as_uvec3();
                new_repr.insert(UVec2::new(new_cell_coords.x, new_cell_coords.y), cell_repr);
            }

            cell_repr.extend(new_repr);
        }

        cell_repr
    }

    pub fn calculate_parameters(&mut self, all_palettes: &Palettes) {
        for (_, map_data) in self.maps.iter_mut() {
            map_data.calculate_parameters(all_palettes);
        }
    }
}

impl Default for MapDataCollection {
    fn default() -> Self {
        let mut maps = HashMap::new();
        maps.insert(MapCoordinates::ZERO, MapData::default());
        Self {
            maps,
            global_map_size: DEFAULT_MAP_DATA_SIZE,
        }
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorData {
    pub config: EditorConfig,

    pub tabs: Vec<Tab>,

    pub available_tilesets: Option<Vec<String>>,
}

pub struct EditorDataSaver {
    pub path: PathBuf,
}

impl Save<EditorData> for EditorDataSaver {
    fn save(&self, data: &EditorData) -> Result<(), Error> {
        let serialized = serde_json::to_string_pretty(data).expect("Serialization to not fail");
        fs::write(self.path.join("config.json"), serialized)?;
        info!("Saved EditorData to {}", self.path.display());
        Ok(())
    }
}
