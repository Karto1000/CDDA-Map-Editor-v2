use crate::data::furniture::CDDAFurniture;
use crate::data::item::CDDAItemGroup;
use crate::data::map_data::OmTerrain;
use crate::data::monster::CDDAMonster;
use crate::data::monster_group::CDDAMonsterGroup;
use crate::data::overmap::{
    CDDAOvermapLocation, CDDAOvermapSpecial, CDDAOvermapTerrain,
};
use crate::data::palettes::CDDAPalette;
use crate::data::region_settings::CDDARegionSettings;
use crate::data::terrain::CDDATerrain;
use crate::data::vehicle_parts::CDDAVehiclePart;
use crate::data::vehicles::CDDAVehicle;
use crate::data::{CDDAJsonEntry, TileLayer};
use crate::features::map::MapData;
use crate::features::program_data::{EditorData, MapDataCollection};
use crate::util::Load;
use anyhow::Error;
use async_walkdir::WalkDir;
use cdda_lib::types::{
    CDDAIdentifier, DistributionInner, ImportCDDAObject, MeabyVec,
};
use cdda_lib::{NULL_FURNITURE, NULL_TERRAIN};
use directories::ProjectDirs;
use futures_lite::stream::StreamExt;
use glam::UVec2;
use log::kv::Source;
use log::{debug, error, info, warn};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::string::ToString;
use thiserror::Error;

#[derive(Default, Serialize, Clone)]
pub struct DeserializedCDDAJsonData {
    pub palettes: HashMap<CDDAIdentifier, CDDAPalette>,
    pub map_data: HashMap<CDDAIdentifier, MapData>,
    pub region_settings: HashMap<CDDAIdentifier, CDDARegionSettings>,
    pub terrain: HashMap<CDDAIdentifier, CDDATerrain>,
    pub furniture: HashMap<CDDAIdentifier, CDDAFurniture>,
    pub item_groups: HashMap<CDDAIdentifier, CDDAItemGroup>,
    pub overmap_locations: HashMap<CDDAIdentifier, CDDAOvermapLocation>,
    pub overmap_terrains: HashMap<CDDAIdentifier, CDDAOvermapTerrain>,
    pub overmap_specials: HashMap<CDDAIdentifier, CDDAOvermapSpecial>,
    pub vehicles: HashMap<CDDAIdentifier, CDDAVehicle>,
    pub vehicle_parts: HashMap<CDDAIdentifier, CDDAVehiclePart>,
    pub monster_groups: HashMap<CDDAIdentifier, CDDAMonsterGroup>,
    pub monsters: HashMap<CDDAIdentifier, CDDAMonster>,
}

#[derive(Debug, Error)]
pub enum GetConnectGroupsError {
    #[error("Terrain for {0} does not exist")]
    NoTerrain(CDDAIdentifier),

    #[error("Furniture for {0} does not exist")]
    NoFurniture(CDDAIdentifier),

    #[error("CDDA entry with id {0} does not have any connect groups")]
    NoConnectGroups(CDDAIdentifier),
}

#[derive(Debug, Error)]
pub enum GetFlagsError {
    #[error("Terrain for {0} does not exist")]
    NoTerrain(CDDAIdentifier),

    #[error("Furniture for {0} does not exist")]
    NoFurniture(CDDAIdentifier),

    #[error("CDDA entry with id {0} does not have any flags")]
    NoFlags(CDDAIdentifier),
}

#[derive(Debug, Error)]
pub enum GetConnectsToError {
    #[error("Terrain for {0} does not exist")]
    NoTerrain(CDDAIdentifier),

    #[error("Furniture for {0} does not exist")]
    NoFurniture(CDDAIdentifier),

    #[error("CDDA entry with id {0} does not have any connect to mappings")]
    NoConnectsTo(CDDAIdentifier),
}

