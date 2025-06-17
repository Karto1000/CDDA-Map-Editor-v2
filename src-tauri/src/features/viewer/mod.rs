use crate::features::program_data::{MapDataCollection, ZLevel};
use cdda_lib::types::CDDAIdentifier;
use glam::UVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

mod data;
pub mod handlers;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapViewer {
    #[serde(skip)]
    pub maps: HashMap<ZLevel, MapDataCollection>,
    pub data: LiveViewerData,
    pub size: UVec2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
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
