pub(crate) mod handlers;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TabType {
    #[default]
    Welcome,
    MapEditor,
    LiveViewer,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Tab {
    pub name: String,
    pub tab_type: TabType,
}
