use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::item::CDDAItemGroup;
use crate::cdda_data::map_data::OmTerrain;
use crate::cdda_data::monster::CDDAMonsterGroup;
use crate::cdda_data::overmap::{
    CDDAOvermapLocation, CDDAOvermapSpecial, CDDAOvermapTerrain,
};
use crate::cdda_data::palettes::CDDAPalette;
use crate::cdda_data::region_settings::CDDARegionSettings;
use crate::cdda_data::terrain::CDDATerrain;
use crate::cdda_data::{CDDAExtendOp, CDDAJsonEntry, TileLayer};
use crate::editor_data::MapDataCollection;
use crate::map::MapData;
use crate::util::{CDDAIdentifier, Load};
use anyhow::Error;
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use glam::UVec2;
use log::kv::Source;
use log::{debug, error, info, warn};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::string::ToString;

pub const NULL_TERRAIN: &'static str = "t_null";
pub const NULL_FURNITURE: &'static str = "f_null";
pub const NULL_NESTED: &'static str = "null";
pub const NULL_FIELD: &'static str = "fd_null";
pub const NULL_TRAP: &'static str = "tr_null";

#[derive(Default, Serialize, Clone)]
pub struct DeserializedCDDAJsonData {
    pub palettes: HashMap<CDDAIdentifier, CDDAPalette>,
    pub map_data: HashMap<CDDAIdentifier, MapData>,
    pub region_settings: HashMap<CDDAIdentifier, CDDARegionSettings>,
    pub terrain: HashMap<CDDAIdentifier, CDDATerrain>,
    pub furniture: HashMap<CDDAIdentifier, CDDAFurniture>,
    pub item_groups: HashMap<CDDAIdentifier, CDDAItemGroup>,
    pub monstergroups: HashMap<CDDAIdentifier, CDDAMonsterGroup>,
    pub overmap_locations: HashMap<CDDAIdentifier, CDDAOvermapLocation>,
    pub overmap_terrains: HashMap<CDDAIdentifier, CDDAOvermapTerrain>,
    pub overmap_specials: HashMap<CDDAIdentifier, CDDAOvermapSpecial>,
}

impl DeserializedCDDAJsonData {
    pub fn get_connect_groups(
        &self,
        id: Option<CDDAIdentifier>,
        layer: &TileLayer,
    ) -> HashSet<CDDAIdentifier> {
        id.map(|id| {
            match layer {
                TileLayer::Terrain => {
                    // TODO: Figure out what to do when terrain does not exist
                    if id == CDDAIdentifier(NULL_TERRAIN.to_string()) {
                        return HashSet::new();
                    };

                    let id = self.terrain.get(&id).expect(
                        format!("Terrain for {} to exist", id).as_str(),
                    );
                    id.connect_groups
                        .clone()
                        .map(|cg| HashSet::from_iter(cg.into_vec()))
                        .unwrap_or_default()
                },
                TileLayer::Furniture => {
                    if id == CDDAIdentifier(NULL_FURNITURE.to_string()) {
                        return HashSet::new();
                    };

                    let id = self.furniture.get(&id).expect(
                        format!("Furniture for {} to exist", id).as_str(),
                    );
                    id.connect_groups
                        .clone()
                        .map(|cg| HashSet::from_iter(cg.into_vec()))
                        .unwrap_or_default()
                },
                // TODO: I don't know if traps have connect groups, have to check later
                TileLayer::Monster => HashSet::new(),
                TileLayer::Field => HashSet::new(),
            }
        })
        .unwrap_or_default()
    }

    pub fn get_flags(
        &self,
        id: Option<CDDAIdentifier>,
        layer: &TileLayer,
    ) -> Vec<String> {
        id.map(|id| match layer {
            TileLayer::Terrain => {
                if id == CDDAIdentifier(NULL_TERRAIN.to_string()) {
                    return vec![];
                };

                let terrain = self
                    .terrain
                    .get(&id)
                    .expect(format!("Terrain for {} to exist", id).as_str());
                terrain.flags.clone().unwrap_or_default()
            },
            TileLayer::Furniture => {
                if id == CDDAIdentifier(NULL_FURNITURE.to_string()) {
                    return vec![];
                };

                let furniture = self
                    .furniture
                    .get(&id)
                    .expect(format!("Terrain for {} to exist", id).as_str());
                furniture.flags.clone().unwrap_or_default()
            },
            // TODO: Again, not sure if they have flags
            TileLayer::Monster => vec![],
            TileLayer::Field => vec![],
        })
        .unwrap_or_default()
    }

