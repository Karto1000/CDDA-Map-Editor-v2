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
use crate::features::program_data::{AdjacentTiles, ZLevel, DEFAULT_Z_LEVEL};
use crate::features::tileset::legacy_tileset::TilesheetCDDAId;
use crate::util::Rotation;
use anyhow::{anyhow, Error};
use cdda_lib::types::{
    CDDAIdentifier, DistributionInner, MapGenValue, NumberOrRange,
    ParameterIdentifier, Weighted,
};
use cdda_lib::{
    MapgenCellCoordinates, OvermapCoordinates, DEFAULT_REGION_SETTING_ENTRY,
    MAX_MAPGEN_HEIGHT, MAX_MAPGEN_WIDTH, MIN_MAPGEN_HEIGHT, MIN_MAPGEN_WIDTH,
    NULL_FURNITURE, NULL_TERRAIN,
};
use comfy_bounded_ints::types::Bound_u32;
use derive_more::Display;
use downcast_rs::{impl_downcast, Downcast, DowncastSend, DowncastSync};
use dyn_clone::{clone_trait_object, DynClone};
use futures_lite::StreamExt;
use glam::{IVec2, IVec3, UVec2};
use indexmap::IndexMap;
use log::warn;
use rand::{rng, Rng, RngCore};
use serde::de::{MapAccess, Visitor};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter, Write};
use std::sync::Arc;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use thiserror::Error;

pub const MAX_MAP_DATA_SIZE: UVec2 =
    UVec2::new(MAX_MAPGEN_WIDTH, MAX_MAPGEN_HEIGHT);

pub trait Place:
    Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend
{
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), anyhow::Error>;
}

clone_trait_object!(Place);
impl_downcast!(sync Place);

#[derive(Debug)]
pub struct Representation {
    pub id: TilesheetCDDAId,
    pub tile_layer: TileLayer,
}

// Things like terrain, furniture, monsters This allows us to get the Identifier
pub trait Property:
    Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend
{
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Result<(), anyhow::Error>;

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation>;

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
pub struct InstancedOvermapConfig {
    pub simulated_neighbors: HashMap<NeighborDirection, Vec<CDDAIdentifier>>,
}

impl Default for InstancedOvermapConfig {
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

        InstancedOvermapConfig {
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

pub fn serialize_properties<S>(
    properties: &HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let serialized_properties: HashMap<_, HashMap<_, _>> = properties
        .iter()
        .map(|(key, value)| {
            let serialized_inner = value
                .iter()
                .map(|(char_key, property)| (char_key, property.value()))
                .collect();
            (key, serialized_inner)
        })
        .collect();

    serialized_properties.serialize(serializer)
}

pub struct PropertiesVisitor;

impl<'de> Visitor<'de> for PropertiesVisitor {
    type Value = HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("any valid mapgen property")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut properties: HashMap<
            MappingKind,
            HashMap<char, Arc<dyn Property>>,
        > = HashMap::new();

        while let Some((kind, inner_map)) =
            map.next_entry::<MappingKind, HashMap<char, Value>>()?
        {
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

        Ok(properties)
    }
}

pub fn deserialize_properties<'de, D>(
    deserializer: D,
) -> Result<HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(PropertiesVisitor)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapGen {
    pub id: CDDAIdentifier,

    #[serde(
        deserialize_with = "cdda_lib::serde_vec::deserialize_indexmap_with_uvec2_keys",
        serialize_with = "cdda_lib::serde_vec::serialize_indexmap_with_uvec2_keys"
    )]
    pub cells: IndexMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,
    pub map_size: UVec2,
    pub predecessor: Option<CDDAIdentifier>,
    pub rotation: MapDataRotation,

    pub parameters: IndexMap<ParameterIdentifier, Parameter>,
    pub palettes: Vec<MapGenValue>,
    pub flags: HashSet<MapDataFlag>,

    #[serde(
        serialize_with = "serialize_properties",
        deserialize_with = "deserialize_properties"
    )]
    pub properties: HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>,

    #[serde(skip)]
    pub place: HashMap<MappingKind, Vec<PlaceOuter<Arc<dyn Place>>>>,
}

