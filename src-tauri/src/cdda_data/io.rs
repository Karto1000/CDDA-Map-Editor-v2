use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::map_data::{
    CDDAMapData, CDDAMapDataObject, OmTerrain, DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH,
};
use crate::cdda_data::palettes::CDDAPalette;
use crate::cdda_data::region_settings::CDDARegionSettings;
use crate::cdda_data::terrain::CDDATerrain;
use crate::cdda_data::CDDAJsonEntry;
use crate::util::{CDDAIdentifier, Load};
use anyhow::Error;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Default)]
pub struct DeserializedCDDAJsonData {
    pub palettes: HashMap<CDDAIdentifier, CDDAPalette>,
    pub mapgens: HashMap<CDDAIdentifier, CDDAMapData>,
    pub region_settings: HashMap<CDDAIdentifier, CDDARegionSettings>,
    pub terrain: HashMap<CDDAIdentifier, CDDATerrain>,
    pub furniture: HashMap<CDDAIdentifier, CDDAFurniture>,
}

pub struct CDDADataLoader {
    pub json_path: PathBuf,
}

impl Load<DeserializedCDDAJsonData> for CDDADataLoader {
    fn load(&self) -> Result<DeserializedCDDAJsonData, Error> {
        let walkdir = WalkDir::new(&self.json_path);

        let mut cdda_data = DeserializedCDDAJsonData::default();

        for entry in walkdir {
            let entry = entry?;

            let extension = match entry.path().extension() {
                None => {
                    info!(
                        "Skipping entry {:?} because it does not have an extension",
                        entry.path()
                    );
                    continue;
                }
                Some(e) => e,
            };

            if extension != "json" {
                info!("Skipping {:?} because it is not a json file", entry.path());
                continue;
            }

            info!("Reading and parsing json file at {:?}", entry.path());
            let reader = BufReader::new(File::open(entry.path())?);

            let des = match serde_json::from_reader::<BufReader<File>, Vec<CDDAJsonEntry>>(reader) {
                Ok(des) => des,
                Err(e) => {
                    error!("Failed to deserialize {:?}, error: {}", entry.path(), e);
                    continue;
                }
            };

            for des_entry in des {
                match des_entry {
                    CDDAJsonEntry::Mapgen(mg) => match &mg.om_terrain {
                        OmTerrain::Single(id) => {
                            debug!("Found Single Mapgen '{}' in {:?}", id, entry.path());
                            cdda_data
                                .mapgens
                                .insert(CDDAIdentifier(id.clone()), mg.clone());
                        }
                        OmTerrain::Duplicate(duplicate) => {
                            debug!(
                                "Found Duplicate Mapgen '{:?}' in {:?}",
                                duplicate,
                                entry.path()
                            );
                            for id in duplicate.iter() {
                                cdda_data
                                    .mapgens
                                    .insert(CDDAIdentifier(id.clone()), mg.clone());
                            }
                        }
                        OmTerrain::Nested(nested) => {
                            debug!("Found Nested Mapgen '{:?}' in {:?}", nested, entry.path());

                            for (row, vec) in nested.iter().enumerate() {
                                for (column, om_terrain) in vec.iter().enumerate() {
                                    let rows = mg
                                        .object
                                        .rows
                                        // Get correct range of rows for this om_terrain from row..row + DEFAULT_MAP_HEIGHT
                                        .get(
                                            row * DEFAULT_MAP_HEIGHT
                                                ..row * DEFAULT_MAP_HEIGHT + DEFAULT_MAP_HEIGHT,
                                        )
                                        .expect("Row to not be out of bounds")
                                        .iter()
                                        .map(|colstring| {
                                            colstring
                                                .chars()
                                                .skip(column * DEFAULT_MAP_WIDTH)
                                                .take(
                                                    column * DEFAULT_MAP_WIDTH + DEFAULT_MAP_WIDTH,
                                                )
                                                .collect()
                                        })
                                        .collect();

                                    let mapgen = CDDAMapData {
                                        method: mg.method.clone(),
                                        om_terrain: OmTerrain::Single(om_terrain.clone()),
                                        weight: mg.weight.clone(),
                                        object: CDDAMapDataObject {
                                            fill_ter: mg.object.fill_ter.clone(),
                                            rows,
                                            palettes: mg.object.palettes.clone(),
                                            terrain: mg.object.terrain.clone(),
                                            furniture: mg.object.furniture.clone(),
                                            parameters: mg.object.parameters.clone(),
                                        },
                                    };

                                    cdda_data
                                        .mapgens
                                        .insert(CDDAIdentifier(om_terrain.clone()), mapgen);
                                }
                            }
                        }
                    },
                    CDDAJsonEntry::RegionSettings(rs) => {
                        debug!("Found Region setting {} in {:?}", rs.id, entry.path());
                        cdda_data.region_settings.insert(rs.id.clone(), rs);
                    }
                    CDDAJsonEntry::Palette(p) => {
                        debug!("Found Palette {} in {:?}", p.id, entry.path());
                        cdda_data.palettes.insert(p.id.clone(), p);
                    }
                    CDDAJsonEntry::Terrain(terrain) => {
                        debug!("Found Terrain entry {} in {:?}", terrain.id, entry.path());
                        cdda_data.terrain.insert(terrain.id.clone(), terrain);
                    }
                    CDDAJsonEntry::Furniture(furniture) => {
                        debug!(
                            "Found Furniture entry {} in {:?}",
                            furniture.id,
                            entry.path()
                        );
                        cdda_data.furniture.insert(furniture.id.clone(), furniture);
                    }
                    _ => {
                        info!("Unused JSON entry in {:?}", entry.path());
                    }
                }
            }
        }

        Ok(cdda_data)
    }
}

mod tests {
    use crate::cdda_data::io::CDDADataLoader;
    use crate::util::Load;
    use std::path::PathBuf;

    const CDDA_TEST_JSON_PATH: &'static str = r"C:\CDDA\testing\data\json";

    #[test]
    fn test_load_cdda_data() {
        let data_loader = CDDADataLoader {
            json_path: PathBuf::from(CDDA_TEST_JSON_PATH),
        };

        data_loader.load().expect("Loading to not fail");
    }
}
