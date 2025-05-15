use crate::cdda_data::map_data::{CDDAMapDataIntermediate, OmTerrain};
use crate::editor_data::MapDataCollection;
use crate::util::Load;
use anyhow::{anyhow, Error};
use glam::UVec2;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct SingleMapDataImporter {
    pub path: PathBuf,
    pub om_terrain: String,
}

impl Load<MapDataCollection> for SingleMapDataImporter {
    async fn load(&mut self) -> Result<MapDataCollection, Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        let importing_map_datas: Vec<CDDAMapDataIntermediate> =
            serde_json::from_reader::<BufReader<File>, Vec<Value>>(reader)
                .map_err(|e| anyhow::Error::from(e))?
                .into_iter()
                .filter_map(|v: Value| {
                    serde_json::from_value::<CDDAMapDataIntermediate>(v).ok()
                })
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
                            match duplicate
                                .iter()
                                .find(|d| *d == &self.om_terrain)
                            {
                                Some(_) => Some(mdi.into()),
                                None => None,
                            }
                        },
                        OmTerrain::Nested(n) => {
                            match n
                                .iter()
                                .flatten()
                                .find(|s| *s == &self.om_terrain)
                            {
                                None => None,
                                Some(_) => Some(mdi.into()),
                            }
                        },
                    };
                };

                None
            })
            .ok_or(anyhow!("Could not find map data"))?;

        Ok(map_data)
    }
}
pub struct NestedMapDataImporter {
    pub path: PathBuf,
    pub om_terrain_ids: HashMap<String, UVec2>,
}

impl Load<MapDataCollection> for NestedMapDataImporter {
    async fn load(&mut self) -> Result<MapDataCollection, Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        let importing_map_datas: Vec<CDDAMapDataIntermediate> =
            serde_json::from_reader::<BufReader<File>, Vec<Value>>(reader)
                .map_err(|e| anyhow::Error::from(e))?
                .into_iter()
                .filter_map(|v: Value| {
                    serde_json::from_value::<CDDAMapDataIntermediate>(v).ok()
                })
                .collect();

        let mut collection = MapDataCollection::default();

        importing_map_datas.into_iter().for_each(|mdi| {
            if let Some(om_terrain) = &mdi.om_terrain {
                return match om_terrain {
                    OmTerrain::Single(s) => match self.om_terrain_ids.get(s) {
                        Some(pos) => {
                            let mut intermediate: MapDataCollection =
                                mdi.into();

                            collection.maps.insert(
                                pos.clone(),
                                intermediate.maps.remove(&UVec2::ZERO).unwrap(),
                            );
                        },
                        None => {},
                    },
                    OmTerrain::Duplicate(duplicate) => {
                        match duplicate
                            .iter()
                            .filter_map(|d| self.om_terrain_ids.get(d))
                            .next()
                        {
                            Some(pos) => {
                                let mut intermediate: MapDataCollection =
                                    mdi.into();

                                collection.maps.insert(
                                    pos.clone(),
                                    intermediate
                                        .maps
                                        .remove(&UVec2::ZERO)
                                        .unwrap(),
                                );
                            },
                            None => {},
                        }
                    },
                    OmTerrain::Nested(n) => {
                        let mut found_ids = vec![];

                        for (row, id_list) in n.iter().enumerate() {
                            for (col, id) in id_list.iter().enumerate() {
                                match self.om_terrain_ids.get(id) {
                                    Some(pos) => {
                                        found_ids.push((
                                            pos.clone(),
                                            UVec2::new(col as u32, row as u32),
                                        ));
                                    },
                                    None => {},
                                }
                            }
                        }

                        if found_ids.len() > 0 {
                            let mut intermediate: MapDataCollection =
                                mdi.into();

                            for (final_pos, found_pos) in found_ids {
                                collection.maps.insert(
                                    final_pos,
                                    intermediate
                                        .maps
                                        .remove(&found_pos)
                                        .unwrap(),
                                );
                            }
                        }
                    },
                };
            };
        });

        Ok(collection)
    }
}
