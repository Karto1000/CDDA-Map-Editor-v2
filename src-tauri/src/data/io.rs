use crate::data::furniture::{CDDAFurniture, CDDAFurnitureIntermediate};
use crate::data::item::CDDAItemGroup;
use crate::data::map_data::{CDDAMapDataIntermediate, OmTerrain};
use crate::data::monster::CDDAMonster;
use crate::data::monster_group::{
    CDDAMonsterGroup, CDDAMonsterGroupIntermediate,
};
use crate::data::overmap::{
    CDDAOvermapLocation, CDDAOvermapLocationIntermediate, CDDAOvermapSpecial,
    CDDAOvermapSpecialIntermediate, CDDAOvermapTerrain,
    CDDAOvermapTerrainIntermediate,
};
use crate::data::palettes::{CDDAPalette, CDDAPaletteIntermediate};
use crate::data::region_settings::CDDARegionSettings;
use crate::data::terrain::{CDDATerrain, CDDATerrainIntermediate};
use crate::data::vehicle_parts::{
    CDDAVehiclePart, CDDAVehiclePartIntermediate,
};
use crate::data::vehicles::{CDDAVehicle, CDDAVehicleIntermediate};
use crate::data::{replace_region_setting, CDDAJsonEntry, TileLayer};
use crate::features::map::{GetMappedCDDAIdsError, MapGen};
use crate::features::program_data::io::{ProgramDataLoader, ProjectLoader};
use crate::features::program_data::{Overmap, ProgramData, Project};
use crate::util::Load;
use anyhow::{anyhow, Error};
use async_walkdir::WalkDir;
use cdda_lib::types::{
    CDDAIdentifier, DistributionInner, ImportCDDAObject, MeabyVec,
};
use cdda_lib::{DEFAULT_REGION_SETTING_ENTRY, NULL_FURNITURE, NULL_TERRAIN};
use directories::ProjectDirs;
use futures_lite::stream::StreamExt;
use glam::UVec2;
use log::kv::Source;
use log::{debug, error, info, warn};
use rand::RngCore;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::string::ToString;
use thiserror::Error;
use thiserror::__private::AsDisplay;

#[derive(Default, Serialize, Clone)]
pub struct DeserializedCDDAJsonData {
    pub palettes: HashMap<CDDAIdentifier, CDDAPalette>,
    pub map_data: HashMap<CDDAIdentifier, MapGen>,
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
    pub fn replace_possible_region_settings(
        &self,
        rng: &mut dyn RngCore,
        id: CDDAIdentifier,
    ) -> CDDAIdentifier {
        match self
            .region_settings
            .get(&CDDAIdentifier(DEFAULT_REGION_SETTING_ENTRY.into()))
        {
            None => id,
            Some(settings) => replace_region_setting(rng, &id, settings),
        }
    }

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

        let mut forest = MapGen::default();
        forest.fill =
            Some(DistributionInner::Normal("t_region_groundcover".into()));
        self.map_data.insert("forest".into(), forest);

        let mut river_curved_not = MapGen::default();
        river_curved_not.fill =
            Some(DistributionInner::Normal("t_water".into()));
        self.map_data
            .insert("river_curved_not".into(), river_curved_not);

        let mut river_straight = MapGen::default();
        river_straight.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data
            .insert("river_straight".into(), river_straight);

        let mut river_curved = MapGen::default();
        river_curved.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("river_curved".into(), river_curved);

        let mut subway_straight = MapGen::default();
        subway_straight.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data
            .insert("subway_straight".into(), subway_straight);

        let mut subway_curved = MapGen::default();
        subway_curved.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data.insert("subway_curved".into(), subway_curved);

        let mut subway_end = MapGen::default();
        subway_end.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data.insert("subway_end".into(), subway_end);

        let mut subway_tee = MapGen::default();
        subway_tee.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data.insert("subway_tee".into(), subway_tee);

