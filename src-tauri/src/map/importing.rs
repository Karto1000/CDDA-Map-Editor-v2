use crate::cdda_data::map_data::{CDDAMapDataIntermediate, OmTerrain};
use crate::cdda_data::overmap::{
    CDDAOvermapSpecial, CDDAOvermapSpecialIntermediate
    , OvermapSpecialSubType,
};
use crate::cdda_data::IdOrAbstract;
use crate::editor_data::{MapDataCollection, ZLevel};
use crate::map::MapData;
use crate::util::{CDDAIdentifier, Load};
use anyhow::{anyhow, Error};
use glam::{IVec3, UVec2};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub struct MapDataImporter {
    pub paths: Vec<PathBuf>,
    pub om_ids: Vec<CDDAIdentifier>,
}

impl Load<HashMap<CDDAIdentifier, MapData>> for MapDataImporter {
    async fn load(
        &mut self,
    ) -> Result<HashMap<CDDAIdentifier, MapData>, Error> {
        let mut found_map_datas: HashMap<CDDAIdentifier, MapData> =
            HashMap::new();

        for path in self.paths.iter() {
            let mut file = File::open(path).await?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;

            let importing_map_datas: Vec<CDDAMapDataIntermediate> =
                serde_json::from_slice::<Vec<Value>>(buf.as_slice())
                    .map_err(|e| anyhow::Error::from(e))?
                    .into_iter()
                    .filter_map(|v: Value| {
                        serde_json::from_value::<CDDAMapDataIntermediate>(v)
                            .ok()
                    })
                    .collect();

            for mdi in importing_map_datas {
                if let Some(om_terrain) = mdi.om_terrain.clone() {
                    for om_id_to_find in self.om_ids.iter() {
                        match &om_terrain {
                            OmTerrain::Single(s) => {
                                if om_id_to_find == &CDDAIdentifier(s.clone()) {
                                    let mut map_data: MapDataCollection =
                                        mdi.into();

                                    found_map_datas.insert(
                                        om_id_to_find.clone(),
                                        map_data
                                            .maps
                                            .remove(&UVec2::ZERO)
                                            .unwrap(),
                                    );
                                    break;
                                }
                            },
                            OmTerrain::Duplicate(duplicate) => {
                                let any_matches = duplicate
                                    .iter()
                                    .find(|d| {
                                        &CDDAIdentifier((*d).clone())
                                            == om_id_to_find
                                    })
                                    .is_some();

                                if any_matches {
                                    let mut map_data: MapDataCollection =
                                        mdi.into();

                                    found_map_datas.insert(
                                        om_id_to_find.clone(),
                                        map_data
                                            .maps
                                            .remove(&UVec2::ZERO)
                                            .unwrap(),
                                    );
                                    break;
                                }
                            },
                            OmTerrain::Nested(n) => {
                                let any_matches = n
                                    .iter()
                                    .flatten()
                                    .find(|s| {
                                        &CDDAIdentifier((*s).clone())
                                            == om_id_to_find
                                    })
                                    .is_some();

                                if any_matches {
                                    let map_data: MapDataCollection =
                                        mdi.into();

                                    for (k, v) in map_data.maps {
                                        let id = n
                                            .get(k.y as usize)
                                            .unwrap()
                                            .get(k.x as usize)
                                            .unwrap()
                                            .clone();

                                        found_map_datas
                                            .insert(CDDAIdentifier(id), v);
                                    }

                                    break;
                                }
                            },
                        };
                    }
                }
            }
        }

        Ok(found_map_datas)
    }
}

pub struct SingleMapDataImporter {
    pub paths: Vec<PathBuf>,
    pub om_terrain: CDDAIdentifier,
}

impl Load<MapDataCollection> for SingleMapDataImporter {
    async fn load(&mut self) -> Result<MapDataCollection, Error> {
        for path in &self.paths {
            let mut file = File::open(path).await?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;

            let importing_map_datas: Vec<CDDAMapDataIntermediate> =
                serde_json::from_slice::<Vec<Value>>(buf.as_slice())
                    .map_err(|e| anyhow::Error::from(e))?
                    .into_iter()
                    .filter_map(|v: Value| {
                        serde_json::from_value::<CDDAMapDataIntermediate>(v)
                            .ok()
                    })
                    .collect();

            let map_data = importing_map_datas
                .into_iter()
                .find_map(|mdi| {
                    if let Some(update_terrain) = &mdi.update_mapgen_id {
                        return match self.om_terrain == *update_terrain {
                            true => Some(mdi.into()),
                            false => None,
                        };
                    }

                    if let Some(nested_terrain) = &mdi.nested_mapgen_id {
                        return match self.om_terrain == *nested_terrain {
                            true => Some(mdi.into()),
                            false => None,
                        };
                    }

                    if let Some(om_terrain) = &mdi.om_terrain {
                        return match om_terrain {
                            OmTerrain::Single(s) => match self.om_terrain
                                == CDDAIdentifier((*s).clone())
                            {
                                true => Some(mdi.into()),
                                false => None,
                            },
                            OmTerrain::Duplicate(duplicate) => {
                                match duplicate.iter().find(|d| {
                                    CDDAIdentifier((*d).clone())
                                        == self.om_terrain
                                }) {
                                    Some(_) => Some(mdi.into()),
                                    None => None,
                                }
                            },
                            OmTerrain::Nested(n) => {
                                match n.iter().flatten().find(|s| {
                                    CDDAIdentifier((*s).clone())
                                        == self.om_terrain
                                }) {
                                    None => None,
                                    Some(_) => Some(mdi.into()),
                                }
                            },
                        };
                    };

                    None
                })
                .ok_or(anyhow!("Could not find map data"))?;

            return Ok(map_data);
        }

        Err(anyhow!("No map data found"))
    }
}

