use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub(crate) mod handlers;
pub(crate) mod tile_config;

#[derive(Debug, Serialize, Deserialize)]
pub struct TileNew {
    file: String,
    #[serde(rename = "//")]
    range: Option<String>,

    sprite_width: Option<u32>,
    sprite_height: Option<u32>,
    sprite_offset_x: Option<i32>,
    sprite_offset_y: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileInfo {
    pixelscale: u32,
    width: u32,
    height: u32,
    zlevel_height: u32,
    iso: bool,
    retract_dist_min: f32,
    retract_dist_max: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpritesheetConfig {
    #[serde(rename = "tiles-new")]
    tiles_new: Vec<TileNew>,

    tile_info: Vec<TileInfo>,
}

pub struct SpritesheetConfigReader {
    tileset_path: PathBuf,
}

impl SpritesheetConfigReader {
    pub fn new(tileset_path: PathBuf) -> Self {
        Self { tileset_path }
    }

    pub fn read(&self) -> serde_json::Result<SpritesheetConfig> {
        let config_path = self.tileset_path.join("tile_config.json");
        serde_json::from_str(fs::read_to_string(config_path).unwrap().as_str())
    }
}
