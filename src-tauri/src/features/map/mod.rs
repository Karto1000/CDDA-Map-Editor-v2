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
use crate::features::map::map_properties::{
    value_to_property, TerrainProperty,
};
use crate::features::program_data::ZLevel;
use crate::features::tileset::legacy_tileset::TilesheetCDDAId;
use crate::util::{Rotation, UVec2JsonKey};
use cdda_lib::types::{
    CDDAIdentifier, DistributionInner, MapGenValue, NumberOrRange,
    ParameterIdentifier, Weighted,
};
use cdda_lib::{
    DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH, NULL_FURNITURE, NULL_TERRAIN,
};
use derive_more::Display;
use downcast_rs::{impl_downcast, Downcast, DowncastSend, DowncastSync};
use dyn_clone::{clone_trait_object, DynClone};
use futures_lite::StreamExt;
use glam::{IVec2, IVec3, UVec2};
use indexmap::IndexMap;
use log::warn;
use rand::{rng, Rng};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    ) -> Option<Vec<SetTile>>;

    fn value(&self) -> Value;
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
    Display,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapGenNested {
    pub neighbors: Option<HashMap<NeighborDirection, Vec<OmTerrainMatch>>>,
    pub joins: Option<HashMap<NeighborDirection, Vec<OmTerrainMatch>>>,

    pub chunks: Vec<Weighted<MapGenValue>>,

    #[serde(default)]
    // This is basically just any "else_chunks"
    pub invert_condition: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum MapDataRotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

#[derive(Debug, Clone)]
pub struct MapData {
    pub cells: IndexMap<UVec2JsonKey, Cell>,
    pub fill: Option<DistributionInner>,
    pub map_size: UVec2,
    pub predecessor: Option<CDDAIdentifier>,

    pub config: MapDataConfig,
    pub rotation: MapDataRotation,

    pub parameters: IndexMap<ParameterIdentifier, Parameter>,
    pub palettes: Vec<MapGenValue>,
    pub flags: HashSet<MapDataFlag>,
    pub calculated_parameters: IndexMap<ParameterIdentifier, CDDAIdentifier>,

    pub properties: HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>,
    pub place: HashMap<MappingKind, Vec<PlaceOuter<Arc<dyn Place>>>>,
}

impl<'de> Deserialize<'de> for MapData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct MapDataHelper {
            cells: IndexMap<UVec2JsonKey, Cell>,
            fill: Option<DistributionInner>,
            map_size: UVec2,
            predecessor: Option<CDDAIdentifier>,
            config: MapDataConfig,
            rotation: MapDataRotation,
            parameters: IndexMap<ParameterIdentifier, Parameter>,
            palettes: Vec<MapGenValue>,
            flags: HashSet<MapDataFlag>,
            properties: HashMap<MappingKind, HashMap<char, Value>>,
        }

        let helper = MapDataHelper::deserialize(deserializer)?;

        let mut properties: HashMap<
            MappingKind,
            HashMap<char, Arc<dyn Property>>,
        > = HashMap::new();
        for (kind, inner_map) in helper.properties {
            let mut transformed_inner_map = HashMap::new();
            for (char_key, value) in inner_map {
                let property = match value_to_property(kind.clone(), value) {
                    Ok(p) => p,
                    Err(e) => {
                        warn!(
                            "Could not serialize property of kind {} due to error: {}",
                            kind, e
                        );
                        continue;
                    },
                };
                transformed_inner_map.insert(char_key, property);
            }
            properties.insert(kind, transformed_inner_map);
        }

        Ok(MapData {
            cells: helper.cells,
            fill: helper.fill,
            map_size: helper.map_size,
            predecessor: helper.predecessor,
            config: helper.config,
            rotation: helper.rotation,
            parameters: helper.parameters,
            palettes: helper.palettes,
            flags: helper.flags,
            calculated_parameters: IndexMap::new(),
            properties,
            place: HashMap::new(),
        })
    }
}

impl Serialize for MapData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MapData", 10)?;
        state.serialize_field("cells", &self.cells)?;
        state.serialize_field("fill", &self.fill)?;
        state.serialize_field("map_size", &self.map_size)?;
        state.serialize_field("predecessor", &self.predecessor)?;
        state.serialize_field("config", &self.config)?;
        state.serialize_field("rotation", &self.rotation)?;
        state.serialize_field("parameters", &self.parameters)?;
        state.serialize_field("palettes", &self.palettes)?;
        state.serialize_field("flags", &self.flags)?;

        let serialized_properties: HashMap<_, HashMap<_, _>> = self
            .properties
            .iter()
            .map(|(key, value)| {
                let serialized_inner = value
                    .iter()
                    .map(|(char_key, property)| (char_key, property.value()))
                    .collect();
                (key, serialized_inner)
            })
            .collect();

        state.serialize_field("properties", &serialized_properties)?;

        state.end()
    }
}

impl Default for MapData {
    fn default() -> Self {
        let mut cells = IndexMap::new();

        for y in 0..DEFAULT_MAP_HEIGHT {
            for x in 0..DEFAULT_MAP_WIDTH {
                cells.insert(
                    UVec2JsonKey(UVec2::new(x as u32, y as u32)),
                    Cell { character: ' ' },
                );
            }
        }
        let fill =
            Some(DistributionInner::Normal(CDDAIdentifier::from("t_grass")));

        let mut properties = HashMap::new();
        for kind in MappingKind::iter() {
            let mapping = match kind {
                MappingKind::Terrain => {
                    let mut mapping = HashMap::new();
                    mapping.insert(
                        'g',
                        Arc::new(TerrainProperty {
                            mapgen_value: MapGenValue::String("t_grass".into()),
                        }) as Arc<dyn Property>,
                    );

                    mapping
                },
                _ => HashMap::new(),
            };

            properties.insert(kind, mapping);
        }

        Self {
            cells,
            fill,
            map_size: DEFAULT_MAP_DATA_SIZE,
            predecessor: None,
            config: Default::default(),
            rotation: Default::default(),
            calculated_parameters: Default::default(),
            parameters: Default::default(),
            properties,
            palettes: vec![MapGenValue::String("apartment_palette".into())],
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
                self.transform_coordinates(&p.0.as_ivec2());
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
                self.transform_coordinates(&p.0.as_ivec2());

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