impl DeserializedCDDAJsonData {
    pub fn get_connect_groups(
        &self,
        id: CDDAIdentifier,
        layer: &TileLayer,
    ) -> Result<HashSet<CDDAIdentifier>, GetConnectGroupsError> {
        match layer {
            TileLayer::Terrain => {
                // TODO: Figure out what to do when terrain does not exist
                if id == CDDAIdentifier(NULL_TERRAIN.to_string()) {
                    return Ok(HashSet::new());
                };

                let id = self
                    .terrain
                    .get(&id)
                    .ok_or(GetConnectGroupsError::NoTerrain(id.clone()))?;

                Ok(id
                    .connect_groups
                    .clone()
                    .map(|cg| HashSet::from_iter(cg.into_vec()))
                    .unwrap_or_default())
            },
            TileLayer::Furniture => {
                if id == CDDAIdentifier(NULL_FURNITURE.to_string()) {
                    return Ok(HashSet::new());
                };

                let id = self
                    .furniture
                    .get(&id)
                    .ok_or(GetConnectGroupsError::NoFurniture(id.clone()))?;

                Ok(id
                    .connect_groups
                    .clone()
                    .map(|cg| HashSet::from_iter(cg.into_vec()))
                    .unwrap_or_default())
            },
            _ => Err(GetConnectGroupsError::NoConnectGroups(id.clone())),
        }
    }

    pub fn get_flags(
        &self,
        id: CDDAIdentifier,
        layer: &TileLayer,
    ) -> Result<Vec<String>, GetFlagsError> {
        match layer {
            TileLayer::Terrain => {
                if id == CDDAIdentifier(NULL_TERRAIN.to_string()) {
                    return Ok(vec![]);
                };

                let terrain = self
                    .terrain
                    .get(&id)
                    .ok_or(GetFlagsError::NoTerrain(id.clone()))?;

                Ok(terrain.flags.clone())
            },
            TileLayer::Furniture => {
                if id == CDDAIdentifier(NULL_FURNITURE.to_string()) {
                    return Ok(vec![]);
                };

                let furniture = self
                    .furniture
                    .get(&id)
                    .ok_or(GetFlagsError::NoFurniture(id.clone()))?;

                Ok(furniture.flags.clone())
            },
            _ => Err(GetFlagsError::NoFlags(id.clone())),
        }
    }

    pub fn get_connects_to(
        &self,
        id: CDDAIdentifier,
        layer: &TileLayer,
    ) -> Result<HashSet<CDDAIdentifier>, GetConnectsToError> {
        match layer {
            TileLayer::Terrain => {
                // TODO: Figure out what to do when terrain does not exist
                // TODO: Handle Season specific ids
                let id = self
                    .terrain
                    .get(&id)
                    .ok_or(GetConnectsToError::NoTerrain(id.clone()))?;

                Ok(id
                    .connects_to
                    .clone()
                    .map(|cg| HashSet::from_iter(cg.into_vec()))
                    .unwrap_or_default())
            },
            TileLayer::Furniture => {
                let id = self
                    .furniture
                    .get(&id)
                    .ok_or(GetConnectsToError::NoFurniture(id.clone()))?;
                Ok(id
                    .connects_to
                    .clone()
                    .map(|cg| HashSet::from_iter(cg.into_vec()))
                    .unwrap_or_default())
            },
            _ => Err(GetConnectsToError::NoConnectsTo(id.clone())),
        }
    }

    pub fn add_hardcoded_map_data(&mut self) {
        // TODO: Implement this
        // { "forest",           &mapgen_forest },
        // { "river_curved_not", &mapgen_river_curved_not },
        // { "river_straight",   &mapgen_river_straight },
        // { "river_curved",     &mapgen_river_curved },
        // { "subway_straight",    &mapgen_subway },
        // { "subway_curved",      &mapgen_subway },
        // { "subway_end",         &mapgen_subway },
        // { "subway_tee",         &mapgen_subway },
        // { "subway_four_way",    &mapgen_subway },
        // { "lake_shore", &mapgen_lake_shore },
        // { "ocean_shore", &mapgen_ocean_shore },
        // { "ravine_edge", &mapgen_ravine_edge },

        let mut forest = MapData::default();
        forest.fill =
            Some(DistributionInner::Normal("t_region_groundcover".into()));
        self.map_data.insert("forest".into(), forest);

        let mut river_curved_not = MapData::default();
        river_curved_not.fill =
            Some(DistributionInner::Normal("t_water".into()));
        self.map_data
            .insert("river_curved_not".into(), river_curved_not);

        let mut river_straight = MapData::default();
        river_straight.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data
            .insert("river_straight".into(), river_straight);

        let mut river_curved = MapData::default();
        river_curved.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("river_curved".into(), river_curved);

        let mut subway_straight = MapData::default();
        subway_straight.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data
            .insert("subway_straight".into(), subway_straight);

        let mut subway_curved = MapData::default();
        subway_curved.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data.insert("subway_curved".into(), subway_curved);

        let mut subway_end = MapData::default();
        subway_end.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data.insert("subway_end".into(), subway_end);

        let mut subway_tee = MapData::default();
        subway_tee.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data.insert("subway_tee".into(), subway_tee);

        let mut subway_four_way = MapData::default();
        subway_four_way.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data
            .insert("subway_four_way".into(), subway_four_way);

        let mut lake_shore = MapData::default();
        lake_shore.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("lake_shore".into(), lake_shore);

        let mut ocean_shore = MapData::default();
        ocean_shore.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("ocean_shore".into(), ocean_shore);

        let mut ravine_edge = MapData::default();
        ravine_edge.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("ravine_edge".into(), ravine_edge);
    }
}

