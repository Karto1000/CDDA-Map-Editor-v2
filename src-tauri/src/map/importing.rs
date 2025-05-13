use crate::cdda_data::map_data::{CDDAMapDataIntermediate, OmTerrain};
use crate::editor_data::MapDataCollection;
use crate::util::Load;
use anyhow::anyhow;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct MapDataImporter {
    pub path: PathBuf,
    pub om_terrain: String,
}

impl Load<MapDataCollection> for MapDataImporter {
    async fn load(&mut self) -> Result<MapDataCollection, anyhow::Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        let importing_map_datas: Vec<CDDAMapDataIntermediate> =
            serde_json::from_reader::<BufReader<File>, Vec<Value>>(reader)
                .map_err(|e| anyhow::Error::from(e))?
                .into_iter()
                .filter_map(|v: Value| serde_json::from_value::<CDDAMapDataIntermediate>(v).ok())
                .collect();

        // TODO: Handle multiple z-levels
        let map_data = importing_map_datas
            .into_iter()
            .find_map(|mdi| {
                if let Some(update_terrain) = &mdi.update_mapgen_id {
                    return match self.om_terrain == update_terrain.0 {
                        true => Some(mdi.into()),
                        false => None,
                    };
                }

                if let Some(nested_terrain) = &mdi.nested_mapgen_id {
                    return match self.om_terrain == nested_terrain.0 {
                        true => Some(mdi.into()),
                        false => None,
                    };
                }

                if let Some(om_terrain) = &mdi.om_terrain {
                    return match om_terrain {
                        OmTerrain::Single(s) => match &self.om_terrain == s {
                            true => Some(mdi.into()),
                            false => None,
                        },
                        OmTerrain::Duplicate(duplicate) => {
                            match duplicate.iter().find(|d| *d == &self.om_terrain) {
                                Some(_) => Some(mdi.into()),
                                None => None,
                            }
                        }
                        OmTerrain::Nested(n) => {
                            match n.iter().flatten().find(|s| *s == &self.om_terrain) {
                                None => None,
                                Some(_) => Some(mdi.into()),
                            }
                        }
                    };
                };

                None
            })
            .ok_or(anyhow!("Could not find map data"))?;

        Ok(map_data)
    }
}
