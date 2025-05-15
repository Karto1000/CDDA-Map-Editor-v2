use serde::{Deserialize, Serialize};

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
