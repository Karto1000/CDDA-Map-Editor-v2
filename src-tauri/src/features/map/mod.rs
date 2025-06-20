pub(crate) mod importing;
pub(crate) mod map_properties;
pub(crate) mod place;

use crate::data::io::DeserializedCDDAJsonData;
use crate::data::map_data::{
    MapGenMonsterType, NeighborDirection, OmTerrainMatch, PlaceOuter,
};
use crate::data::palettes::{CDDAPalette, Parameter};
use crate::data::{
    replace_region_setting, GetIdentifier, GetIdentifierError, GetRandomError,
    TileLayer,
};
use crate::features::program_data::ZLevel;
use crate::features::tileset::legacy_tileset::TilesheetCDDAId;
use crate::util::Rotation;
use cdda_lib::types::{
    CDDAIdentifier, DistributionInner, MapGenValue, NumberOrRange,
    ParameterIdentifier, Weighted,
};
use cdda_lib::{
    DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH, NULL_FURNITURE, NULL_TERRAIN,
};
use downcast_rs::{impl_downcast, Downcast, DowncastSend, DowncastSync};
use dyn_clone::{clone_trait_object, DynClone};
use futures_lite::StreamExt;
use glam::{IVec2, IVec3, UVec2};
use indexmap::IndexMap;
use log::warn;
use rand::{rng, Rng};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use thiserror::Error;

pub const SPECIAL_EMPTY_CHAR: char = ' ';
pub const DEFAULT_MAP_DATA_SIZE: UVec2 = UVec2::new(24, 24);

pub trait Place:
    Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend
{
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        None
    }
}

clone_trait_object!(Place);
impl_downcast!(sync Place);

// Things like terrain, furniture, monsters This allows us to get the Identifier
pub trait Property:
    Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend
{
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        None
    }
}

clone_trait_object!(Property);
impl_downcast!(sync Property);

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Hash,
    PartialOrd,
    PartialEq,
    Eq,
    Ord,
    EnumIter,
)]
#[serde(rename_all = "snake_case")]
pub enum MappingKind {
    Terrain,
    Furniture,
    Trap,
    ItemGroups,
    Computer,
    Sign,
    Toilet,
    Gaspump,
    Monsters,
    Monster,
    Field,
    Nested,
    Vehicle,
    Corpse,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    pub character: char,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FurnitureRepresentation {
    pub selected_furniture: Value,
    pub selected_sign: Value,
    pub selected_computer: Value,
    pub selected_gaspump: Value,
}

// The struct which holds the data that will be shown in the side panel in the ui
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellRepresentation {
    pub terrain: Value,
    pub furniture: FurnitureRepresentation,
    pub item_groups: Value,
}

#[derive(Debug, Default, Serialize, Eq, PartialEq)]
pub enum TileState {
    #[default]
    Normal,
    Broken,
    Open,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct SetTile {
    id: TilesheetCDDAId,
    layer: TileLayer,
    coordinates: IVec2,
    rotation: Rotation,
    state: TileState,
}

impl SetTile {
    pub fn terrain(
        id: impl Into<TilesheetCDDAId>,
        coordinates: IVec2,
        rotation: impl Into<Rotation>,
        state: TileState,
    ) -> Self {
        Self {
            id: id.into(),
            layer: TileLayer::Terrain,
            rotation: rotation.into(),
            coordinates,
            state,
        }
    }

    pub fn furniture(
        id: impl Into<TilesheetCDDAId>,
        coordinates: IVec2,
        rotation: impl Into<Rotation>,
        state: TileState,
    ) -> Self {
        Self {
            id: id.into(),
            layer: TileLayer::Furniture,
            rotation: rotation.into(),
            coordinates,
            state,
        }
    }

    pub fn field(
        id: impl Into<TilesheetCDDAId>,
        coordinates: IVec2,
        rotation: impl Into<Rotation>,
        state: TileState,
    ) -> Self {
        Self {
            id: id.into(),
            layer: TileLayer::Field,
            rotation: rotation.into(),
            coordinates,
            state,
        }
    }