pub struct CDDADataLoader {
    pub json_path: PathBuf,
}

impl Load<DeserializedCDDAJsonData> for CDDADataLoader {
    async fn load(&mut self) -> Result<DeserializedCDDAJsonData, Error> {
        let mut walkdir = WalkDir::new(&self.json_path);

        let mut cdda_data = DeserializedCDDAJsonData::default();
        cdda_data.add_hardcoded_map_data();

        let mut intermediate_vehicles = HashMap::new();
        let mut intermediate_vehicle_parts = HashMap::new();
        let mut intermediate_terrains = HashMap::new();
        let mut intermediate_furnitures = HashMap::new();
        let mut intermediate_overmap_locations = HashMap::new();
        let mut intermediate_overmap_terrains = HashMap::new();
        let mut intermediate_overmap_specials = HashMap::new();
        let mut intermediate_monster_groups = HashMap::new();

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

                                    let mut map_data_collection: MapDataCollection = mapgen.try_into()?;

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
                                        mapgen.try_into()?;

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
                                        mapgen.try_into()?;

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
                                mapgen.try_into()?;

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
                                mapgen.try_into()?;

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
                        for ident in terrain.id.clone().into_vec() {
                            debug!(
                                "Found Terrain entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = terrain.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_terrains.insert(ident, clone);
                        }
                    },
                    CDDAJsonEntry::Furniture(furniture) => {
                        for ident in furniture.id.clone().into_vec() {
                            debug!(
                                "Found Furniture entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = furniture.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_furnitures.insert(ident, clone);
                        }
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
                        for ident in group.id.clone().into_vec() {
                            debug!(
                                "Found MonsterGroup entry {} in {:?}",
                                ident,
                                entry.path()
                            );

                            let mut clone = group.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_monster_groups.insert(ident, clone);
                        }
                    },
                    CDDAJsonEntry::OvermapLocation(location) => {
                        for ident in location.id.clone().into_vec() {
                            debug!(
                                "Found OvermapLocation entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = location.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_overmap_locations.insert(ident, clone);
                        }
                    },
                    CDDAJsonEntry::OvermapTerrain(terrain) => {
                        for ident in terrain.id.clone().into_vec() {
                            debug!(
                                "Found OvermapTerrain entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = terrain.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_overmap_terrains.insert(ident, clone);
                        }
                    },
                    CDDAJsonEntry::OvermapSpecial(s) => {
                        for ident in s.id.clone().into_vec() {
                            debug!(
                                "Found OvermapSpecial entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = s.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_overmap_specials.insert(ident, clone);
                        }
                    },
                    CDDAJsonEntry::Vehicle(v) => {
                        for ident in v.id.clone().into_vec() {
                            debug!(
                                "Found Vehicle entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = v.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_vehicles.insert(ident, clone);
                        }
                    },
                    CDDAJsonEntry::VehiclePart(vp) => {
                        for ident in vp.id.clone().into_vec() {
                            debug!(
                                "Found VehiclePart entry {} in {:?}",
                                &ident,
                                entry.path()
                            );

                            let mut clone = vp.clone();
                            clone.id = MeabyVec::Single(ident.clone());

                            intermediate_vehicle_parts.insert(ident, clone);
                        }
                    },
                    _ => {
                        info!("Unused JSON entry in {:?}", entry.path());
                    },
                }
            }
        }

        for (id, intermediate_vehicle) in intermediate_vehicles.iter() {
            cdda_data.vehicles.insert(
                id.clone(),
                intermediate_vehicle
                    .calculate_copy(&intermediate_vehicles)
                    .into(),
            );
        }

        for (id, intermediate_vehicle_part) in intermediate_vehicle_parts.iter()
        {
            cdda_data.vehicle_parts.insert(
                id.clone(),
                intermediate_vehicle_part
                    .calculate_copy(&intermediate_vehicle_parts)
                    .into(),
            );
        }

        for (id, intermediate_terrain) in intermediate_terrains.iter() {
            cdda_data.terrain.insert(
                id.clone(),
                intermediate_terrain
                    .calculate_copy(&intermediate_terrains)
                    .into(),
            );
        }

        for (id, intermediate_furniture) in intermediate_furnitures.iter() {
            cdda_data.furniture.insert(
                id.clone(),
                intermediate_furniture
                    .calculate_copy(&intermediate_furnitures)
                    .into(),
            );
        }

        for (id, intermediate_overmap_location) in
            intermediate_overmap_locations.iter()
        {
            cdda_data.overmap_locations.insert(
                id.clone(),
                intermediate_overmap_location
                    .calculate_copy(&intermediate_overmap_locations)
                    .into(),
            );
        }

        for (id, intermediate_overmap_terrain) in
            intermediate_overmap_terrains.iter()
        {
            cdda_data.overmap_terrains.insert(
                id.clone(),
                intermediate_overmap_terrain
                    .calculate_copy(&intermediate_overmap_terrains)
                    .into(),
            );
        }

        for (id, intermediate_monster_group) in
            intermediate_monster_groups.iter()
        {
            cdda_data.monster_groups.insert(
                id.clone(),
                intermediate_monster_group
                    .calculate_copy(&intermediate_monster_groups)
                    .into(),
            );
        }

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

pub async fn load_cdda_json_data(
    cdda_path: impl Into<PathBuf>,
    json_data_path: impl Into<PathBuf>,
) -> Result<DeserializedCDDAJsonData, anyhow::Error> {
    let mut data_loader = CDDADataLoader {
        json_path: cdda_path.into().join(json_data_path.into()),
    };

    data_loader.load().await
}

pub fn get_saved_editor_data() -> Result<EditorData, Error> {
    let project_dir = ProjectDirs::from("", "", "CDDA Map Editor");

    let directory_path = match project_dir {
        None => {
            warn!("No valid project directory found, creating data folder application directory instead");
            let app_dir = match std::env::current_dir() {
                Ok(d) => d,
                Err(e) => {
                    error!("{}", e);
                    panic!()
                },
            };

            app_dir
        },
        Some(dir) => {
            let local_dir = dir.config_local_dir();
            info!(
                "Got Path for CDDA-Map-Editor config directory at {:?}",
                local_dir
            );
            local_dir.to_path_buf()
        },
    };

    if !fs::exists(&directory_path).expect("IO Error to not occur") {
        info!(
            "Created CDDA-Map-Editor config directory at {:?}",
            directory_path
        );
        fs::create_dir_all(&directory_path)?;
    }

    let config_file_path = directory_path.join("config.json");
    let config_exists =
        fs::exists(&config_file_path).expect("IO Error to not occur");
    let config = match config_exists {
        true => {
            info!("Reading config.json file");
            let contents = fs::read_to_string(&config_file_path)
                .expect("File to be valid UTF-8");

            let data =
                match serde_json::from_str::<EditorData>(contents.as_str()) {
                    Ok(d) => {
                        info!("config.json file successfully read and parsed");
                        d
                    },
                    Err(e) => {
                        error!("{}", e.to_string());
                        info!(
                        "Error while reading config.json file, recreating file"
                    );

                        let mut default_editor_data = EditorData::default();
                        default_editor_data.config.config_path =
                            directory_path.clone();

                        let serialized =
                            serde_json::to_string_pretty(&default_editor_data)
                                .expect("Serialization to not fail");
                        fs::write(&config_file_path, serialized).expect(
                            "Directory path to config to have been created",
                        );
                        default_editor_data
                    },
                };

            data
        },
        false => {
            info!("config.json file does not exist");
            info!("Creating config.json file with default data");

            let mut default_editor_data = EditorData::default();
            default_editor_data.config.config_path = directory_path.clone();

            let serialized = serde_json::to_string_pretty(&default_editor_data)
                .expect("Serialization to not fail");
            fs::write(&config_file_path, serialized)
                .expect("Directory path to config to have been created");
            default_editor_data
        },
    };

    Ok(config)
}