    pub fn get_connects_to(
        &self,
        id: Option<CDDAIdentifier>,
        layer: &TileLayer,
    ) -> HashSet<CDDAIdentifier> {
        id.map(|id| {
            match layer {
                TileLayer::Terrain => {
                    // TODO: Figure out what to do when terrain does not exist
                    // TODO: Handle Season specific ids
                    let id = self.terrain.get(&id).expect(
                        format!("Terrain for {} to exist", id).as_str(),
                    );
                    id.connects_to
                        .clone()
                        .map(|cg| HashSet::from_iter(cg.into_vec()))
                        .unwrap_or_default()
                },
                TileLayer::Furniture => {
                    let id = self.furniture.get(&id).expect(
                        format!("Furniture for {} to exist", id).as_str(),
                    );
                    id.connects_to
                        .clone()
                        .map(|cg| HashSet::from_iter(cg.into_vec()))
                        .unwrap_or_default()
                },
                // TODO: See comments up top
                TileLayer::Monster => HashSet::new(),
                TileLayer::Field => HashSet::new(),
            }
        })
        .unwrap_or_default()
    }

    fn calculate_copy_property_of_overmap_terrain(
        &self,
        overmap_terrain: CDDAOvermapTerrain,
    ) -> CDDAOvermapTerrain {
        match &overmap_terrain.copy_from {
            None => overmap_terrain,
            Some(copy_from_id) => {
                let mut copy_from_terrain =
                    match self.overmap_terrains.get(copy_from_id) {
                        None => {
                            warn!(
                            "Could not copy {} for {} due to it not existing",
                            copy_from_id, overmap_terrain.id
                        );
                            return overmap_terrain;
                        },
                        Some(t) => t.clone(),
                    };

                if copy_from_terrain.copy_from.is_some() {
                    copy_from_terrain = self
                        .calculate_copy_property_of_overmap_terrain(
                            copy_from_terrain,
                        );
                }

                CDDAOvermapTerrain::merge_with_precedence(
                    &copy_from_terrain,
                    &overmap_terrain,
                )
            },
        }
    }

    fn calculate_copy_property_of_terrain(
        &self,
        terrain: CDDATerrain,
    ) -> CDDATerrain {
        match &terrain.copy_from {
            None => terrain,
            Some(copy_from_id) => {
                let mut copy_from_terrain = match self.terrain.get(copy_from_id)
                {
                    None => {
                        warn!(
                            "Could not copy {} for {} due to it not existing",
                            copy_from_id, terrain.id
                        );
                        return terrain;
                    },
                    Some(t) => t.clone(),
                };

                if copy_from_terrain.copy_from.is_some() {
                    copy_from_terrain = self
                        .calculate_copy_property_of_terrain(copy_from_terrain);
                }

                CDDATerrain::merge_with_precedence(&copy_from_terrain, &terrain)
            },
        }
    }

    pub fn calculate_operations(&mut self) {
        let mut updated_terrain: HashMap<CDDAIdentifier, CDDATerrain> =
            HashMap::new();
        for (copy_to_id, to) in self.terrain.iter() {
            let mut new_terrain =
                self.terrain.get(copy_to_id).expect("To Exist").clone();

            new_terrain = self.calculate_copy_property_of_terrain(new_terrain);

            match &to.extend {
                None => {},
                Some(extend) => match &extend.flags {
                    None => {},
                    Some(new_flags) => {
                        let mut old_flags =
                            new_terrain.flags.clone().unwrap_or_default();
                        old_flags.extend(new_flags.clone());
                        new_terrain.flags = Some(old_flags)
                    },
                },
            };

            match &to.delete {
                None => {},
                Some(delete) => match &delete.flags {
                    None => {},
                    Some(new_flags) => {
                        let old_flags =
                            new_terrain.flags.clone().unwrap_or_default();
                        let new_flags = old_flags
                            .into_iter()
                            .filter(|f| {
                                new_flags.iter().find(|nf| *nf == f).is_some()
                            })
                            .collect();
                        new_terrain.flags = Some(new_flags)
                    },
                },
            };

            updated_terrain.insert(copy_to_id.clone(), new_terrain);
        }

        let mut updated_overmap_terrains: HashMap<
            CDDAIdentifier,
            CDDAOvermapTerrain,
        > = HashMap::new();
        for (copy_to_id, to) in self.overmap_terrains.iter() {
            let mut new_terrain = self
                .overmap_terrains
                .get(copy_to_id)
                .expect("To Exist")
                .clone();
            new_terrain =
                self.calculate_copy_property_of_overmap_terrain(new_terrain);

            match &to.extend {
                None => {},
                Some(extend) => match &extend.flags {
                    None => {},
                    Some(new_flags) => {
                        let mut old_flags =
                            new_terrain.flags.clone().unwrap_or_default();
                        old_flags.extend(new_flags.clone());
                        new_terrain.flags = Some(old_flags)
                    },
                },
            }

            match &to.delete {
                None => {},
                Some(delete) => match &delete.flags {
                    None => {},
                    Some(new_flags) => {
                        let old_flags =
                            new_terrain.flags.clone().unwrap_or_default();
                        let new_flags = old_flags
                            .into_iter()
                            .filter(|f| {
                                new_flags.iter().find(|nf| *nf == f).is_some()
                            })
                            .collect();
                        new_terrain.flags = Some(new_flags)
                    },
                },
            };

            updated_overmap_terrains.insert(copy_to_id.clone(), new_terrain);
        }

        self.terrain.extend(updated_terrain);
        self.overmap_terrains.extend(updated_overmap_terrains);
    }
}

