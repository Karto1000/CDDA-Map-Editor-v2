use crate::tileset::legacy_tileset::tile_config::LegacyTileConfig;
use crate::util::Load;
use anyhow::Error;
use serde_json::Value;
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

    pub async fn load_value(&mut self) -> Result<Value, Error> {
        let legacy_tilesheet =
            <TilesheetConfigLoader as Load<LegacyTileConfig>>::load(self).await;

        match legacy_tilesheet {
            Ok(v) => Ok(serde_json::to_value(v)?),
            Err(e) => {
                anyhow::bail!(e);
            },
        }
    }
}
