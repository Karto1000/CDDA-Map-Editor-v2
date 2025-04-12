use crate::legacy_tileset::SpritesheetConfig;
use std::fs;
use std::path::PathBuf;

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
