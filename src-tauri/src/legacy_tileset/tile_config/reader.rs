use crate::legacy_tileset::tile_config::TileConfig;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct TileConfigReader {
    pub path: PathBuf,
}

impl TileConfigReader {
    pub fn read(&self) -> Result<TileConfig, anyhow::Error> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| anyhow::anyhow!(e))
    }
}
