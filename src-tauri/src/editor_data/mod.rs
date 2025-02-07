pub(crate) mod handlers;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::Theme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub cdda_path: Option<PathBuf>,
    pub selected_tileset: Option<String>,
    pub theme: Theme,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            cdda_path: None,
            selected_tileset: None,
            theme: Theme::Dark,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorData {
    pub config: EditorConfig,
}