        let mut subway_four_way = MapGen::default();
        subway_four_way.fill = Some(DistributionInner::Normal("t_road".into()));
        self.map_data
            .insert("subway_four_way".into(), subway_four_way);

        let mut lake_shore = MapGen::default();
        lake_shore.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("lake_shore".into(), lake_shore);

        let mut ocean_shore = MapGen::default();
        ocean_shore.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("ocean_shore".into(), ocean_shore);

        let mut ravine_edge = MapGen::default();
        ravine_edge.fill = Some(DistributionInner::Normal("t_water".into()));
        self.map_data.insert("ravine_edge".into(), ravine_edge);
    }
}

pub fn parse_json_entries(path: &PathBuf) -> Result<Vec<CDDAJsonEntry>, Error> {
    info!("Reading and parsing json file at {}", path.display());
    let reader = BufReader::new(File::open(path)?);

    let des = match serde_json::from_reader::<BufReader<File>, Vec<CDDAJsonEntry>>(
        reader,
    ) {
        Ok(des) => des,
        Err(e) => {
            warn!("Failed to deserialize {}, error: {}", path.display(), e);
            return Err(anyhow!(e));
        },
    };

    Ok(des)
}

#[derive(Default)]
pub struct IntermediateCDDAJsonData {
    pub intermediate_map_data: Vec<CDDAMapDataIntermediate>,
    pub intermediate_region_settings:
        HashMap<CDDAIdentifier, CDDARegionSettings>,
    pub intermediate_palettes: HashMap<CDDAIdentifier, CDDAPaletteIntermediate>,
    pub intermediate_vehicles: HashMap<CDDAIdentifier, CDDAVehicleIntermediate>,
    pub intermediate_vehicle_parts:
        HashMap<CDDAIdentifier, CDDAVehiclePartIntermediate>,
    pub intermediate_terrains: HashMap<CDDAIdentifier, CDDATerrainIntermediate>,
    pub intermediate_furnitures:
        HashMap<CDDAIdentifier, CDDAFurnitureIntermediate>,
    pub intermediate_overmap_locations:
        HashMap<CDDAIdentifier, CDDAOvermapLocationIntermediate>,
    pub intermediate_overmap_terrains:
        HashMap<CDDAIdentifier, CDDAOvermapTerrainIntermediate>,
    pub intermediate_overmap_specials:
        HashMap<CDDAIdentifier, CDDAOvermapSpecialIntermediate>,
    pub intermediate_monster_groups:
        HashMap<CDDAIdentifier, CDDAMonsterGroupIntermediate>,
}

