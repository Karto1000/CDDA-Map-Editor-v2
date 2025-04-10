use crate::cdda_data::map_data::{CDDAMapData, OmTerrain};
use crate::map_data::MapData;
use crate::util::Load;
use anyhow::anyhow;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct MapDataImporter {
    pub path: PathBuf,
    pub om_terrain: String,
}

impl Load<MapData> for MapDataImporter {
    fn load(&self) -> Result<MapData, anyhow::Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        let importing_map_datas: Vec<CDDAMapData> =
            serde_json::from_reader(reader).map_err(|e| anyhow::Error::from(e))?;

        let importing_map_data = importing_map_datas
            .into_iter()
            .find(|md| match &md.om_terrain {
                OmTerrain::Single(s) => &self.om_terrain == s,
                OmTerrain::Duplicate(duplicate) => {
                    duplicate.iter().find(|d| *d == &self.om_terrain).is_some()
                }
                OmTerrain::Nested(n) => n
                    .iter()
                    .flatten()
                    .find(|s| *s == &self.om_terrain)
                    .is_some(),
            })
            .ok_or(anyhow!("Could not find map data"))?;

        Ok(importing_map_data.into(self.om_terrain.clone()))
    }
}
