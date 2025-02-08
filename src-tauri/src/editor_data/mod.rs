pub(crate) mod handlers;

use crate::util::Save;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Error;
use std::path::PathBuf;
use tauri::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub cdda_path: Option<PathBuf>,
    pub config_path: PathBuf,
    pub selected_tileset: Option<String>,
    pub theme: Theme,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            cdda_path: None,
            config_path: Default::default(),
            selected_tileset: None,
            theme: Theme::Dark,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorData {
    pub config: EditorConfig,

    pub available_tilesets: Option<Vec<String>>,
}

pub struct EditorDataSaver {
    pub path: PathBuf,
}

impl Save<EditorData> for EditorDataSaver {
    fn save(&self, data: &EditorData) -> Result<(), Error> {
        let serialized = serde_json::to_string_pretty(data).expect("Serialization to not fail");
        fs::write(self.path.join("config.json"), serialized)?;
        Ok(())
    }
}
