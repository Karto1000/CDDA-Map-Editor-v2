use crate::cdda_data::map_data::{CDDAMapDataIntermediate, OmTerrain};
use crate::cdda_data::overmap::{
    CDDAOvermapSpecial, CDDAOvermapSpecialIntermediate, OvermapSpecialOvermap,
    OvermapSpecialSubType,
};
use crate::cdda_data::IdOrAbstract;
use crate::editor_data::{MapDataCollection, ZLevel};
use crate::map::{MapData, MapDataRotation};
use crate::util::{CDDAIdentifier, Load};
use anyhow::{anyhow, Error};
use glam::UVec2;
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

fn remove_orientation_suffix_and_get_rotation(
    om_id: CDDAIdentifier,
) -> (CDDAIdentifier, MapDataRotation) {
    let mut rotation = MapDataRotation::Deg0;
    let mut final_overmap_id = om_id;

    if let Some(final_id) = final_overmap_id.0.strip_suffix("_north") {
        final_overmap_id = final_id.into();
        rotation = MapDataRotation::Deg0;
    }

    if let Some(final_id) = final_overmap_id.0.strip_suffix("_east") {
        final_overmap_id = final_id.into();
        rotation = MapDataRotation::Deg90;
    }

    if let Some(final_id) = final_overmap_id.0.strip_suffix("_south") {
        final_overmap_id = final_id.into();
        rotation = MapDataRotation::Deg180;
    }

    if let Some(final_id) = final_overmap_id.0.strip_suffix("_west") {
        final_overmap_id = final_id.into();
        rotation = MapDataRotation::Deg270;
    }

    (final_overmap_id, rotation)
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

            let om_specials: Vec<OvermapSpecialOvermap> =
                match overmap_special.ty {
                    OvermapSpecialSubType::Fixed { overmaps, .. } => overmaps,
                    OvermapSpecialSubType::Mutable { .. } => {
                        return Err(anyhow!(
                            "Mutable special overmap not supported"
                        ))
                    },
                };

            let mut importer = MapDataImporter {
                paths: self.mapgen_entry_paths.clone(),
                om_ids: om_specials
                    .clone()
                    .into_iter()
                    .map(|s| {
                        remove_orientation_suffix_and_get_rotation(
                            s.overmap.unwrap_or("null".into()),
                        )
                        .0
                    })
                    .collect(),
            };

            let mut data = importer.load().await?;

            for om_special in om_specials {
                let (final_id, rotation) =
                    remove_orientation_suffix_and_get_rotation(
                        om_special.overmap.unwrap_or("null".into()),
                    );

                let mut map_data = match data.get(&final_id) {
                    None => continue,
                    Some(md) => md.clone(),
                };
                map_data.rotation = rotation;

                match aggregated_map_data.get_mut(&om_special.point.z) {
                    None => {
                        aggregated_map_data.insert(
                            om_special.point.z,
                            MapDataCollection::default(),
                        );
                        let map_data_collection = aggregated_map_data
                            .get_mut(&om_special.point.z)
                            .unwrap();

                        map_data_collection.maps.insert(
                            UVec2::new(
                                om_special.point.x as u32,
                                om_special.point.y as u32,
                            ),
                            map_data,
                        );
                    },
                    Some(s) => {
                        s.maps.insert(
                            UVec2::new(
                                om_special.point.x as u32,
                                om_special.point.y as u32,
                            ),
                            map_data,
                        );
                    },
                }
            }
        }

        Ok(aggregated_map_data)
    }
}
