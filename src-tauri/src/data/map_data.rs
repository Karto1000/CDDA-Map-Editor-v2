use crate::data::io::DeserializedCDDAJsonData;
use crate::data::item::CDDAItemGroupInPlace;
use crate::data::map_data::IntoMapDataCollectionError::MissingNestedOmTerrain;
use crate::data::palettes::Parameter;
use crate::features::map::map_properties::ComputersProperty;
use crate::features::map::map_properties::CorpsesProperty;
use crate::features::map::map_properties::ToiletsProperty;
use crate::features::map::map_properties::TrapsProperty;
use crate::features::map::map_properties::VehiclesProperty;
use crate::features::map::map_properties::{
    FieldsProperty, FurnitureProperty, MonstersProperty, NestedProperty,
    SignsProperty, TerrainProperty,
};
use crate::features::map::map_properties::{GaspumpsProperty, ItemsProperty};
use crate::features::map::place::{PlaceFurniture, PlaceNested, PlaceTerrain};
use crate::features::map::SetTile;
use crate::features::map::DEFAULT_MAP_DATA_SIZE;
use crate::features::map::{
    Cell, MapData, MapDataFlag, MapGenNested, MappingKind, Place, Property,
};
use crate::features::program_data::{MapCoordinates, MapDataCollection};
use crate::util::UVec2JsonKey;
use cdda_lib::types::{
    CDDAIdentifier, CDDAString, DistributionInner, MapGenValue, MeabyVec,
    MeabyWeighted, NumberOrRange, ParameterIdentifier, Weighted,
};
use cdda_lib::{DEFAULT_MAP_HEIGHT, DEFAULT_MAP_WIDTH};
use glam::{IVec2, UVec2};
use indexmap::IndexMap;
use paste::paste;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OmTerrain {
    Single(String),
    Duplicate(Vec<String>),
    Nested(Vec<Vec<String>>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetIntermediate {
    line: Option<CDDAIdentifier>,
    point: Option<CDDAIdentifier>,
    square: Option<CDDAIdentifier>,
    id: Option<CDDAIdentifier>,
    x: Option<NumberOrRange<u32>>,
    y: Option<NumberOrRange<u32>>,
    z: Option<i32>,
    x2: Option<NumberOrRange<u32>>,
    y2: Option<NumberOrRange<u32>>,
    amount: Option<NumberOrRange<u32>>,
    chance: Option<u32>,
    repeat: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ReferenceOrInPlace<P> {
    Reference(CDDAIdentifier),
    InPlace(P),
}

impl<P> ReferenceOrInPlace<P> {
    pub fn ref_or(&self, value: impl Into<CDDAIdentifier>) -> CDDAIdentifier {
        match self {
            ReferenceOrInPlace::Reference(r) => r.clone(),
            ReferenceOrInPlace::InPlace(_) => value.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenItem {
    pub item: ReferenceOrInPlace<CDDAItemGroupInPlace>,
    pub chance: Option<NumberOrRange<u32>>,
    pub repeat: Option<NumberOrRange<u32>>,
    pub faction: Option<CDDAIdentifier>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MapGenMonsterType {
    Monster { monster: MapGenValue },
    MonsterGroup { group: MapGenValue },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenMonsters {
    #[serde(flatten)]
    pub id: MapGenMonsterType,
    pub chance: Option<NumberOrRange<u32>>,
    pub pack_size: Option<NumberOrRange<u32>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NeighborDirection {
    North,
    East,
    South,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Above,
    Below,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OmTerrainMatchType {
    Exact,
    Type,
    Subtype,
    Prefix,
    Contains,
}

#[derive(Debug, Clone, Serialize)]
pub struct OmTerrainMatch {
    pub om_terrain: CDDAIdentifier,
    pub om_terrain_match_type: OmTerrainMatchType,
}

impl<'de> Deserialize<'de> for OmTerrainMatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OmTerrainMatchVisitor;

        impl<'de> serde::de::Visitor<'de> for OmTerrainMatchVisitor {
            type Value = OmTerrainMatch;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(OmTerrainMatch {
                    om_terrain: value.into(),
                    om_terrain_match_type: OmTerrainMatchType::Contains,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut om_terrain = None;
                let mut om_terrain_match_type = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "om_terrain" => {
                            om_terrain = Some(map.next_value()?);
                        },
                        "om_terrain_match_type" => {
                            om_terrain_match_type = Some(map.next_value()?);
                        },
                        _ => {
                            let _ =
                                map.next_value::<serde::de::IgnoredAny>()?;
                        },
                    }
                }

                let om_terrain = om_terrain.ok_or_else(|| {
                    serde::de::Error::missing_field("om_terrain")
                })?;
                let om_terrain_match_type = om_terrain_match_type
                    .unwrap_or(OmTerrainMatchType::Contains);

                Ok(OmTerrainMatch {
                    om_terrain,
                    om_terrain_match_type,
                })
            }
        }

        deserializer.deserialize_any(OmTerrainMatchVisitor)
    }
}

impl OmTerrainMatch {
    pub fn matches_identifier(&self, ident: &CDDAIdentifier) -> bool {
        // https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/JSON/JSON_INFO.md#Starting-locations
        match self.om_terrain_match_type {
            OmTerrainMatchType::Exact => &self.om_terrain == ident,
            OmTerrainMatchType::Type => {
                // Strip any suffixes like rotation or linear directions
                let base_type = ident.0.split('_').next().unwrap_or("");
                let match_type =
                    self.om_terrain.0.split('_').next().unwrap_or("");
                base_type == match_type
            },
            OmTerrainMatchType::Subtype => {
                // Match base type and linear type suffix
                let parts: Vec<&str> = ident.0.split('_').collect();
                let match_parts: Vec<&str> =
                    self.om_terrain.0.split('_').collect();

                if parts.len() >= 2 && match_parts.len() >= 2 {
                    parts[0] == match_parts[0] && parts[1] == match_parts[1]
                } else {
                    false
                }
            },
            OmTerrainMatchType::Prefix => {
                // Must be complete prefix with underscore delimiter
                ident.0.starts_with(&self.om_terrain.0)
                    && (ident.0.len() == self.om_terrain.0.len()
                        || ident.0.chars().nth(self.om_terrain.0.len())
                            == Some('_'))
            },
            OmTerrainMatchType::Contains => {
                // Simple substring match
                ident.0.contains(&self.om_terrain.0)
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MapGenNestedIntermediate {
    Chunks {
        chunks: MeabyVec<MeabyWeighted<MapGenValue>>,
        neighbors: Option<HashMap<NeighborDirection, MeabyVec<OmTerrainMatch>>>,
    },
    ElseChunks {
        else_chunks: MeabyVec<MeabyWeighted<MapGenValue>>,
        neighbors: Option<HashMap<NeighborDirection, MeabyVec<OmTerrainMatch>>>,
    },
}

impl Into<MapGenNested> for MapGenNestedIntermediate {
    fn into(self) -> MapGenNested {
        let (transformed_chunks, neighbors, is_else) = match self {
            MapGenNestedIntermediate::Chunks { chunks, neighbors } => (
                chunks
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
                neighbors,
                false,
            ),
            MapGenNestedIntermediate::ElseChunks {
                else_chunks,
                neighbors,
            } => (
                else_chunks
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
                neighbors,
                true,
            ),
        };

        let neighbors = neighbors.map(|neighbors| {
            HashMap::from_iter(
                neighbors.into_iter().map(|(p, n)| (p, n.into_vec())),
            )
        });

        MapGenNested {
            neighbors,
            joins: None,
            chunks: transformed_chunks,
            invert_condition: is_else,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenField {
    pub field: CDDAIdentifier,
    pub intensity: Option<NumberOrRange<i32>>,
    pub age: Option<NumberOrRange<i32>>,
}

const fn default_security() -> i32 {
    1
}

pub type HardcodedAction = String;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenComputerAction {
    pub name: String,
    pub action: HardcodedAction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenComputerFailure {
    pub action: HardcodedAction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenComputer {
    pub name: String,

    #[serde(default = "default_security")]
    pub security: i32,

    #[serde(default)]
    pub options: Vec<MapGenComputerAction>,

    #[serde(default)]
    pub failures: Vec<MapGenComputerFailure>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenSign {
    pub signage: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenGaspump {
    pub fuel: Option<MapGenGaspumpFuelType>,
    pub amount: Option<NumberOrRange<i32>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MapGenGaspumpFuelType {
    Gasoline,
    Diesel,
    Jp8,
    Avgas,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MapGenTrap {
    TrapRef { trap: MapGenValue },
    MapGenValue(MapGenValue),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenCorpse {
    pub group: CDDAIdentifier,
    pub age: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenMonster {
    #[serde(rename = "mon")]
    pub monster: CDDAIdentifier,
    pub name: CDDAString,
    pub chance: Option<NumberOrRange<u32>>,
}

macro_rules! create_place_inner {
    (
        $name: ident,
        $inner_value: ty
    ) => {
        paste! {
            #[derive(Debug, Clone)]
            pub struct [<Place $name>] {
                pub property: [<$name Property>]
            }

            impl Place for [<Place $name>] {
                fn get_commands(
                    &self,
                    position: &IVec2,
                    map_data: &MapData,
                    json_data: &DeserializedCDDAJsonData,
                ) -> Option<Vec<SetTile>> {
                    self.property.get_commands(position, map_data, json_data)
                }
            }

            #[derive(Debug, Clone, Deserialize, Serialize)]
            pub struct [<PlaceInner $name>] {
                #[serde(flatten)]
                pub value: $inner_value,
            }

            impl Into<Arc<dyn Place>> for [<PlaceInner $name>] {
                fn into(self) -> Arc<dyn Place> {
                    Arc::new([<Place $name>] {
                        property: [<$name Property>]::from(self)
                    })
                }
            }
        }
    };
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerFurniture {
    #[serde(rename = "furn")]
    pub furniture_id: DistributionInner,
}

impl Into<Arc<dyn Place>> for PlaceInnerFurniture {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceFurniture {
            visible: FurnitureProperty {
                mapgen_value: self.furniture_id.clone().into(),
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerTerrain {
    #[serde(rename = "ter")]
    pub terrain_id: DistributionInner,
}

impl Into<Arc<dyn Place>> for PlaceInnerTerrain {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceTerrain {
            visible: TerrainProperty {
                mapgen_value: self.terrain_id.clone().into(),
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerNested {
    #[serde(flatten)]
    pub chunks: MapGenNestedIntermediate,
}

impl Into<Arc<dyn Place>> for PlaceInnerNested {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceNested {
            nested_property: NestedProperty {
                nested: vec![Weighted::new(self.chunks, 1)],
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerMonster {
    #[serde(flatten)]
    pub value: MapGenMonsters,
}

impl Into<Arc<dyn Place>> for PlaceInnerMonster {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceMonsters {
            property: MonstersProperty::from(self),
        })
    }
}

create_place_inner!(Items, MapGenItem);

create_place_inner!(Fields, MapGenField);

create_place_inner!(Computers, MapGenComputer);

create_place_inner!(Signs, MapGenSign);

create_place_inner!(Gaspumps, MapGenGaspump);

create_place_inner!(Monsters, MapGenMonsters);

create_place_inner!(Toilets, ());

create_place_inner!(Traps, MapGenTrap);
create_place_inner!(Vehicles, MapGenVehicle);
create_place_inner!(Corpses, MapGenCorpse);

const fn default_chance() -> i32 {
    100
}

const fn default_repeat() -> NumberOrRange<i32> {
    NumberOrRange::Number(1)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOuter<T> {
    #[serde(flatten)]
    pub inner: T,

    pub x: NumberOrRange<i32>,
    pub y: NumberOrRange<i32>,

    #[serde(default = "default_repeat")]
    pub repeat: NumberOrRange<i32>,

    #[serde(default = "default_chance")]
    pub chance: i32,
}

pub trait IntoArcDyn<T> {
    fn into_arc_dyn_place(
        value: T,
        local_x_coords: NumberOrRange<i32>,
        local_y_coords: NumberOrRange<i32>,
    ) -> Self;
}

// TODO: For some reason i cannot implement <T: Into<Arc<dyn Place>> From<PlaceOuter<T>> for PlaceOuter<Arc<dyn Place>>
// Since it conflicts with inbuilt implementations?

macro_rules! impl_from {
    (
        $identifier: ident
    ) => {
        impl IntoArcDyn<PlaceOuter<$identifier>>
            for PlaceOuter<Arc<dyn Place>>
        {
            fn into_arc_dyn_place(
                value: PlaceOuter<$identifier>,
                local_x_coords: NumberOrRange<i32>,
                local_y_coords: NumberOrRange<i32>,
            ) -> Self {
                PlaceOuter {
                    x: local_x_coords,
                    y: local_y_coords,
                    repeat: value.repeat,
                    chance: value.chance,
                    inner: value.inner.into(),
                }
            }
        }
    };
}

impl_from!(PlaceInnerFurniture);
impl_from!(PlaceInnerTerrain);
impl_from!(PlaceInnerItems);
impl_from!(PlaceInnerNested);
impl_from!(PlaceInnerToilets);
impl_from!(PlaceInnerFields);
impl_from!(PlaceInnerComputers);
impl_from!(PlaceInnerSigns);
impl_from!(PlaceInnerGaspumps);
impl_from!(PlaceInnerTraps);
impl_from!(PlaceInnerVehicles);
impl_from!(PlaceInnerCorpses);

impl IntoArcDyn<PlaceOuter<PlaceInnerMonster>> for PlaceOuter<Arc<dyn Place>> {
    fn into_arc_dyn_place(
        mut value: PlaceOuter<PlaceInnerMonster>,
        local_x_coords: NumberOrRange<i32>,
        local_y_coords: NumberOrRange<i32>,
    ) -> Self {
        // TODO: Special case since both PlaceOuter and Monster can have a chance field which
        // causes one of these to not be set when deserializing so we do this here
        value.inner.value.chance =
            Some(NumberOrRange::Number(value.chance as u32));

        PlaceOuter {
            x: local_x_coords,
            y: local_y_coords,
            repeat: value.repeat,
            chance: value.chance,
            inner: value.inner.into(),
        }
    }
}

impl IntoArcDyn<PlaceOuter<PlaceInnerMonsters>> for PlaceOuter<Arc<dyn Place>> {
    fn into_arc_dyn_place(
        mut value: PlaceOuter<PlaceInnerMonsters>,
        local_x_coords: NumberOrRange<i32>,
        local_y_coords: NumberOrRange<i32>,
    ) -> Self {
        // TODO: Special case since both PlaceOuter and Monster can have a chance field which
        // causes one of these to not be set when deserializing so we do this here
        value.inner.value.chance =
            Some(NumberOrRange::Number(value.chance as u32));

        PlaceOuter {
            x: local_x_coords,
            y: local_y_coords,
            repeat: value.repeat,
            chance: value.chance,
            inner: value.inner.into(),
        }
    }
}

impl<T> PlaceOuter<T> {
    pub fn coordinates(&self) -> IVec2 {
        IVec2::new(self.x.rand_number(), self.y.rand_number())
    }
}

macro_rules! map_data_object {
    (
        $name: ident,
        [REGULAR_FIELDS]
        $($r_field: ident: $r_ty: ty),*
        [FIELDS_WITH_PLACE]
        $($place_field: ident: $place_ty: ty),*
    ) => {
        paste! {
            #[derive(Debug, Clone, Deserialize, Serialize)]
            pub struct $name {
                $(
                    #[serde(default)]
                    pub $r_field: $r_ty,
                )*

                $(
                    #[serde(default)]
                    pub $place_field: HashMap<char, $place_ty>,

                    #[serde(default)]
                    pub [<place_ $place_field>]: Vec<PlaceOuter<[<PlaceInner$place_field: camel>]>>,
                )*
            }
        }
    };
}

map_data_object!(
    CDDAMapDataObjectCommonIntermediate,

    [REGULAR_FIELDS]
    palettes: Vec<MapGenValue>,
    parameters: IndexMap<ParameterIdentifier, Parameter>,
    set: Vec<SetIntermediate>,
    flags: HashSet<MapDataFlag>,
    predecessor_mapgen: Option<CDDAIdentifier>

    [FIELDS_WITH_PLACE]
    terrain: MapGenValue,
    furniture: MapGenValue,
    items: MeabyVec<MeabyWeighted<MapGenItem>>,
    monsters: MeabyVec<MeabyWeighted<MapGenMonsters>>,
    monster: MeabyVec<MeabyWeighted<MapGenMonsters>>,
    nested: MeabyVec<MeabyWeighted<MapGenNestedIntermediate>>,
    // Toilets do not have any data
    // TODO: we have to use Value here since there is a comment in one of the files with fails
    // to deserialize since a object with the key // cannot deserialize to a unit
    toilets: Value,
    fields: MeabyVec<MeabyWeighted<MapGenField>>,
    computers:  MeabyVec<MeabyWeighted<MapGenComputer>>,
    signs:  MeabyVec<MeabyWeighted<MapGenSign>>,
    gaspumps:  MeabyVec<MeabyWeighted<MapGenGaspump>>,
    traps:  MeabyVec<MeabyWeighted<MapGenTrap>>,
    vehicles: MeabyVec<MeabyWeighted<MapGenVehicle>>,
    corpses: MeabyVec<MeabyWeighted<MapGenCorpse>>
);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMapDataObjectIntermediate {
    pub fill_ter: Option<DistributionInner>,

    pub rows: Option<Vec<String>>,

    #[serde(rename = "mapgensize")]
    pub mapgen_size: Option<UVec2>,

    #[serde(flatten)]
    pub common: CDDAMapDataObjectCommonIntermediate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Weight {
    InPlace(i32),
    GlobalVal {
        global_val: CDDAIdentifier,
        default: i32,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IdCollection {
    pub update_mapgen_id: Option<CDDAIdentifier>,
    pub om_terrain: Option<OmTerrain>,
    pub nested_mapgen_id: Option<CDDAIdentifier>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMapDataIntermediate {
    pub update_mapgen_id: Option<CDDAIdentifier>,
    pub om_terrain: Option<OmTerrain>,
    pub nested_mapgen_id: Option<CDDAIdentifier>,

    pub weight: Option<Weight>,
    pub object: CDDAMapDataObjectIntermediate,
}

impl CDDAMapDataIntermediate {
    fn get_properties(
        &self,
    ) -> HashMap<MappingKind, HashMap<char, Arc<dyn Property>>> {
        let mut properties = HashMap::new();

        let mut terrain_map = HashMap::new();
        for (char, terrain) in self.object.common.terrain.clone() {
            let ter_prop = Arc::new(TerrainProperty {
                mapgen_value: terrain,
            });

            terrain_map.insert(char, ter_prop as Arc<dyn Property>);
        }

        let mut furniture_map = HashMap::new();
        for (char, furniture) in self.object.common.furniture.clone() {
            let fur_prop = Arc::new(FurnitureProperty {
                mapgen_value: furniture,
            });

            furniture_map.insert(char, fur_prop as Arc<dyn Property>);
        }

        let mut toilet_map = HashMap::new();
        for (char, _) in self.object.common.toilets.clone() {
            let toilet_prop = Arc::new(FurnitureProperty {
                mapgen_value: MapGenValue::String("f_toilet".into()),
            });

            toilet_map.insert(char, toilet_prop as Arc<dyn Property>);
        }

        let mut computer_map = HashMap::new();
        for (char, _) in self.object.common.computers.clone() {
            let ter_prop = Arc::new(FurnitureProperty {
                mapgen_value: MapGenValue::String("f_console".into()),
            });

            computer_map.insert(char, ter_prop as Arc<dyn Property>);
        }

        let mut monsters_map = HashMap::new();
        for (char, monster) in self.object.common.monsters.clone() {
            let monster_prop = Arc::new(MonstersProperty {
                monster: monster
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });

            monsters_map.insert(char, monster_prop as Arc<dyn Property>);
        }

        let mut monster_map = HashMap::new();
        for (char, monster) in self.object.common.monster.clone() {
            let monster_prop = Arc::new(MonstersProperty {
                monster: monster
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });

            monster_map.insert(char, monster_prop as Arc<dyn Property>);
        }

        let mut nested_map = HashMap::new();
        for (char, nested) in self.object.common.nested.clone() {
            let nested_terrain_prop = Arc::new(NestedProperty {
                nested: nested
                    .clone()
                    .into_vec()
                    .into_iter()
                    .map(|mw| mw.to_weighted())
                    .map(|w| Weighted::<MapGenNested>::new(w.data, w.weight))
                    .collect(),
            });
            nested_map.insert(char, nested_terrain_prop as Arc<dyn Property>);
        }

        let mut field_map = HashMap::new();
        for (char, field) in self.object.common.fields.clone() {
            let field_prop = Arc::new(FieldsProperty {
                field: field
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            field_map.insert(char, field_prop as Arc<dyn Property>);
        }

        let mut item_map = HashMap::new();
        for (char, items) in self.object.common.items.clone() {
            let item_prop = Arc::new(ItemsProperty {
                items: items
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            item_map.insert(char, item_prop as Arc<dyn Property>);
        }

        let mut sign_map = HashMap::new();
        for (char, sign) in self.object.common.signs.clone() {
            let sign_prop = Arc::new(SignsProperty {
                signs: sign
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            sign_map.insert(char, sign_prop as Arc<dyn Property>);
        }

        let mut gaspumps_map = HashMap::new();
        for (char, gaspump) in self.object.common.gaspumps.clone() {
            let gaspump_prop = Arc::new(GaspumpsProperty {
                gaspumps: gaspump
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            gaspumps_map.insert(char, gaspump_prop as Arc<dyn Property>);
        }

        let mut trap_map = HashMap::new();
        for (char, trap) in self.object.common.traps.clone() {
            let trap_prop = Arc::new(TrapsProperty {
                trap: trap
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .map(|v| {
                        let id = match v.data {
                            MapGenTrap::TrapRef { trap } => trap,
                            MapGenTrap::MapGenValue(v) => v,
                        };

                        Weighted::new(id, v.weight)
                    })
                    .collect(),
            });
            trap_map.insert(char, trap_prop as Arc<dyn Property>);
        }

        let mut vehicles_map = HashMap::new();
        for (char, vehicles) in self.object.common.vehicles.clone() {
            let vehicles_prop = Arc::new(VehiclesProperty {
                vehicles: vehicles
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            vehicles_map.insert(char, vehicles_prop as Arc<dyn Property>);
        }

        let mut corpses_map = HashMap::new();
        for (char, corpses) in self.object.common.corpses.clone() {
            let corpses_prop = Arc::new(CorpsesProperty {
                corpses: corpses
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            corpses_map.insert(char, corpses_prop as Arc<dyn Property>);
        }

        properties.insert(MappingKind::Terrain, terrain_map);
        properties.insert(MappingKind::Furniture, furniture_map);
        properties.insert(MappingKind::Monsters, monsters_map);
        properties.insert(MappingKind::Nested, nested_map);
        properties.insert(MappingKind::Field, field_map);
        properties.insert(MappingKind::ItemGroups, item_map);
        properties.insert(MappingKind::Computer, computer_map);
        properties.insert(MappingKind::Toilet, toilet_map);
        properties.insert(MappingKind::Sign, sign_map);
        properties.insert(MappingKind::Gaspump, gaspumps_map);
        properties.insert(MappingKind::Trap, trap_map);
        properties.insert(MappingKind::Vehicle, vehicles_map);
        properties.insert(MappingKind::Corpse, corpses_map);
        properties.insert(MappingKind::Monster, monster_map);

        properties
    }

    fn get_place(
        &self,
        map_coordinates: MapCoordinates,
    ) -> HashMap<MappingKind, Vec<PlaceOuter<Arc<dyn Place>>>> {
        let mut place: HashMap<MappingKind, Vec<PlaceOuter<Arc<dyn Place>>>> =
            HashMap::new();
        let map_size = self.object.mapgen_size.unwrap_or(DEFAULT_MAP_DATA_SIZE);

        macro_rules! insert_place {
            (
                $name: path,
                $multi: expr
            ) => {
                paste! {
                    let mut map_vec = vec![];

                    for mapping in self.object.common.[<place_ $multi:lower>].iter() {
                        let remapped_x = mapping.x.clone() - (map_coordinates.x * map_size.x as u32) as i32;
                        let remapped_y = mapping.y.clone() - (map_coordinates.y * map_size.y as u32) as i32;

                        if remapped_x >= 0 && remapped_x < map_size.x as i32 &&
                           remapped_y >= 0 && remapped_y < map_size.y as i32 {
                            map_vec.push(PlaceOuter::into_arc_dyn_place(
                                mapping.clone(),
                                remapped_x,
                                remapped_y
                            ))
                        }
                    }

                    place.insert(
                        MappingKind::$name,
                        map_vec
                    );
                }
            };
            (
                $name: path
            ) => {
                paste! {
                    let mut map_vec = vec![];

                    for mapping in self.object.common.[<place_ $name:lower>].iter() {
                        let remapped_x = mapping.x.clone() - (map_coordinates.x * DEFAULT_MAP_WIDTH as u32) as i32;
                        let remapped_y = mapping.y.clone() - (map_coordinates.y * DEFAULT_MAP_HEIGHT as u32) as i32;

                        if remapped_x >= 0 && remapped_x < DEFAULT_MAP_WIDTH as i32 &&
                           remapped_y >= 0 && remapped_y < DEFAULT_MAP_HEIGHT as i32 {
                            map_vec.push(PlaceOuter::into_arc_dyn_place(
                                mapping.clone(),
                                remapped_x,
                                remapped_y
                            ))
                        }
                    }

                    place.insert(
                        MappingKind::$name,
                        map_vec
                    );
                }
            };
        }

        insert_place!(Furniture);
        insert_place!(Toilet, toilets);
        insert_place!(Terrain);
        insert_place!(Computer, computers);
        insert_place!(Sign, signs);
        insert_place!(Trap, traps);
        insert_place!(Gaspump, gaspumps);
        insert_place!(Monsters);
        insert_place!(Monster);
        insert_place!(Nested);
        insert_place!(Field, fields);
        insert_place!(ItemGroups, items);
        insert_place!(Vehicle, vehicles);
        insert_place!(Corpse, corpses);

        place
    }
}

#[derive(Debug, Error)]
pub enum IntoMapDataCollectionError {
    #[error("Nested om Terrain is missing identifier")]
    MissingNestedOmTerrain,
}

impl TryInto<MapDataCollection> for CDDAMapDataIntermediate {
    type Error = IntoMapDataCollectionError;

    fn try_into(self) -> Result<MapDataCollection, Self::Error> {
        let mut map_data_collection = MapDataCollection::default();

        match &self.om_terrain {
            None => {},
            Some(om) => {
                if let OmTerrain::Nested(n) = om {
                    let num_rows = n.len();
                    let num_cols =
                        n.get(0).ok_or(MissingNestedOmTerrain)?.len();

                    for map_row_index in 0..num_rows {
                        for map_column_index in 0..num_cols {
                            let mut nested_cells: IndexMap<UVec2JsonKey, Cell> =
                                IndexMap::new();

                            match self.object.rows.clone() {
                                None => {
                                    for row in 0..DEFAULT_MAP_HEIGHT {
                                        for column in 0..DEFAULT_MAP_WIDTH {
                                            nested_cells.insert(
                                                UVec2::new(
                                                    column as u32,
                                                    row as u32,
                                                )
                                                .into(),
                                                Cell { character: ' ' },
                                            );
                                        }
                                    }
                                },
                                Some(map_row_slice) => {
                                    let new_slice: Vec<String> = map_row_slice
                                        [map_row_index * DEFAULT_MAP_HEIGHT
                                            ..map_row_index
                                                * DEFAULT_MAP_HEIGHT
                                                + DEFAULT_MAP_HEIGHT]
                                        .into_iter()
                                        .map(|str| {
                                            str.chars()
                                                .skip(
                                                    map_column_index
                                                        * DEFAULT_MAP_WIDTH,
                                                )
                                                .take(DEFAULT_MAP_WIDTH)
                                                .collect::<String>()
                                        })
                                        .collect();

                                    for (row_index, slice) in
                                        new_slice.into_iter().enumerate()
                                    {
                                        for (column_index, character) in
                                            slice.chars().enumerate()
                                        {
                                            nested_cells.insert(
                                                UVec2::new(
                                                    column_index as u32,
                                                    row_index as u32,
                                                )
                                                .into(),
                                                Cell { character },
                                            );
                                        }
                                    }
                                },
                            }

                            let map_coordinates = UVec2::new(
                                map_column_index as u32,
                                map_row_index as u32,
                            );
                            let mut map_data = MapData::default();

                            let properties = self.get_properties();
                            let place = self.get_place(map_coordinates.into());

                            map_data.cells = nested_cells;
                            map_data.properties = properties;
                            map_data.place = place;
                            map_data.parameters =
                                self.object.common.parameters.clone();
                            map_data.palettes =
                                self.object.common.palettes.clone();
                            map_data.fill = self.object.fill_ter.clone();
                            map_data.map_size = self
                                .object
                                .mapgen_size
                                .unwrap_or(DEFAULT_MAP_DATA_SIZE);
                            map_data.flags = self.object.common.flags.clone();
                            map_data.predecessor =
                                self.object.common.predecessor_mapgen.clone();

                            map_data_collection.maps.insert(
                                UVec2::new(
                                    map_column_index as u32,
                                    map_row_index as u32,
                                )
                                .into(),
                                map_data,
                            );
                        }
                    }

                    return Ok(map_data_collection);
                }
            },
        };

        let mut collection = MapDataCollection::default();
        let mut map_data = MapData::default();

        let properties = self.get_properties();
        let place = self.get_place(UVec2::ZERO.into());

        let mut cells: IndexMap<UVec2JsonKey, Cell> = IndexMap::new();

        for row in 0..self.object.mapgen_size.unwrap_or(DEFAULT_MAP_DATA_SIZE).y
        {
            for column in
                0..self.object.mapgen_size.unwrap_or(DEFAULT_MAP_DATA_SIZE).x
            {
                let char = match self.object.rows.as_ref() {
                    None => ' ',
                    Some(s) => match s.get(row as usize) {
                        None => ' ',
                        Some(row) => {
                            row.chars().nth(column as usize).unwrap_or(' ')
                        },
                    },
                };

                cells.insert(
                    UVec2::new(column, row).into(),
                    Cell { character: char },
                );
            }
        }

        map_data.cells = cells;
        map_data.properties = properties;
        map_data.place = place;
        map_data.parameters = self.object.common.parameters.clone();
        map_data.palettes = self.object.common.palettes.clone();
        map_data.fill = self.object.fill_ter.clone();
        map_data.map_size =
            self.object.mapgen_size.unwrap_or(DEFAULT_MAP_DATA_SIZE);
        map_data.flags = self.object.common.flags.clone();
        map_data.predecessor = self.object.common.predecessor_mapgen.clone();

        collection.maps.insert(UVec2::ZERO.into(), map_data);

        Ok(collection)
    }
}

fn default_rotation() -> MeabyVec<i32> {
    MeabyVec::Single(0)
}

// (optional, integer) Defaults to -1, light damage. A value of 0 equates to undamaged,
// 1 heavily damaged and 2 perfect condition with no faults and disabled security.
#[derive(Debug, Default, Clone)]
pub enum VehicleStatus {
    #[default]
    LightDamage = -1,
    Undamaged = 0,
    HeavilyDamaged = 1,
    Perfect = 2,
}

impl<'de> Deserialize<'de> for VehicleStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i32::deserialize(deserializer)?;
        Ok(match value {
            -1 => VehicleStatus::LightDamage,
            0 => VehicleStatus::Undamaged,
            1 => VehicleStatus::HeavilyDamaged,
            2 => VehicleStatus::Perfect,
            // TODO: Some values in the json files are above 2.
            // These don't seem to be handled anywhere so i'm guessing it just defaults to light damage.
            _ => VehicleStatus::LightDamage,
        })
    }
}

impl Serialize for VehicleStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.clone() as i32)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MapGenVehicle {
    pub vehicle: CDDAIdentifier,

    #[serde(default)]
    pub status: VehicleStatus,

    #[serde(default = "default_rotation")]
    pub rotation: MeabyVec<i32>,
}
