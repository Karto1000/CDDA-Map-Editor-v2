pub mod handlers;
pub mod io;
mod keybinds;

use crate::data::io::DeserializedCDDAJsonData;
use crate::data::palettes::Palettes;
use crate::data::TileLayer;
use crate::features::editor::{MapEditor, MapSize};
use crate::features::map::importing::{
    OvermapSpecialImporter, OvermapSpecialImporterError, SingleMapDataImporter,
    SingleMapDataImporterError,
};
use crate::features::map::{
    CalculateParameters, CalculateParametersError, CalculateRandomParameters,
    GetMappedCDDAIdsError, InstantiatedOvermap, InstantiatedTile,
    MapGen, MappedCDDAId, MAX_MAP_DATA_SIZE,
};
use crate::features::program_data::keybinds::{Keybind, KeybindAction};
use crate::features::viewer::{LiveViewerData, MapViewer};
use crate::impl_serialize_for_error;
use crate::util::{Load, Save, SaveError};
use cdda_lib::serde_vec::serialize_uvec2_as_key;
use cdda_lib::types::CDDAIdentifier;
use futures_lite::StreamExt;
use glam::{IVec3, UVec2};
use log::info;
use rand::Rng;
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
pub const DEFAULT_Z_LEVEL: i32 = 0;
pub type ProjectName = String;
pub type LoadedProjects = HashMap<ProjectName, Project>;

#[derive(Debug, Error)]
pub enum GetLiveViewerDataError {
    #[error(transparent)]
    SingleImporterError(#[from] SingleMapDataImporterError),

    #[error(transparent)]
    OvermapSpecialImporterError(#[from] OvermapSpecialImporterError),
}

impl_serialize_for_error!(GetLiveViewerDataError);

pub async fn get_map_data_collection_from_map_viewer(
    viewer: &MapViewer,
) -> Result<HashMap<ZLevel, Overmap>, GetLiveViewerDataError> {
    info!("Opening Live viewer");

    let map_data_collection = match &viewer.data {
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
#[serde(rename_all = "camelCase")]
pub enum ProjectType {
    MapEditor(MapEditor),
    MapViewer(MapViewer),
}

impl ProjectType {
    pub fn maps(&self) -> &HashMap<ZLevel, Overmap> {
        match self {
            ProjectType::MapEditor(me) => &me.overmaps,
            ProjectType::MapViewer(lv) => &lv.maps,
        }
    }

    pub fn maps_mut(&mut self) -> &mut HashMap<ZLevel, Overmap> {
        match self {
            ProjectType::MapEditor(me) => &mut me.overmaps,
            ProjectType::MapViewer(lv) => &mut lv.maps,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub project_type: ProjectType,
}

impl Project {
    pub fn new(name: String, project_type: ProjectType) -> Self {
        Self { name, project_type }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overmap {
    #[serde(
        serialize_with = "cdda_lib::serde_vec::serialize_hashmap_with_uvec2_keys",
        deserialize_with = "cdda_lib::serde_vec::deserialize_hashmap_with_uvec2_keys"
    )]
    pub maps: HashMap<UVec2, MapGen>,
}

impl Overmap {
    pub fn new(size: MapSize, project_name: String, z: ZLevel) -> Self {
        let size_value = size.value();

        let mut maps = HashMap::new();

        let is_bigger_than_default = size_value.x <= MAX_MAP_DATA_SIZE.x
            || size_value.y <= MAX_MAP_DATA_SIZE.y;

        match is_bigger_than_default {
            false => {
                for y in 0..(size_value.y / MAX_MAP_DATA_SIZE.y) {
                    for x in 0..(size_value.x / MAX_MAP_DATA_SIZE.x) {
                        let mut map_data = MapGen::default();

                        map_data.id = CDDAIdentifier(format!(
                            "{}_{}_{}_{}",
                            project_name.clone(),
                            x,
                            y,
                            z
                        ));

                        maps.insert(UVec2::new(x, y).into(), map_data);
                    }
                }
            },
            true => {
                let mut map_data = MapGen::default();
                map_data.id = CDDAIdentifier(project_name);
                maps.insert(UVec2::new(0, 0).into(), map_data);
            },
        }

        Self { maps }
    }

    // TODO: return correct Error
    pub fn instantiate(
        &self,
        calculate_parameter_strategy: impl CalculateParameters,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Result<InstantiatedOvermap, ()> {
        let mut instantiated_mapgens = HashMap::new();

        for (map_coordinates, mapgen) in self.maps.iter() {
            let instantiated_mapgen = mapgen
                .instantiate(calculate_parameter_strategy.clone(), cdda_data)?;

            instantiated_mapgens
                .insert(map_coordinates.clone(), instantiated_mapgen);
        }

        Ok(InstantiatedOvermap {
            config: Default::default(),
            instantiated_mapgens,
        })
    }

    pub fn map_to_global_cell_coords(
        map_coordinates: &UVec2,
        cell_coordinates: &UVec2,
        z: ZLevel,
    ) -> IVec3 {
        IVec3::new(
            cell_coordinates.x as i32
                + map_coordinates.x as i32 * MAX_MAP_DATA_SIZE.x as i32,
            cell_coordinates.y as i32
                + map_coordinates.y as i32 * MAX_MAP_DATA_SIZE.y as i32,
            z,
        )
    }
}

impl Default for Overmap {
    fn default() -> Self {
        let mut maps = HashMap::new();
        maps.insert(UVec2::ZERO.into(), MapGen::default());
        Self { maps }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
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

impl ProgramConfig {
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

impl Default for ProgramConfig {
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
                Keybind::single("F5")
                    .action(KeybindAction::ReloadMap)
                    .global(),
                Keybind::single("f").action(KeybindAction::Fill),
                Keybind::single("d").action(KeybindAction::Draw),
                Keybind::single("c").action(KeybindAction::ChunkSelect),
            ]),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedProject {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProgramData {
    pub config: ProgramConfig,

    pub recent_projects: HashMap<ProjectName, PathBuf>,
    pub openable_projects: HashMap<ProjectName, PathBuf>,
    pub opened_project: Option<ProjectName>,

    pub available_tilesets: Option<Vec<String>>,
}

impl ProgramData {
    pub fn create_and_open_project(
        &mut self,
        project_name: String,
        path: PathBuf,
    ) {
        self.opened_project = Some(project_name.clone());
        self.openable_projects
            .insert(project_name.clone(), path.clone());
        self.recent_projects.insert(project_name, path.clone());
    }
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

#[derive(Debug, Clone)]
pub struct AdjacentTiles {
    pub top: Option<MappedCDDAId>,
    pub right: Option<MappedCDDAId>,
    pub bottom: Option<MappedCDDAId>,
    pub left: Option<MappedCDDAId>,
}

impl AdjacentTiles {
    pub fn none() -> Self {
        Self {
            top: None,
            right: None,
            bottom: None,
            left: None,
        }
    }
}
