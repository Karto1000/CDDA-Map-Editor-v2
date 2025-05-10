pub(crate) mod handlers;
pub(crate) mod tab;

use crate::editor_data::tab::Tab;
use crate::map::{MapData, DEFAULT_MAP_DATA_SIZE};
use crate::util::Save;
use glam::UVec2;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Editor,
    Viewer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub maps: HashMap<ZLevel, MapData>,
    pub size: UVec2,
    pub ty: ProjectType,
}

impl Project {
    pub fn new(name: String, size: UVec2, ty: ProjectType) -> Self {
        let mut maps = HashMap::new();

        maps.insert(0, MapData::default());

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
        maps.insert(0, MapData::default());

        Self {
            name: "Unnamed".to_string(),
            maps,
            size: DEFAULT_MAP_DATA_SIZE,
            ty: ProjectType::Editor,
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
