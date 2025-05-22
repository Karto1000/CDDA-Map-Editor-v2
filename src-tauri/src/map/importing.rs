use crate::cdda_data::map_data::{
    CDDAMapDataIntermediate, IdCollection, IntoMapDataCollectionError,
    OmTerrain,
};
use crate::cdda_data::overmap::{
    CDDAOvermapSpecial, CDDAOvermapSpecialIntermediate, OvermapSpecialOvermap,
    OvermapSpecialSubType,
};
use crate::editor_data::{MapDataCollection, ZLevel};
use crate::map::{MapData, MapDataRotation};
use crate::util::Load;
use anyhow::{anyhow, Error};
use cdda_lib::types::{CDDAIdentifier, IdOrAbstract};
use glam::UVec2;
use log::warn;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::utils::config::parse::does_supported_file_name_exist;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Error)]
pub enum MapDataImporterError {
    #[error("Could not find file at path {0}")]
    FileNotFound(PathBuf),
    #[error("Could not find map data in given paths")]
    NoMapDataFound,
    #[error("Could not read bytes of file at path {0}")]
    ReadError(PathBuf),
    #[error("The file at {0} is not a valid CDDA json file. CDDA Json files must have a top level array"
    )]
    InvalidJson(PathBuf),
}

pub struct MapDataImporter {
    pub paths: Vec<PathBuf>,
    pub om_ids: Vec<CDDAIdentifier>,
}