impl Default for MapGen {
    fn default() -> Self {
        let mut cells = IndexMap::new();

        for y in MIN_MAPGEN_WIDTH..MAX_MAPGEN_WIDTH {
            for x in MIN_MAPGEN_HEIGHT..MAX_MAPGEN_HEIGHT {
                cells.insert(UVec2::new(x, y), Cell { character: ' ' });
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
            id: CDDAIdentifier::from("map_data_default"),
            cells,
            fill,
            map_size: MAX_MAP_DATA_SIZE,
            predecessor: None,
            rotation: Default::default(),
            parameters: Default::default(),
            properties,
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

pub trait CalculateParameters: Clone + Debug {
    fn calculate_parameters(
        &self,
        mapgen: &MapGen,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Result<
        IndexMap<ParameterIdentifier, CDDAIdentifier>,
        CalculateParametersError,
    >;
}

#[derive(Debug, Clone)]
pub struct CalculateRandomParameters;

impl CalculateParameters for CalculateRandomParameters {
    fn calculate_parameters(
        &self,
        mapgen: &MapGen,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Result<
        IndexMap<ParameterIdentifier, CDDAIdentifier>,
        CalculateParametersError,
    > {
        let mut calculated_parameters = IndexMap::new();

        for (id, parameter) in mapgen.parameters.iter() {
            let calculated_value = parameter
                .default
                .distribution
                .get_random_identifier(&mut rng(), &calculated_parameters)?;

            calculated_parameters.insert(id.clone(), calculated_value);
        }

        for mapgen_value in mapgen.palettes.iter() {
            let id = mapgen_value
                .get_random_identifier(&mut rng(), &calculated_parameters)?;
            let palette = cdda_data.palettes.get(&id).ok_or(
                CalculateParametersError::MissingPalette(id.to_string()),
            )?;

            palette
                .calculate_parameters(&mut rng(), &cdda_data.palettes)?
                .into_iter()
                .for_each(|(palette_id, ident)| {
                    calculated_parameters.insert(palette_id, ident);
                });
        }

        Ok(calculated_parameters)
    }
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

impl MapGen {
    pub fn instantiate(
        &self,
        calculate_parameters_strategy: impl CalculateParameters,
        cdda_data: &DeserializedCDDAJsonData,
        // TODO: Error
    ) -> Result<InstantiatedMapgen<ParametersCalculated>, ()> {
        let mut instantiated_mapgen = InstantiatedMapgen::new()
            .calculate_parameters(
                calculate_parameters_strategy,
                self,
                cdda_data,
            )
            .unwrap();

        self.instantiate_tiles(&mut instantiated_mapgen, cdda_data)
            .unwrap();

        Ok(instantiated_mapgen)
    }

    fn instantiate_tiles(
        &self,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Result<(), GetMappedCDDAIdsError> {
        let region_settings = cdda_data
            .region_settings
            .get(&CDDAIdentifier(DEFAULT_REGION_SETTING_ENTRY.into()))
            .ok_or(GetMappedCDDAIdsError::MissingRegionSettings)?;

        let fill_terrain_sprite = match &self.fill {
            None => None,
            Some(id) => Some(replace_region_setting(
                &id.get_random_identifier(
                    &mut rng(),
                    &instantiation.calculated_parameters.0,
                )
                .unwrap(),
                region_settings,
            )),
        };

        // we need to calculate the predecessor_mapgen here before so we can replace it later
        match &self.predecessor {
            None => {},
            Some(predecessor_id) => {
                let predecessor =
                    cdda_data.overmap_terrains.get(predecessor_id)
                        .ok_or(GetMappedCDDAIdsError::MissingOvermapTerrainForPredecessor(predecessor_id.0.clone()))?;

                let predecessor_map_data = match &predecessor
                    .mapgen
                    .clone()
                    .unwrap_or_default()
                    .first()
                {
                    None => {
                        // This terrain is defined in a json file, so we can just search for it
                        cdda_data.map_data.get(predecessor_id).ok_or(GetMappedCDDAIdsError::MissingMapgenEntryForPredecessor(predecessor_id.0.clone()))?
                    }
                    Some(omtm) => cdda_data.map_data.get(&omtm.builtin).expect(
                        format!(
                            "Hardcoded Map data for the predecessor {} to exist",
                            omtm.builtin
                        ).as_str(),
                    ),
                };

                let instantiated_predecessor = predecessor_map_data
                    .instantiate(CalculateRandomParameters, cdda_data)
                    .unwrap();

                predecessor_map_data
                    .instantiate_tiles(instantiation, cdda_data)?;

                instantiation.instantiated_tiles =
                    instantiated_predecessor.instantiated_tiles;
            },
        }

        for (position, cell) in self.cells.iter() {
            let transformed_position = MapgenCellCoordinates::from(
                self.transform_coordinates(position),
            );

            // If there was no id added from the predecessor mapgen, we will add the fill sprite here
            match instantiation
                .instantiated_tiles
                .get_mut(&transformed_position)
            {
                None => {
                    let mut mapped_ids = InstantiatedTile::default();

                    mapped_ids.terrain = fill_terrain_sprite.clone().map(|s| {
                        MappedCDDAId::simple(TilesheetCDDAId::simple(
                            replace_region_setting(&s, region_settings),
                        ))
                    });

                    instantiation
                        .instantiated_tiles
                        .insert(transformed_position, mapped_ids);
                },
                Some(mapped_ids) => {
                    if mapped_ids.terrain.is_none() {
                        mapped_ids.terrain =
                            fill_terrain_sprite.clone().map(|s| {
                                MappedCDDAId::simple(TilesheetCDDAId::simple(
                                    replace_region_setting(&s, region_settings),
                                ))
                            })
                    }
                },
            };

            for mapping_kind in MappingKind::iter() {
                let property = match self
                    .get_random_property_from_character_recursive(
                        &mut rng(),
                        instantiation,
                        cell.character,
                        &mapping_kind,
                        cdda_data,
                    ) {
                    None => continue,
                    Some(p) => p,
                };

                property
                    .apply_to_instantiation(
                        self,
                        instantiation,
                        position.clone(),
                        cdda_data,
                    )
                    .unwrap();
            }
        }

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

                    match place.inner.apply_to_instantiation(
                        self,
                        instantiation,
                        transformed_position,
                        cdda_data,
                    ) {
                        Ok(_) => {},
                        Err(e) => {
                            warn!("{}", e)
                        },
                    }
                }
            }
        }

        Ok(())
    }

    fn get_random_property_from_character_recursive(
        &self,
        rng: &mut impl Rng,
        instantiation: &InstantiatedMapgen<ParametersCalculated>,
        character: char,
        mapping_kind: &MappingKind,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Option<Arc<dyn Property>> {
        match self.properties.get(&mapping_kind) {
            None => {},
            Some(p) => match p.get(&character) {
                None => {},
                Some(p) => return Some(Arc::clone(p)),
            },
        }

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value
                .get_random_identifier(
                    rng,
                    &instantiation.calculated_parameters.0,
                )
                .ok()?;

            let palette = cdda_data.palettes.get(&palette_id)?;

            if let Some(p) = palette
                .get_random_property_from_character_recursive(
                    rng,
                    instantiation,
                    character,
                    mapping_kind,
                    cdda_data,
                )
            {
                return Some(p);
            }
        }

        None
    }

    /// Transform 2d coordinates based on the rotation of the map
    /// This is used to rotate nested mapgens as well as vehicles and other tiles which need to be rotated
    fn transform_coordinates(&self, position: &UVec2) -> UVec2 {
        let (map_width, map_height) = (self.map_size.x, self.map_size.y);

        match self.rotation {
            MapDataRotation::Deg0 => position.clone(),
            MapDataRotation::Deg90 => {
                UVec2::new(map_height - 1 - position.y, position.x)
            },
            MapDataRotation::Deg180 => UVec2::new(
                map_width - 1 - position.x,
                map_height - 1 - position.y,
            ),
            MapDataRotation::Deg270 => {
                UVec2::new(position.y, map_width - 1 - position.x)
            },
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

impl From<TilesheetCDDAId> for MappedCDDAId {
    fn from(value: TilesheetCDDAId) -> Self {
        Self {
            tilesheet_id: value,
            rotation: Default::default(),
            is_broken: false,
            is_open: false,
        }
    }
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
pub struct InstantiatedTile {
    pub terrain: Option<MappedCDDAId>,
    pub furniture: Option<MappedCDDAId>,
    pub monster: Option<MappedCDDAId>,
    pub field: Option<MappedCDDAId>,
}

impl InstantiatedTile {
    pub fn override_none(&mut self, other: InstantiatedTile) {
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

pub trait CalculatedParametersMarker {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ParametersCalculated(
    pub IndexMap<ParameterIdentifier, CDDAIdentifier>,
);

impl CalculatedParametersMarker for ParametersCalculated {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ParametersNotCalculated;

impl CalculatedParametersMarker for ParametersNotCalculated {}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InstantiatedMapgen<
    Params: CalculatedParametersMarker = ParametersNotCalculated,
> {
    pub calculated_parameters: Params,
    pub instantiated_tiles: HashMap<MapgenCellCoordinates, InstantiatedTile>,
}

impl InstantiatedMapgen<ParametersNotCalculated> {
    pub fn new() -> InstantiatedMapgen<ParametersNotCalculated> {
        InstantiatedMapgen {
            calculated_parameters: ParametersNotCalculated,
            instantiated_tiles: HashMap::new(),
        }
    }

    pub fn calculate_parameters(
        self,
        calculate_parameters_strategy: impl CalculateParameters,
        from_mapgen: &MapGen,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> Result<
        InstantiatedMapgen<ParametersCalculated>,
        CalculateParametersError,
    > {
        let params = calculate_parameters_strategy
            .calculate_parameters(from_mapgen, cdda_data)?;

        Ok(InstantiatedMapgen {
            calculated_parameters: ParametersCalculated(params),
            instantiated_tiles: self.instantiated_tiles,
        })
    }
}

impl InstantiatedMapgen<ParametersCalculated> {
    pub fn get_or_create_tile_at_position(
        &mut self,
        position: MapgenCellCoordinates,
    ) -> &mut InstantiatedTile {
        self.instantiated_tiles
            .entry(position)
            .or_insert_with(InstantiatedTile::default)
    }

    pub fn place_nested(
        &mut self,
        position: MapgenCellCoordinates,
        instantiated_mapgen: InstantiatedMapgen<ParametersCalculated>,
    ) {
        for (local_nested_tile_coordinates, nested_tile) in
            instantiated_mapgen.instantiated_tiles.into_iter()
        {
            let global_coordinates = position + local_nested_tile_coordinates;

            match self.instantiated_tiles.get_mut(&global_coordinates) {
                None => {
                    self.instantiated_tiles
                        .insert(global_coordinates, nested_tile);
                },
                Some(t) => {
                    t.override_none(nested_tile);
                },
            }
        }
    }

    pub fn place_terrain(
        &mut self,
        position: MapgenCellCoordinates,
        terrain: impl Into<MappedCDDAId>,
    ) {
        let tile = self.get_or_create_tile_at_position(position);
        tile.terrain = Some(terrain.into());
    }

    pub fn place_furniture(
        &mut self,
        position: MapgenCellCoordinates,
        furniture: impl Into<MappedCDDAId>,
    ) {
        let tile = self.get_or_create_tile_at_position(position);
        tile.furniture = Some(furniture.into());
    }

    pub fn place_monster(
        &mut self,
        position: MapgenCellCoordinates,
        monster: impl Into<MappedCDDAId>,
    ) {
        let tile = self.get_or_create_tile_at_position(position);
        tile.monster = Some(monster.into());
    }

    pub fn place_field(
        &mut self,
        position: MapgenCellCoordinates,
        field: impl Into<MappedCDDAId>,
    ) {
        let tile = self.get_or_create_tile_at_position(position);
        tile.field = Some(field.into());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InstantiatedOvermap {
    pub config: InstancedOvermapConfig,
    pub instantiated_mapgens:
        HashMap<OvermapCoordinates, InstantiatedMapgen<ParametersCalculated>>,
}

impl InstantiatedOvermap {
    fn to_overmap_coordinates(&self, position: &UVec2) -> OvermapCoordinates {
        OvermapCoordinates {
            x: position.x / MAX_MAPGEN_WIDTH,
            y: position.y / MAX_MAPGEN_HEIGHT,
        }
    }

    fn get_id_from_mapped_sprites(
        &self,
        coords: &UVec2,
        layer: &TileLayer,
    ) -> Option<CDDAIdentifier> {
        let overmap_coordinates = self.to_overmap_coordinates(coords);
        let mapgen = self.instantiated_mapgens.get(&overmap_coordinates)?;

        let local_mapgen_coordinates = UVec2::new(
            coords.x % MAX_MAPGEN_WIDTH,
            coords.y % MAX_MAPGEN_HEIGHT,
        );
        let tile = mapgen
            .instantiated_tiles
            .get(&MapgenCellCoordinates::from(local_mapgen_coordinates))?;

        match layer {
            TileLayer::Terrain => {
                tile.terrain.clone().map(|v| v.tilesheet_id.id)
            },
            TileLayer::Furniture => {
                tile.furniture.clone().map(|v| v.tilesheet_id.id)
            },
            TileLayer::Monster => {
                tile.monster.clone().map(|v| v.tilesheet_id.id)
            },
            TileLayer::Field => tile.field.clone().map(|v| v.tilesheet_id.id),
        }
    }

    pub fn get_adjacent_tile_identifiers(
        &self,
        tile_overmap_coordinates: &UVec2,
        layer: &TileLayer,
    ) -> AdjacentTiles {
        let top_cords = tile_overmap_coordinates + UVec2::new(0, 1);
        let top = self.get_id_from_mapped_sprites(&top_cords, &layer);

        let right_cords = tile_overmap_coordinates + UVec2::new(1, 0);
        let right = self.get_id_from_mapped_sprites(&right_cords, &layer);

        let bottom = match tile_overmap_coordinates.y == 0 {
            true => None,
            false => {
                let bottom_cords = tile_overmap_coordinates - UVec2::new(0, 1);
                self.get_id_from_mapped_sprites(&bottom_cords, &layer)
            },
        };

        let left = match tile_overmap_coordinates.x == 0 {
            true => None,
            false => {
                let left_cords = tile_overmap_coordinates - UVec2::new(1, 0);
                self.get_id_from_mapped_sprites(&left_cords, &layer)
            },
        };

        AdjacentTiles {
            top,
            right,
            bottom,
            left,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct InstantiatedOvermapStack {
    pub instantiated_overmaps: HashMap<ZLevel, InstantiatedOvermap>,
}