pub fn set_intermediate_data_from_json_entries(
    intermediate_data: &mut IntermediateCDDAJsonData,
    path: &PathBuf,
    deserialized_entries: Vec<CDDAJsonEntry>,
) -> Result<(), Error> {
    for des_entry in deserialized_entries {
        match des_entry {
            CDDAJsonEntry::Mapgen(mapgen) => {
                intermediate_data.intermediate_map_data.push(mapgen);
            },
            CDDAJsonEntry::RegionSettings(rs) => {
                debug!("Found Region setting {} in {}", rs.id, path.display());

                let id = rs.id.clone();
                let mut region_settings: CDDARegionSettings = rs.into();
                region_settings.source = Some(path.clone());

                intermediate_data
                    .intermediate_region_settings
                    .insert(id, region_settings);
            },
            CDDAJsonEntry::Palette(mut p) => {
                debug!("Found Palette {} in {}", p.id, path.display());

                p.source = Some(path.clone());

                intermediate_data
                    .intermediate_palettes
                    .insert(p.id.clone(), p);
            },
            CDDAJsonEntry::Terrain(terrain) => {
                for ident in terrain.id.clone().into_vec() {
                    debug!(
                        "Found Terrain entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = terrain.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_terrains
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::Furniture(furniture) => {
                for ident in furniture.id.clone().into_vec() {
                    debug!(
                        "Found Furniture entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = furniture.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_furnitures
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::MonsterGroup(group) => {
                for ident in group.id.clone().into_vec() {
                    debug!(
                        "Found MonsterGroup entry {} in {}",
                        ident,
                        path.display()
                    );

                    let mut clone = group.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_monster_groups
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::OvermapLocation(location) => {
                for ident in location.id.clone().into_vec() {
                    debug!(
                        "Found OvermapLocation entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = location.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_overmap_locations
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::OvermapTerrain(terrain) => {
                for ident in terrain.id.clone().into_vec() {
                    debug!(
                        "Found OvermapTerrain entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = terrain.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_overmap_terrains
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::OvermapSpecial(s) => {
                for ident in s.id.clone().into_vec() {
                    debug!(
                        "Found OvermapSpecial entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = s.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_overmap_specials
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::Vehicle(v) => {
                for ident in v.id.clone().into_vec() {
                    debug!(
                        "Found Vehicle entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = v.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_vehicles
                        .insert(ident, clone);
                }
            },
            CDDAJsonEntry::VehiclePart(vp) => {
                for ident in vp.id.clone().into_vec() {
                    debug!(
                        "Found VehiclePart entry {} in {}",
                        &ident,
                        path.display()
                    );

                    let mut clone = vp.clone();
                    clone.id = MeabyVec::Single(ident.clone());
                    clone.source = Some(path.clone());

                    intermediate_data
                        .intermediate_vehicle_parts
                        .insert(ident, clone);
                }
            },
            _ => {
                info!("Unused JSON entry in {}", path.display());
            },
        }
    }

    Ok(())
}

pub fn replace_data_in_cdda_data(
    cdda_data: &mut DeserializedCDDAJsonData,
    intermediate_data: IntermediateCDDAJsonData,
) -> Result<(), Error> {
    for mapgen in intermediate_data.intermediate_map_data {
        if let Some(om_terrain) = mapgen.om_terrain.clone() {
            match om_terrain {
                OmTerrain::Single(id) => {
                    let mut map_data_collection: Overmap = mapgen.try_into()?;

                    cdda_data.map_data.insert(
                        CDDAIdentifier(id.clone()),
                        map_data_collection
                            .maps
                            .remove(&UVec2::ZERO.into())
                            .unwrap(),
                    );
                },
                OmTerrain::Duplicate(duplicate) => {
                    let map_data_collection: Overmap = mapgen.try_into()?;

                    for id in duplicate.iter() {
                        cdda_data.map_data.insert(
                            CDDAIdentifier(id.clone()),
                            map_data_collection
                                .maps
                                .get(&UVec2::ZERO.into())
                                .unwrap()
                                .clone(),
                        );
                    }
                },
                OmTerrain::Nested(nested) => {
                    let map_data_collection: Overmap = mapgen.try_into()?;

                    for (coords, map_data) in map_data_collection.maps {
                        let om_terrain = nested
                            .get(coords.y as usize)
                            .unwrap()
                            .get(coords.x as usize)
                            .unwrap()
                            .clone();

                        cdda_data
                            .map_data
                            .insert(CDDAIdentifier(om_terrain), map_data);
                    }
                },
            }
        } else if let Some(nested_mapgen) = mapgen.nested_mapgen_id.clone() {
            let mut map_data_collection: Overmap = mapgen.try_into()?;

            cdda_data.map_data.insert(
                nested_mapgen.clone(),
                map_data_collection
                    .maps
                    .remove(&UVec2::ZERO.into())
                    .unwrap(),
            );
        } else if let Some(update_mapgen) = mapgen.update_mapgen_id.clone() {
            let mut map_data_collection: Overmap = mapgen.try_into()?;

            cdda_data.map_data.insert(
                update_mapgen.clone(),
                map_data_collection
                    .maps
                    .remove(&UVec2::ZERO.into())
                    .unwrap(),
            );
        }
    }

    for (id, region_setting) in intermediate_data.intermediate_region_settings {
        cdda_data.region_settings.insert(id, region_setting);
    }

    for (id, palette) in intermediate_data.intermediate_palettes {
        let palette: CDDAPalette = palette.into();
        cdda_data.palettes.insert(id, palette);
    }

    for (id, terrain) in &intermediate_data.intermediate_terrains {
        cdda_data.terrain.insert(
            id.clone(),
            terrain
                .calculate_copy(&intermediate_data.intermediate_terrains)
                .into(),
        );
    }

    for (id, furniture) in &intermediate_data.intermediate_furnitures {
        cdda_data.furniture.insert(
            id.clone(),
            furniture
                .calculate_copy(&intermediate_data.intermediate_furnitures)
                .into(),
        );
    }

    for (id, monster_group) in &intermediate_data.intermediate_monster_groups {
        cdda_data.monster_groups.insert(
            id.clone(),
            monster_group
                .calculate_copy(&intermediate_data.intermediate_monster_groups)
                .into(),
        );
    }

    for (id, overmap_location) in
        &intermediate_data.intermediate_overmap_locations
    {
        cdda_data.overmap_locations.insert(
            id.clone(),
            overmap_location
                .calculate_copy(
                    &intermediate_data.intermediate_overmap_locations,
                )
                .into(),
        );
    }

    for (id, overmap_terrain) in
        &intermediate_data.intermediate_overmap_terrains
    {
        cdda_data.overmap_terrains.insert(
            id.clone(),
            overmap_terrain
                .calculate_copy(
                    &intermediate_data.intermediate_overmap_terrains,
                )
                .into(),
        );
    }

    for (id, overmap_special) in
        &intermediate_data.intermediate_overmap_specials
    {
        cdda_data.overmap_specials.insert(
            id.clone(),
            overmap_special
                .calculate_copy(
                    &intermediate_data.intermediate_overmap_specials,
                )
                .into(),
        );
    }

    for (id, vehicle) in &intermediate_data.intermediate_vehicles {
        cdda_data.vehicles.insert(
            id.clone(),
            vehicle
                .calculate_copy(&intermediate_data.intermediate_vehicles)
                .into(),
        );
    }

    for (id, vehicle_part) in &intermediate_data.intermediate_vehicle_parts {
        cdda_data.vehicle_parts.insert(
            id.clone(),
            vehicle_part
                .calculate_copy(&intermediate_data.intermediate_vehicle_parts)
                .into(),
        );
    }

    Ok(())
}

pub struct CDDADataLoader {
    pub json_path: PathBuf,
}

impl Load<DeserializedCDDAJsonData> for CDDADataLoader {
    async fn load(&mut self) -> Result<DeserializedCDDAJsonData, Error> {
        let mut walkdir = WalkDir::new(&self.json_path);

        let mut cdda_data = DeserializedCDDAJsonData::default();
        cdda_data.add_hardcoded_map_data();

        let mut intermediate_data = IntermediateCDDAJsonData::default();

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

            let deserialized_entries = match parse_json_entries(&path) {
                Ok(d) => d,
                Err(_) => continue,
            };

            set_intermediate_data_from_json_entries(
                &mut intermediate_data,
                &path,
                deserialized_entries,
            )?
        }

        replace_data_in_cdda_data(&mut cdda_data, intermediate_data)?;

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

pub async fn get_saved_editor_data() -> Result<ProgramData, Error> {
    let project_dir = ProjectDirs::from("", "", "CDDA Map Editor");

    let directory_path = match project_dir {
        None => {
            warn!(
                "No valid project directory found, creating data folder application directory instead"
            );
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
            let local_dir = dir.data_local_dir();
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
            let mut editor_data_loader = ProgramDataLoader {
                path: directory_path.clone(),
            };

            let mut data = match editor_data_loader.load() {
                Ok(d) => {
                    info!("config.json file successfully read and parsed");
                    d
                },
                Err(e) => {
                    error!("{}", e.to_string());
                    info!(
                        "Error while reading config.json file, recreating file"
                    );

                    let mut default_editor_data = ProgramData::default();
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

            let mut default_editor_data = ProgramData::default();
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