impl Load<HashMap<CDDAIdentifier, MapData>, MapDataImporterError>
    for MapDataImporter
{
    async fn load(
        &mut self,
    ) -> Result<HashMap<CDDAIdentifier, MapData>, MapDataImporterError> {
        let mut found_map_datas: HashMap<CDDAIdentifier, MapData> =
            HashMap::new();

        for path in self.paths.iter() {
            let mut file = File::open(path).await.map_err(|e| {
                warn!("{}", e);
                MapDataImporterError::FileNotFound(path.clone())
            })?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.map_err(|e| {
                warn!("{}", e);
                MapDataImporterError::ReadError(path.clone())
            })?;

            let importing_map_datas: Vec<CDDAMapDataIntermediate> =
                serde_json::from_slice::<Vec<Value>>(buf.as_slice())
                    .map_err(|e| {
                        warn!("{}", e);
                        MapDataImporterError::InvalidJson(path.clone())
                    })?
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
                                    match <CDDAMapDataIntermediate as TryInto<
                                        MapDataCollection,
                                    >>::try_into(
                                        mdi
                                    ) {
                                        Ok(mut map_data) => {
                                            match map_data
                                                .maps
                                                .remove(&UVec2::ZERO)
                                            {
                                                None => {
                                                    warn!("Missing map data at 0,0 for duplicate terrain {}", om_id_to_find);
                                                    break;
                                                },
                                                Some(v) => {
                                                    found_map_datas.insert(
                                                        om_id_to_find.clone(),
                                                        v,
                                                    );
                                                },
                                            }
                                        },
                                        Err(e) => {
                                            warn!("{}", e);
                                        },
                                    }

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
                                    match <CDDAMapDataIntermediate as TryInto<
                                        MapDataCollection,
                                    >>::try_into(
                                        mdi
                                    ) {
                                        Ok(mut map_data) => {
                                            match map_data
                                                .maps
                                                .remove(&UVec2::ZERO)
                                            {
                                                None => {
                                                    warn!("Missing map data at 0,0 for duplicate terrain {}", om_id_to_find);
                                                    break;
                                                },
                                                Some(v) => {
                                                    found_map_datas.insert(
                                                        om_id_to_find.clone(),
                                                        v,
                                                    );
                                                },
                                            }
                                        },
                                        Err(e) => {
                                            warn!("{}", e);
                                        },
                                    }
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
                                    match <CDDAMapDataIntermediate as TryInto<
                                        MapDataCollection,
                                    >>::try_into(
                                        mdi
                                    ) {
                                        Ok(map_data) => {
                                            for (k, v) in map_data.maps {
                                                let id_list = match n
                                                    .get(k.y as usize)
                                                {
                                                    None => {
                                                        warn!("Missing nested terrain identifier list for map data {}", om_id_to_find);
                                                        break;
                                                    },
                                                    Some(id_list) => id_list,
                                                };

                                                match id_list.get(k.x as usize)
                                                {
                                                    None => {
                                                        warn!("Missing nested terrain identifier list for map data {}", om_id_to_find);
                                                        break;
                                                    },
                                                    Some(id) => {
                                                        found_map_datas.insert(
                                                            CDDAIdentifier(
                                                                id.clone(),
                                                            ),
                                                            v,
                                                        );
                                                    },
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            warn!("{}", e);
                                        },
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

#[derive(Debug, Error)]
pub enum SingleMapDataImporterError {
    #[error("Could not find file at path {0}")]
    FileNotFound(PathBuf),
    #[error("Could not find map data in given paths")]
    NoMapDataFound,
    #[error("Could not read bytes of file at path {0}")]
    ReadError(PathBuf),
    #[error("The file at {0} is not a valid CDDA json file; {1}")]
    InvalidJson(PathBuf, serde_json::Error),
    #[error(transparent)]
    ImportError(#[from] IntoMapDataCollectionError),
    #[error("The map data {0} at is not valid; {1}")]
    InvalidMapData(CDDAIdentifier, serde_json::Error),
}

pub struct SingleMapDataImporter {
    pub paths: Vec<PathBuf>,
    pub om_terrain: CDDAIdentifier,
}

impl Load<MapDataCollection, SingleMapDataImporterError>
    for SingleMapDataImporter
{
    async fn load(
        &mut self,
    ) -> Result<MapDataCollection, SingleMapDataImporterError> {
        for path in &self.paths {
            let mut file = File::open(path).await.map_err(|e| {
                warn!("{}", e);
                SingleMapDataImporterError::FileNotFound(path.clone())
            })?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.map_err(|e| {
                warn!("{}", e);
                SingleMapDataImporterError::ReadError(path.clone())
            })?;

            let importing_map_data_ids: Vec<(IdCollection, Value)> =
                serde_json::from_slice::<Vec<Value>>(buf.as_slice())
                    .map_err(|e| {
                        warn!("{}", e);
                        SingleMapDataImporterError::InvalidJson(path.clone(), e)
                    })?
                    .into_iter()
                    .filter_map(|v: Value| {
                        let id_collection =
                            serde_json::from_value::<IdCollection>(v.clone())
                                .ok()?;
                        Some((id_collection, v))
                    })
                    .collect();

            for (id_collection, v) in importing_map_data_ids {
                let mdi: Result<CDDAMapDataIntermediate, serde_json::Error> =
                    serde_json::from_value(v);

                if let Some(update_terrain) = &id_collection.update_mapgen_id {
                    if self.om_terrain == *update_terrain {
                        return match mdi {
                            Ok(mdi) => Ok(mdi.try_into()?),
                            Err(e) => {
                                Err(SingleMapDataImporterError::InvalidMapData(
                                    self.om_terrain.clone(),
                                    e,
                                ))
                            },
                        };
                    }
                }

                if let Some(nested_terrain) = &id_collection.nested_mapgen_id {
                    if self.om_terrain == *nested_terrain {
                        return match mdi {
                            Ok(mdi) => Ok(mdi.try_into()?),
                            Err(e) => {
                                Err(SingleMapDataImporterError::InvalidMapData(
                                    self.om_terrain.clone(),
                                    e,
                                ))
                            },
                        };
                    }
                }

                if let Some(om_terrain) = &id_collection.om_terrain {
                    match om_terrain {
                        OmTerrain::Single(s) => {
                            if self.om_terrain == CDDAIdentifier((*s).clone()) {
                                return match mdi {
                                    Ok(mdi) => Ok(mdi.try_into()?),
                                    Err(e) => Err(SingleMapDataImporterError::InvalidMapData(self.om_terrain.clone(), e))
                                };
                            }
                        },
                        OmTerrain::Duplicate(duplicate) => {
                            if duplicate
                                .iter()
                                .find(|d| {
                                    CDDAIdentifier((*d).clone())
                                        == self.om_terrain
                                })
                                .is_some()
                            {
                                return match mdi {
                                    Ok(mdi) => Ok(mdi.try_into()?),
                                    Err(e) => Err(SingleMapDataImporterError::InvalidMapData(self.om_terrain.clone(), e))
                                };
                            }
                        },
                        OmTerrain::Nested(n) => {
                            if n.iter()
                                .flatten()
                                .find(|s| {
                                    CDDAIdentifier((*s).clone())
                                        == self.om_terrain
                                })
                                .is_some()
                            {
                                return match mdi {
                                    Ok(mdi) => Ok(mdi.try_into()?),
                                    Err(e) => Err(SingleMapDataImporterError::InvalidMapData(self.om_terrain.clone(), e))
                                };
                            }
                        },
                    };
                };
            }
        }

        Err(SingleMapDataImporterError::NoMapDataFound)
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

#[derive(Debug, Error)]
pub enum OvermapSpecialImporterError {
    #[error("Could not find file at path {0}")]
    FileNotFound(PathBuf),
    #[error("Could not find overmap special {0} in given paths")]
    NoOvermapSpecialFound(String),
    #[error("Could not read bytes of file at path {0}")]
    ReadError(PathBuf),
    #[error("The file at {0} is not a valid CDDA json file; {1}")]
    InvalidJson(PathBuf, serde_json::Error),
    #[error("Overmap specials with mutable overmaps are not supported")]
    MutableOvermapNotSupported,
    #[error(transparent)]
    ImportError(#[from] MapDataImporterError),
}

pub struct OvermapSpecialImporter {
    pub om_special_id: CDDAIdentifier,
    pub overmap_special_paths: Vec<PathBuf>,
    pub mapgen_entry_paths: Vec<PathBuf>,
}

impl Load<HashMap<ZLevel, MapDataCollection>, OvermapSpecialImporterError>
    for OvermapSpecialImporter
{
    async fn load(
        &mut self,
    ) -> Result<HashMap<ZLevel, MapDataCollection>, OvermapSpecialImporterError>
    {
        let mut aggregated_map_data: HashMap<ZLevel, MapDataCollection> =
            HashMap::new();

        for path in &self.overmap_special_paths {
            let mut file = File::open(path).await.map_err(|e| {
                warn!("{}", e);
                OvermapSpecialImporterError::FileNotFound(path.clone())
            })?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await.map_err(|e| {
                warn!("{}", e);
                OvermapSpecialImporterError::ReadError(path.clone())
            })?;

            let overmap_special: CDDAOvermapSpecialIntermediate =
                serde_json::from_slice::<Vec<Value>>(buf.as_slice())
                    .map_err(|e| {
                        warn!("{}", e);
                        OvermapSpecialImporterError::InvalidJson(path.clone(), e)
                    })?
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
                    .ok_or(OvermapSpecialImporterError::NoOvermapSpecialFound(self.om_special_id.0.clone()))?;

            let overmap_special: CDDAOvermapSpecial = overmap_special.into();

            let om_specials: Vec<OvermapSpecialOvermap> =
                match overmap_special.ty {
                    OvermapSpecialSubType::Fixed { overmaps, .. } => overmaps,
                    OvermapSpecialSubType::Mutable { .. } => return Err(
                        OvermapSpecialImporterError::MutableOvermapNotSupported,
                    ),
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

            let data = importer.load().await?;

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
                            // Safe since we just inserted it
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
