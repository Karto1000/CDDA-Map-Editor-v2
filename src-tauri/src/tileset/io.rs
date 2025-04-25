use crate::tileset::current_tileset::CurrentTileConfig;
use crate::tileset::legacy_tileset::tile_config::LegacyTileConfig;
use crate::util::Load;
use anyhow::{anyhow, Error};
use std::path::PathBuf;

pub struct TilesheetLoader<Config> {
    pub config: Config,
}

impl<Config> TilesheetLoader<Config> {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

pub struct TileConfigLoader {
    pub path: PathBuf,
}

impl TileConfigLoader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

pub struct TilesheetConfigLoader {
    pub(crate) tileset_path: PathBuf,
}

impl TilesheetConfigLoader {
    pub fn new(tileset_path: PathBuf) -> Self {
        Self { tileset_path }
    }

    pub fn load_serde_value(&mut self) -> Result<serde_json::Value, Error> {
        let legacy_tilesheet: Result<LegacyTileConfig, Error> = self.load();
        if let Ok(val) = legacy_tilesheet {
            return Ok(serde_json::to_value(val)?);
        }

        let current_tilesheet: Result<CurrentTileConfig, Error> = self.load();
        if let Ok(val) = current_tilesheet {
            return Ok(serde_json::to_value(val)?);
        }

        Err(anyhow!("Invalid Data"))
    }
}
