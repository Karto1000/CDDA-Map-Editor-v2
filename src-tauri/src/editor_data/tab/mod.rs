pub(crate) mod handlers;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TabType {
    #[default]
    Welcome,
    MapEditor(MapDataState),
    LiveViewer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum MapDataState {
    #[default]
    Unsaved,
    Saved {
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tab {
    pub name: String,
    pub tab_type: TabType,
}