    pub fn monster(
        id: impl Into<TilesheetCDDAId>,
        coordinates: IVec2,
        rotation: impl Into<Rotation>,
        state: TileState,
    ) -> Self {
        Self {
            id: id.into(),
            layer: TileLayer::Monster,
            rotation: rotation.into(),
            coordinates,
            state,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapDataFlag {
    EraseAllBeforePlacingTerrain,
    AllowTerrainUnderOtherData,

    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapGenNested {
    pub neighbors: Option<HashMap<NeighborDirection, Vec<OmTerrainMatch>>>,
    pub joins: Option<HashMap<NeighborDirection, Vec<OmTerrainMatch>>>,

    pub chunks: Vec<Weighted<MapGenValue>>,

    #[serde(default)]
    // This is basically just any "else_chunks"
    pub invert_condition: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapDataConfig {
    pub simulated_neighbors: HashMap<NeighborDirection, Vec<CDDAIdentifier>>,
}

impl Default for MapDataConfig {
    fn default() -> Self {
        let mut simulated_neighbors = HashMap::new();
        simulated_neighbors.insert(NeighborDirection::Above, vec![]);
        simulated_neighbors.insert(NeighborDirection::Below, vec![]);
        simulated_neighbors.insert(NeighborDirection::East, vec![]);
        simulated_neighbors.insert(NeighborDirection::West, vec![]);
        simulated_neighbors.insert(NeighborDirection::North, vec![]);
        simulated_neighbors.insert(NeighborDirection::South, vec![]);
        simulated_neighbors.insert(NeighborDirection::NorthEast, vec![]);
        simulated_neighbors.insert(NeighborDirection::NorthWest, vec![]);
        simulated_neighbors.insert(NeighborDirection::SouthEast, vec![]);
        simulated_neighbors.insert(NeighborDirection::SouthWest, vec![]);

        MapDataConfig {
            simulated_neighbors,
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub enum MapDataRotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub cells: IndexMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,
    pub map_size: UVec2,
    pub predecessor: Option<CDDAIdentifier>,

    pub config: MapDataConfig,
    pub rotation: MapDataRotation,

    pub calculated_parameters: IndexMap<ParameterIdentifier, CDDAIdentifier>,
    pub parameters: IndexMap<ParameterIdentifier, Parameter>,
    pub palettes: Vec<MapGenValue>,
    pub flags: HashSet<MapDataFlag>,

    #[serde(skip)]
    pub properties: HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>,

    // #[serde(skip)]
    // pub set: Vec<Arc<dyn Set>>,
    #[serde(skip)]
    pub place: HashMap<MappingKind, Vec<PlaceOuter<Arc<dyn Place>>>>,
}

impl Default for MapData {
    fn default() -> Self {
        let mut cells = IndexMap::new();

        for y in 0..DEFAULT_MAP_HEIGHT {
            for x in 0..DEFAULT_MAP_WIDTH {
                cells.insert(
                    UVec2::new(x as u32, y as u32),
                    Cell { character: ' ' },
                );
            }
        }
        let fill =
            Some(DistributionInner::Normal(CDDAIdentifier::from("t_grass")));

        Self {
            cells,
            fill,
            map_size: DEFAULT_MAP_DATA_SIZE,
            predecessor: None,
            config: Default::default(),
            rotation: Default::default(),
            calculated_parameters: Default::default(),
            parameters: Default::default(),
            properties: Default::default(),
            palettes: Default::default(),
            place: Default::default(),
            flags: Default::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CalculateParametersError {
    #[error("Missing Palette {0} in Loaded CDDA Palettes")]
    MissingPalette(String),

    #[error(transparent)]
    GetRandomError(#[from] GetRandomError),

    #[error(transparent)]
    GetIdentifierError(#[from] GetIdentifierError),
}

#[derive(Debug, Error)]
pub enum GetMappedCDDAIdsError {
    #[error("Missing default Region Settings in Loaded CDDA Data")]
    MissingRegionSettings,

    #[error("Missing Overmap Terrain in loaded CDDA Data for predecessor {0}")]
    MissingOvermapTerrainForPredecessor(String),

    #[error("Missing Mapgen Entry for Predecessor {0}")]
    MissingMapgenEntryForPredecessor(String),
}

impl MapData {
    pub fn calculate_parameters(
        &mut self,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
    ) -> Result<(), CalculateParametersError> {
        let mut calculated_parameters = IndexMap::new();

        for (id, parameter) in self.parameters.iter() {
            let calculated_value = parameter
                .default
                .distribution
                .get_identifier(&calculated_parameters)?;

            calculated_parameters.insert(id.clone(), calculated_value);
        }

        for mapgen_value in self.palettes.iter() {
            let id = mapgen_value.get_identifier(&calculated_parameters)?;
            let palette = all_palettes.get(&id).ok_or(
                CalculateParametersError::MissingPalette(id.to_string()),
            )?;

            palette
                .calculate_parameters(all_palettes)?
                .into_iter()
                .for_each(|(palette_id, ident)| {
                    calculated_parameters.insert(palette_id, ident);
                });
        }

        self.calculated_parameters = calculated_parameters;

        Ok(())
    }

    pub fn get_mapped_cdda_ids(
        &self,
        json_data: &DeserializedCDDAJsonData,
        z: ZLevel,
    ) -> Result<HashMap<IVec3, MappedCDDAIdsForTile>, GetMappedCDDAIdsError>
    {
        let mut local_mapped_cdda_ids = HashMap::new();

        let region_settings = json_data
            .region_settings
            .get(&CDDAIdentifier("default".into()))
            .ok_or(GetMappedCDDAIdsError::MissingRegionSettings)?;

        let fill_terrain_sprite = match &self.fill {
            None => None,
            Some(id) => {
                Some(id.get_identifier(&self.calculated_parameters).unwrap())
            },
        };

        // we need to calculate the predecessor_mapgen here before so we can replace it later
        match &self.predecessor {
            None => {},
            Some(predecessor_id) => {
                let predecessor =
                    json_data.overmap_terrains.get(predecessor_id)
                        .ok_or(GetMappedCDDAIdsError::MissingOvermapTerrainForPredecessor(predecessor_id.0.clone()))?;

                let predecessor_map_data = match &predecessor
                    .mapgen
                    .clone()
                    .unwrap_or_default()
                    .first()
                {
                    None => {
                        // This terrain is defined in a json file, so we can just search for it
                        json_data.map_data.get(predecessor_id).ok_or(GetMappedCDDAIdsError::MissingMapgenEntryForPredecessor(predecessor_id.0.clone()))?
                    },
                    Some(omtm) => json_data.map_data.get(&omtm.builtin).expect(
                        format!(
                            "Hardcoded Map data for predecessor {} to exist",
                            omtm.builtin
                        )
                        .as_str(),
                    ),
                };

                local_mapped_cdda_ids =
                    predecessor_map_data.get_mapped_cdda_ids(json_data, z)?;
            },
        }

        self.cells.iter().for_each(|(p, _)| {
            let transformed_position =
                self.transform_coordinates(&p.as_ivec2());
            let coords =
                IVec3::new(transformed_position.x, transformed_position.y, z);
            // If there was no id added from the predecessor mapgen, we will add the fill sprite here
            match local_mapped_cdda_ids.get_mut(&coords) {
                None => {
                    let mut mapped_ids = MappedCDDAIdsForTile::default();

                    mapped_ids.terrain = fill_terrain_sprite.clone().map(|s| {
                        MappedCDDAId::simple(TilesheetCDDAId::simple(
                            replace_region_setting(
                                &s,
                                region_settings,
                                &json_data.terrain,
                                &json_data.furniture,
                            ),
                        ))
                    });

                    local_mapped_cdda_ids.insert(coords, mapped_ids);
                },
                Some(mapped_ids) => {
                    if mapped_ids.terrain.is_none() {
                        mapped_ids.terrain =
                            fill_terrain_sprite.clone().map(|s| {
                                MappedCDDAId::simple(TilesheetCDDAId::simple(
                                    replace_region_setting(
                                        &s,
                                        region_settings,
                                        &json_data.terrain,
                                        &json_data.furniture,
                                    ),
                                ))
                            })
                    }
                },
            };
        });

        let all_commands = self.get_commands(&json_data);

        for command in all_commands {
            let command_3d_coords =
                IVec3::new(command.coordinates.x, command.coordinates.y, z);

            let id = TilesheetCDDAId {
                id: replace_region_setting(
                    &command.id.id,
                    region_settings,
                    &json_data.terrain,
                    &json_data.furniture,
                ),
                prefix: command.id.prefix,
                postfix: command.id.postfix,
            };

            let mut mapped_id = MappedCDDAId::simple(id);
            mapped_id.rotation = command.rotation;

            match command.state {
                TileState::Normal => {},
                TileState::Broken => mapped_id.is_broken = true,
                TileState::Open => mapped_id.is_open = true,
            }

            let ident_mut =
                match local_mapped_cdda_ids.get_mut(&command_3d_coords) {
                    None => {
                        local_mapped_cdda_ids.insert(
                            command_3d_coords.clone(),
                            MappedCDDAIdsForTile::default(),
                        );
                        local_mapped_cdda_ids
                            .get_mut(&command_3d_coords)
                            // Safe
                            .unwrap()
                    },
                    Some(i) => i,
                };

            match command.layer {
                TileLayer::Terrain => {
                    ident_mut.terrain = Some(mapped_id.clone());
                },
                TileLayer::Furniture => {
                    ident_mut.furniture = Some(mapped_id.clone());
                },
                TileLayer::Monster => {
                    ident_mut.monster = Some(mapped_id.clone());
                },
                TileLayer::Field => {
                    ident_mut.field = Some(mapped_id.clone());
                },
            }
        }

        Ok(local_mapped_cdda_ids)
    }

    /// Transform 2d coordinates based on the rotation of the map
    /// This is used to rotate nested mapgens as well as vehicles and other tiles which need to be rotated
    fn transform_coordinates(&self, position: &IVec2) -> IVec2 {
        let (map_width, map_height) = (self.map_size.x, self.map_size.y);

        match self.rotation {
            MapDataRotation::Deg0 => position.clone(),
            MapDataRotation::Deg90 => {
                IVec2::new(map_height as i32 - 1 - position.y, position.x)
            },
            MapDataRotation::Deg180 => IVec2::new(
                map_width as i32 - 1 - position.x,
                map_height as i32 - 1 - position.y,
            ),
            MapDataRotation::Deg270 => {
                IVec2::new(position.y, map_width as i32 - 1 - position.x)
            },
        }
    }

    pub fn get_commands(
        &self,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<SetTile> {
        // We need to store all commands in this list here so we can sort it and act them out in
        // the order the VisibleMappingCommandKind enum has
        let mut all_commands: Vec<SetTile> = vec![];

        // We need to insert the mapped_sprite before we get the fg and bg of this sprite since
        // the function relies on the mapped sprite of this sprite to already exist
        self.cells.iter().for_each(|(p, cell)| {
            // Transform the coordinate `p` based on the map rotation
            let transformed_position =
                self.transform_coordinates(&p.as_ivec2());

            let ident_commands = self.get_identifier_change_commands(
                &cell.character,
                &transformed_position,
                &json_data,
            );

            all_commands.extend(ident_commands)
        });

        for (_, place_vec) in self.place.iter() {
            for place in place_vec {
                let upper_bound = place.repeat.rand_number();

                for _ in 0..upper_bound {
                    let position = place.coordinates();
                    let transformed_position =
                        self.transform_coordinates(&position);

                    // We only want to place one in place.chance times
                    let rand_chance_num = rng().random_range(0..=100);
                    if rand_chance_num > place.chance {
                        continue;
                    }

                    match place.inner.get_commands(
                        &transformed_position,
                        self,
                        json_data,
                    ) {
                        None => {},
                        Some(commands) => {
                            all_commands.extend(commands);
                        },
                    }
                }
            }
        }

        all_commands.sort_by(|a, b| a.layer.cmp(&b.layer));
        all_commands
    }

    pub fn get_visible_mapping(
        &self,
        mapping_kind: &MappingKind,
        character: &char,
        position: &IVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let mapping = self.properties.get(mapping_kind)?;

        if let Some(id) = mapping.get(character) {
            return id.get_commands(position, self, json_data);
        }

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value
                .get_identifier(&self.calculated_parameters)
                .ok()?;

            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) = palette.get_visible_mapping(
                mapping_kind,
                character,
                position,
                self,
                json_data,
            ) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_identifier_change_commands(
        &self,
        character: &char,
        position: &IVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<SetTile> {
        let mut commands = Vec::new();

        for kind in MappingKind::iter() {
            let kind_commands = self
                .get_visible_mapping(&kind, character, position, json_data)
                .unwrap_or_default();

            commands.extend(kind_commands)
        }

        commands
    }
}

impl Serialize for MapData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serialized_cells = HashMap::new();

        for (key, value) in &self.cells {
            let key_str = format!("{},{}", key.x, key.y);
            serialized_cells.insert(key_str, value);
        }

        let mut state = serializer
            .serialize_struct("MapData", 2 + serialized_cells.len())?;

        state.serialize_field("cells", &serialized_cells)?;

        state.end()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PlaceableSetType {
    Terrain,
    Furniture,
    Trap,
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RemovableSetType {
    ItemRemove,
    FieldRemove,
    TrapRemove,
    CreatureRemove,
}

#[derive(Debug, Clone)]
pub enum SetOperation {
    Place {
        id: CDDAIdentifier,
        ty: PlaceableSetType,
    },
    Remove {
        ty: RemovableSetType,
    },
    Radiation {
        amount: NumberOrRange<u32>,
    },
    Variable {
        id: CDDAIdentifier,
    },
    Bash {},
    Burn {},
}

#[derive(Debug, Clone)]
pub struct SetPoint {
    pub x: NumberOrRange<u32>,
    pub y: NumberOrRange<u32>,
    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

#[derive(Debug, Clone)]
pub struct SetLine {
    pub from_x: NumberOrRange<u32>,
    pub from_y: NumberOrRange<u32>,

    pub to_x: NumberOrRange<u32>,
    pub to_y: NumberOrRange<u32>,

    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

#[derive(Debug, Clone)]
pub struct SetSquare {
    pub top_left_x: NumberOrRange<u32>,
    pub top_left_y: NumberOrRange<u32>,

    pub bottom_right_x: NumberOrRange<u32>,
    pub bottom_right_y: NumberOrRange<u32>,

    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

#[cfg(test)]
mod tests {
    use crate::features::map::importing::SingleMapDataImporter;
    use crate::features::map::map_properties::TerrainProperty;
    use crate::features::map::MappingKind;
    use crate::util::Load;
    use crate::TEST_CDDA_DATA;
    use cdda_lib::types::{
        CDDADistributionInner, CDDAIdentifier, Distribution, DistributionInner,
        MapGenValue, MeabyVec, MeabyWeighted, ParameterIdentifier, Switch,
        Weighted,
    };
    use glam::UVec2;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tokio;

    const TEST_DATA_PATH: &str = "test_data";

    #[tokio::test]
    async fn test_fill_ter() {
        let mut map_loader = SingleMapDataImporter {
            paths: vec![
                PathBuf::from(TEST_DATA_PATH).join("test_fill_ter.json")
            ],
            om_terrain: "test_fill_ter".into(),
        };

        let map_data = map_loader
            .load()
            .await
            .unwrap()
            .maps
            .remove(&UVec2::ZERO)
            .unwrap();

        for (coords, cell) in map_data.cells.iter() {
            assert_eq!(cell.character, ' ');
            assert!(coords.x < 24 && coords.y < 24);
        }

        assert_eq!(
            map_data.fill,
            Some(DistributionInner::Normal("t_grass".into()))
        );
    }

    #[tokio::test]
    async fn test_parameters() {
        let cdda_data = TEST_CDDA_DATA.get().await;

        let mut map_loader = SingleMapDataImporter {
            paths: vec![PathBuf::from(TEST_DATA_PATH).join("test_terrain.json")],
            om_terrain: "test_terrain".into(),
        };

        let mut map_data = map_loader
            .load()
            .await
            .unwrap()
            .maps
            .remove(&UVec2::ZERO)
            .unwrap();

        map_data.calculate_parameters(&cdda_data.palettes);

        let parameter_identifier =
            ParameterIdentifier("terrain_type".to_string());
        let parameter = map_data.parameters.get(&parameter_identifier).unwrap();

        let weighted_grass = Weighted::new("t_grass", 10);
        let weighted_grass_dead = Weighted::new("t_grass_dead", 1);

        let expected_distribution = Distribution {
            distribution: MeabyVec::Vec(vec![
                MeabyWeighted::Weighted(weighted_grass),
                MeabyWeighted::Weighted(weighted_grass_dead),
            ]),
        };

        assert_eq!(parameter.default, expected_distribution);

        let calculated_parameter = map_data
            .calculated_parameters
            .get(&parameter_identifier)
            .unwrap();

        assert!(
            calculated_parameter.0 == "t_grass".to_string()
                || calculated_parameter.0 == "t_grass_dead".to_string()
        )
    }

    #[tokio::test]
    async fn test_terrain() {
        const SINGLE_CHAR: char = '.';
        const NOT_WEIGHTED_DISTRIBUTION_CHAR: char = '1';
        const WEIGHTED_DISTRIBUTION_CHAR: char = '2';
        const WEIGHTED_DISTRIBUTION_WITH_KEYWORD_CHAR: char = '3';
        const PARAMETER_CHAR: char = '4';
        const SWITCH_CHAR: char = '5';

        let cdda_data = TEST_CDDA_DATA.get().await;

        let mut map_loader = SingleMapDataImporter {
            paths: vec![PathBuf::from(TEST_DATA_PATH).join("test_terrain.json")],
            om_terrain: "test_terrain".into(),
        };

        let mut map_data = map_loader
            .load()
            .await
            .unwrap()
            .maps
            .remove(&UVec2::ZERO)
            .unwrap();

        map_data.calculate_parameters(&cdda_data.palettes);

        // Test the terrain mapped to a single sprite
        {
            let single_terrain = map_data.cells.get(&UVec2::new(0, 1)).unwrap();
            assert_eq!(single_terrain.character, SINGLE_CHAR);

            let terrain_property = map_data
                .properties
                .get(&MappingKind::Terrain)
                .unwrap()
                .get(&SINGLE_CHAR)
                .unwrap()
                .clone();

            let terrain_property =
                terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::String("t_grass".into())
            )
        }

        // Test the distribution that is not weighted
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(0, 0)).unwrap();
            assert_eq!(distr_terrain.character, NOT_WEIGHTED_DISTRIBUTION_CHAR);

            let terrain_property = map_data
                .properties
                .get(&MappingKind::Terrain)
                .unwrap()
                .get(&NOT_WEIGHTED_DISTRIBUTION_CHAR)
                .unwrap()
                .clone();

            let terrain_property =
                terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let expected_distribution = vec![
                MeabyWeighted::NotWeighted("t_grass".into()),
                MeabyWeighted::NotWeighted("t_grass_dead".into()),
            ];

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::Distribution(MeabyVec::Vec(expected_distribution))
            );
        }

        // Test the distribution that is weighted
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(1, 0)).unwrap();
            assert_eq!(distr_terrain.character, WEIGHTED_DISTRIBUTION_CHAR);

            let terrain_property = map_data
                .properties
                .get(&MappingKind::Terrain)
                .unwrap()
                .get(&WEIGHTED_DISTRIBUTION_CHAR)
                .unwrap()
                .clone();

            let terrain_property =
                terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let weighted_grass = Weighted::new("t_grass", 10);
            let weighted_grass_dead = Weighted::new("t_grass_dead", 1);

            let expected_distribution = vec![
                MeabyWeighted::Weighted(weighted_grass),
                MeabyWeighted::Weighted(weighted_grass_dead),
            ];

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::Distribution(MeabyVec::Vec(expected_distribution))
            );
        }

        // Test the weighted distribution with the "distribution" keyword
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(2, 0)).unwrap();
            assert_eq!(
                distr_terrain.character,
                WEIGHTED_DISTRIBUTION_WITH_KEYWORD_CHAR
            );

            let terrain_property = map_data
                .properties
                .get(&MappingKind::Terrain)
                .unwrap()
                .get(&WEIGHTED_DISTRIBUTION_WITH_KEYWORD_CHAR)
                .unwrap()
                .clone();

            let terrain_property =
                terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let weighted_grass = Weighted::new("t_grass", 1);
            let weighted_grass_dead = Weighted::new("t_grass_dead", 10);

            let expected_distribution = Distribution {
                distribution: MeabyVec::Vec(vec![
                    MeabyWeighted::Weighted(weighted_grass),
                    MeabyWeighted::Weighted(weighted_grass_dead),
                ]),
            };

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::Distribution(MeabyVec::Single(
                    MeabyWeighted::NotWeighted(
                        CDDADistributionInner::Distribution(
                            expected_distribution
                        )
                    )
                ))
            );
        }

        // Test if a set parameter works
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(3, 0)).unwrap();
            assert_eq!(distr_terrain.character, PARAMETER_CHAR);

            let terrain_property = map_data
                .properties
                .get(&MappingKind::Terrain)
                .unwrap()
                .get(&PARAMETER_CHAR)
                .unwrap()
                .clone();

            let terrain_property =
                terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let to_eq = MapGenValue::Param {
                param: ParameterIdentifier("terrain_type".to_string()),
                fallback: Some("t_grass".into()),
            };

            assert_eq!(terrain_property.mapgen_value, to_eq);
        }

        // Test if a switch works
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(4, 0)).unwrap();
            assert_eq!(distr_terrain.character, SWITCH_CHAR);

            let terrain_property = map_data
                .properties
                .get(&MappingKind::Terrain)
                .unwrap()
                .get(&SWITCH_CHAR)
                .unwrap()
                .clone();

            let terrain_property =
                terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let mut to_eq_cases: HashMap<CDDAIdentifier, CDDAIdentifier> =
                HashMap::new();
            to_eq_cases.insert("t_grass".into(), "t_concrete_railing".into());
            to_eq_cases.insert("t_grass_dead".into(), "t_concrete_wall".into());

            let to_eq = MapGenValue::Switch {
                switch: Switch {
                    param: ParameterIdentifier("terrain_type".into()),
                    fallback: "t_grass".into(),
                },
                cases: to_eq_cases,
            };

            assert_eq!(terrain_property.mapgen_value, to_eq);
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MappedCDDAId {
    pub tilesheet_id: TilesheetCDDAId,
    pub rotation: Rotation,
    pub is_broken: bool,
    pub is_open: bool,
}

impl MappedCDDAId {
    pub fn simple(id: impl Into<TilesheetCDDAId>) -> Self {
        Self {
            tilesheet_id: id.into(),
            rotation: Default::default(),
            is_broken: false,
            is_open: false,
        }
    }

    ///
    /// Some parts can have multiple variants; each variant can define the symbols and broken symbols,
    /// also each variant is a tileset sprite, if the tileset defines one for the variant.
    //
    // If a part has variants, the specific variant can be specified in the vehicle prototype by
    // appending the variant to the part id after a # symbol. Thus, "frame#cross" is the "cross" variant of the "frame" part.
    //
    // Variants perform a mini-lookup chain by slicing variant string until the next _ from the
    // right until a match is found. For example the tileset lookups for seat_leather#windshield_left are as follows:
    //
    //     vp_seat_leather_windshield_left
    //
    //     vp_seat_leather_windshield
    //
    // ( At this point variant is completely gone and default tile is looked for: )
    //
    //     vp_seat_leather
    //
    // ( If still no match is found then the looks_like field of vp_seat_leather is used and tileset looks for: )
    //
    //     vp_seat
    ///
    ///
    pub fn slice_right(&self) -> MappedCDDAId {
        let new_postfix = self
            .tilesheet_id
            .postfix
            .clone()
            .map(|p| p.rsplit_once('_').map(|(s, _)| s.to_string()));

        MappedCDDAId {
            tilesheet_id: TilesheetCDDAId {
                id: self.tilesheet_id.id.clone(),
                prefix: self.tilesheet_id.prefix.clone(),
                postfix: new_postfix.flatten(),
            },
            rotation: self.rotation.clone(),
            is_broken: self.is_broken.clone(),
            is_open: self.is_open.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MappedCDDAIdsForTile {
    pub terrain: Option<MappedCDDAId>,
    pub furniture: Option<MappedCDDAId>,
    pub monster: Option<MappedCDDAId>,
    pub field: Option<MappedCDDAId>,
}

impl MappedCDDAIdsForTile {
    pub fn override_none(&mut self, other: MappedCDDAIdsForTile) {
        if other.terrain.is_some() {
            self.terrain = other.terrain;
        }

        if other.furniture.is_some() {
            self.furniture = other.furniture;
        }

        if other.monster.is_some() {
            self.monster = other.monster;
        }

        if other.field.is_some() {
            self.field = other.field;
        }
    }
}