pub struct CDDADataLoader {
    pub json_path: PathBuf,
}

impl Load<DeserializedCDDAJsonData> for CDDADataLoader {
    async fn load(&mut self) -> Result<DeserializedCDDAJsonData, Error> {
        let mut walkdir = WalkDir::new(&self.json_path);

        let mut cdda_data = DeserializedCDDAJsonData::default();

        while let Some(entry) = walkdir.next().await {
            let entry = entry?;

            let path = entry.path();
            let extension = match path.extension() {
                None => {
                    info!(
                        "Skipping entry {:?} because it does not have an extension",
                        entry.path()
                    );
                    continue;
                },
                Some(e) => e,
            };

            if extension != "json" {
                info!(
                    "Skipping {:?} because it is not a json file",
                    entry.path()
                );
                continue;
            }

            info!("Reading and parsing json file at {:?}", entry.path());
            let reader = BufReader::new(File::open(entry.path())?);

            let des = match serde_json::from_reader::<
                BufReader<File>,
                Vec<CDDAJsonEntry>,
            >(reader)
            {
                Ok(des) => des,
                Err(e) => {
                    error!(
                        "Failed to deserialize {:?}, error: {}",
                        entry.path(),
                        e
                    );
                    continue;
                },
            };

            for des_entry in des {
                match des_entry {
                    CDDAJsonEntry::Mapgen(mapgen) => {
                        if let Some(om_terrain) = mapgen.om_terrain.clone() {
                            match om_terrain {
                                OmTerrain::Single(id) => {
                                    debug!(
                                        "Found Single Mapgen '{}' in {:?}",
                                        id,
                                        entry.path()
                                    );

                                    let mut map_data_collection: MapDataCollection = mapgen.into();

                                    cdda_data.map_data.insert(
                                        CDDAIdentifier(id.clone()),
                                        map_data_collection
                                            .maps
                                            .remove(&UVec2::ZERO)
                                            .unwrap(),
                                    );
                                },
                                OmTerrain::Duplicate(duplicate) => {
                                    debug!(
                                        "Found Duplicate Mapgen '{:?}' in {:?}",
                                        duplicate,
                                        entry.path()
                                    );

                                    let map_data_collection: MapDataCollection =
                                        mapgen.into();

                                    for id in duplicate.iter() {
                                        cdda_data.map_data.insert(
                                            CDDAIdentifier(id.clone()),
                                            map_data_collection
                                                .maps
                                                .get(&UVec2::ZERO)
                                                .unwrap()
                                                .clone(),
                                        );
                                    }
                                },
                                OmTerrain::Nested(nested) => {
                                    debug!(
                                        "Found Nested Mapgen '{:?}' in {:?}",
                                        nested,
                                        entry.path()
                                    );

                                    let map_data_collection: MapDataCollection =
                                        mapgen.into();

                                    for (coords, map_data) in
                                        map_data_collection.maps
                                    {
                                        let om_terrain = nested
                                            .get(coords.y as usize)
                                            .unwrap()
                                            .get(coords.x as usize)
                                            .unwrap()
                                            .clone();

                                        cdda_data.map_data.insert(
                                            CDDAIdentifier(om_terrain),
                                            map_data,
                                        );
                                    }
                                },
                            }
                        } else if let Some(nested_mapgen) =
                            mapgen.nested_mapgen_id.clone()
                        {
                            debug!(
                                "Found Nested Mapgen Object '{}' in {:?}",
                                nested_mapgen,
                                entry.path()
                            );

                            let mut map_data_collection: MapDataCollection =
                                mapgen.into();

                            cdda_data.map_data.insert(
                                nested_mapgen.clone(),
                                map_data_collection
                                    .maps
                                    .remove(&UVec2::ZERO)
                                    .unwrap(),
                            );
                        } else if let Some(update_mapgen) =
                            mapgen.update_mapgen_id.clone()
                        {
                            debug!(
                                "Found Update Mapgen Object '{:?}' in {:?}",
                                update_mapgen,
                                entry.path()
                            );

                            let mut map_data_collection: MapDataCollection =
                                mapgen.into();

                            cdda_data.map_data.insert(
                                update_mapgen.clone(),
                                map_data_collection
                                    .maps
                                    .remove(&UVec2::ZERO)
                                    .unwrap(),
                            );
                        }
                    },
                    CDDAJsonEntry::RegionSettings(rs) => {
                        debug!(
                            "Found Region setting {} in {:?}",
                            rs.id,
                            entry.path()
                        );
                        cdda_data.region_settings.insert(rs.id.clone(), rs);
                    },
                    CDDAJsonEntry::Palette(p) => {
                        debug!("Found Palette {} in {:?}", p.id, entry.path());
                        cdda_data.palettes.insert(p.id.clone(), p.into());
                    },
                    CDDAJsonEntry::Terrain(terrain) => {
                        let new_terrain: CDDATerrain = terrain.into();
                        debug!(
                            "Found Terrain entry {} in {:?}",
                            new_terrain.id,
                            entry.path()
                        );
                        cdda_data
                            .terrain
                            .insert(new_terrain.id.clone(), new_terrain);
                    },
                    CDDAJsonEntry::Furniture(furniture) => {
                        let new_furniture: CDDAFurniture = furniture.into();
                        debug!(
                            "Found Furniture entry {} in {:?}",
                            new_furniture.id,
                            entry.path()
                        );
                        cdda_data
                            .furniture
                            .insert(new_furniture.id.clone(), new_furniture);
                    },
                    CDDAJsonEntry::ItemGroup(group) => {
                        let new_group: CDDAItemGroup = group.into();
                        debug!(
                            "Found ItemGroup entry {} in {:?}",
                            new_group.id,
                            entry.path()
                        );
                        cdda_data
                            .item_groups
                            .insert(new_group.id.clone(), new_group);
                    },
                    CDDAJsonEntry::MonsterGroup(group) => {
                        debug!(
                            "Found MonsterGroup entry {} in {:?}",
                            group.id,
                            entry.path()
                        );
                        cdda_data.monstergroups.insert(group.id.clone(), group);
                    },
                    CDDAJsonEntry::OvermapLocation(location) => {
                        debug!(
                            "Found OvermapLocation entry {} in {:?}",
                            location.id,
                            entry.path()
                        );
                        cdda_data
                            .overmap_locations
                            .insert(location.id.clone(), location);
                    },
                    CDDAJsonEntry::OvermapTerrain(terrain) => {
                        let terrain_vec: Vec<CDDAOvermapTerrain> =
                            terrain.into();

                        for terrain in terrain_vec {
                            debug!(
                                "Found OvermapTerrain entry {} in {:?}",
                                terrain.id,
                                entry.path()
                            );

                            cdda_data
                                .overmap_terrains
                                .insert(terrain.id.clone(), terrain);
                        }
                    },
                    CDDAJsonEntry::OvermapSpecial(s) => {
                        let special: CDDAOvermapSpecial = s.into();

                        debug!(
                            "Found OvermapSpecial entry {} in {:?}",
                            special.id,
                            entry.path()
                        );

                        cdda_data
                            .overmap_specials
                            .insert(special.id.clone(), special);
                    },
                    _ => {
                        info!("Unused JSON entry in {:?}", entry.path());
                    },
                }
            }
        }

        cdda_data.calculate_operations();

        Ok(cdda_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CDDA_TEST_JSON_PATH: &'static str = r"C:\CDDA\testing\data\json";

    #[test]
    fn test_load_cdda_data() {
        tokio_test::block_on(async {
            let mut data_loader = CDDADataLoader {
                json_path: PathBuf::from(CDDA_TEST_JSON_PATH),
            };

            data_loader.load().await.expect("Loading to not fail");
        })
    }
}