pub struct OvermapSpecialImporter {
    pub om_special_id: CDDAIdentifier,
    pub overmap_special_paths: Vec<PathBuf>,
    pub mapgen_entry_paths: Vec<PathBuf>,
}

impl Load<HashMap<ZLevel, MapDataCollection>> for OvermapSpecialImporter {
    async fn load(
        &mut self,
    ) -> Result<HashMap<ZLevel, MapDataCollection>, Error> {
        let mut aggregated_map_data: HashMap<ZLevel, MapDataCollection> =
            HashMap::new();

        for path in &self.overmap_special_paths {
            let mut file = File::open(path).await?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;

            let overmap_special: CDDAOvermapSpecialIntermediate =
                serde_json::from_slice::<Vec<Value>>(buf.as_slice())
                    .map_err(|e| anyhow::Error::from(e))?
                    .into_iter()
                    .filter_map(|v: Value| {
                        serde_json::from_value::<CDDAOvermapSpecialIntermediate>(v)
                            .ok()
                    })
                    .find_map(|t| match &t.identifier {
                        IdOrAbstract::Id(id) => {
                            if id == &self.om_special_id {
                                return Some(t);
                            }

                            None
                        }
                        IdOrAbstract::Abstract(_) => None,
                    })
                    .ok_or(anyhow!(
                    "Overmap special {} not found",
                    self.om_special_id
                ))?;

            let overmap_special: CDDAOvermapSpecial = overmap_special.into();

            let om_specials: Vec<(IVec3, CDDAIdentifier)> =
                match overmap_special.ty {
                    OvermapSpecialSubType::Fixed { overmaps, .. } => overmaps
                        .into_iter()
                        .map(|om| {
                            let mut new_om_id =
                                om.overmap.unwrap_or("null".into());

                            if new_om_id.0.ends_with("_north") {
                                new_om_id = new_om_id
                                    .0
                                    .strip_suffix("_north")
                                    .unwrap()
                                    .into();
                            }

                            if new_om_id.0.ends_with("_east") {
                                new_om_id = new_om_id
                                    .0
                                    .strip_suffix("_east")
                                    .unwrap()
                                    .into();
                            }

                            if new_om_id.0.ends_with("_south") {
                                new_om_id = new_om_id
                                    .0
                                    .strip_suffix("_south")
                                    .unwrap()
                                    .into();
                            }

                            if new_om_id.0.ends_with("_west") {
                                new_om_id = new_om_id
                                    .0
                                    .strip_suffix("_west")
                                    .unwrap()
                                    .into();
                            }

                            (om.point, new_om_id)
                        })
                        .collect(),
                    OvermapSpecialSubType::Mutable { .. } => {
                        return Err(anyhow!(
                            "Mutable special overmap not supported"
                        ))
                    },
                };

            let mut importer = MapDataImporter {
                paths: self.mapgen_entry_paths.clone(),
                om_ids: om_specials.iter().map(|s| s.1.clone()).collect(),
            };

            let mut data = importer.load().await?;

            for (point, id) in om_specials {
                let map_data = match data.remove(&id) {
                    None => continue,
                    Some(md) => md,
                };

                match aggregated_map_data.get_mut(&point.z) {
                    None => {
                        aggregated_map_data
                            .insert(point.z, MapDataCollection::default());
                        let map_data_collection =
                            aggregated_map_data.get_mut(&point.z).unwrap();

                        map_data_collection.maps.insert(
                            UVec2::new(point.x as u32, point.y as u32),
                            map_data,
                        );
                    },
                    Some(s) => {
                        s.maps.insert(
                            UVec2::new(point.x as u32, point.y as u32),
                            map_data,
                        );
                    },
                }
            }
        }

        Ok(aggregated_map_data)
    }
}
