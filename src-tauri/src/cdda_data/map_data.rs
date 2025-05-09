use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::item::{
    CDDAItemGroup, CDDAItemGroupInPlace, CDDAItemGroupIntermediate, EntryGroupShortcut,
    EntryItemShortcut, Item, ItemEntry, ItemGroupSubtype,
};
use crate::cdda_data::palettes::Parameter;
use crate::cdda_data::{MapGenValue, NumberOrRange};
use crate::map::map_properties::ComputerProperty;
use crate::map::map_properties::ToiletProperty;
use crate::map::map_properties::TrapsProperty;
use crate::map::map_properties::{
    FieldProperty, FurnitureProperty, MonstersProperty, NestedProperty, SignProperty,
    TerrainProperty,
};
use crate::map::map_properties::{GaspumpProperty, ItemsProperty};
use crate::map::place::{PlaceFurniture, PlaceNested, PlaceTerrain};
use crate::map::VisibleMappingCommand;
use crate::map::{
    Cell, MapData, MapDataFlag, MapGenNested, MappingKind, Place, PlaceableSetType, Property,
    RemovableSetType, Set, SetLine, SetOperation, SetPoint, SetSquare, SPECIAL_EMPTY_CHAR,
};
use crate::util::{
    CDDAIdentifier, DistributionInner, MeabyVec, MeabyWeighted, ParameterIdentifier, Weighted,
};
use crate::warn;
use crate::{skip_err, skip_none};
use glam::{IVec2, UVec2};
use indexmap::IndexMap;
use paste::paste;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

pub const DEFAULT_MAP_WIDTH: usize = 24;
pub const DEFAULT_MAP_HEIGHT: usize = 24;
pub const DEFAULT_CELL_CHARACTER: char = ' ';
pub const DEFAULT_MAP_ROWS: [&'static str; 24] = [
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
    "                        ",
];

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
pub struct MapGenMonster {
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

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
                        }
                        "om_terrain_match_type" => {
                            om_terrain_match_type = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let om_terrain =
                    om_terrain.ok_or_else(|| serde::de::Error::missing_field("om_terrain"))?;
                let om_terrain_match_type =
                    om_terrain_match_type.unwrap_or(OmTerrainMatchType::Contains);

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
                let match_type = self.om_terrain.0.split('_').next().unwrap_or("");
                base_type == match_type
            }
            OmTerrainMatchType::Subtype => {
                // Match base type and linear type suffix
                let parts: Vec<&str> = ident.0.split('_').collect();
                let match_parts: Vec<&str> = self.om_terrain.0.split('_').collect();

                if parts.len() >= 2 && match_parts.len() >= 2 {
                    parts[0] == match_parts[0] && parts[1] == match_parts[1]
                } else {
                    false
                }
            }
            OmTerrainMatchType::Prefix => {
                // Must be complete prefix with underscore delimiter
                ident.0.starts_with(&self.om_terrain.0)
                    && (ident.0.len() == self.om_terrain.0.len()
                        || ident.0.chars().nth(self.om_terrain.0.len()) == Some('_'))
            }
            OmTerrainMatchType::Contains => {
                // Simple substring match
                ident.0.contains(&self.om_terrain.0)
            }
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
            HashMap::from_iter(neighbors.into_iter().map(|(p, n)| (p, n.into_vec())))
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
                ) -> Option<Vec<VisibleMappingCommand>> {
                    self.property.get_commands(position, map_data, json_data)
                }

                fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
                    self.property.representation(json_data)
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
                nested: vec![self.chunks.into()],
            },
        })
    }
}

create_place_inner!(Items, MapGenItem);

create_place_inner!(Field, MapGenField);

create_place_inner!(Computer, MapGenComputer);

create_place_inner!(Sign, MapGenSign);

create_place_inner!(Gaspump, MapGenGaspump);

create_place_inner!(Monsters, MapGenMonster);

create_place_inner!(Toilet, ());

create_place_inner!(Traps, MapGenTrap);

const fn default_chance() -> i32 {
    100
}

const fn default_repeat() -> NumberOrRange<i32> {
    NumberOrRange::Number(1)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOuter<T> {
    pub x: NumberOrRange<i32>,
    pub y: NumberOrRange<i32>,

    #[serde(default = "default_repeat")]
    pub repeat: NumberOrRange<i32>,

    #[serde(default = "default_chance")]
    pub chance: i32,

    #[serde(flatten)]
    pub inner: T,
}

// TODO: For some reason i cannot implement <T: Into<Arc<dyn Place>> From<PlaceOuter<T>> for PlaceOuter<Arc<dyn Place>>
// Since it conflicts with inbuilt implementations?

macro_rules! impl_from {
    (
        $identifier: ident
    ) => {
        impl From<PlaceOuter<$identifier>> for PlaceOuter<Arc<dyn Place>> {
            fn from(value: PlaceOuter<$identifier>) -> Self {
                PlaceOuter {
                    x: value.x,
                    y: value.y,
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
impl_from!(PlaceInnerMonsters);
impl_from!(PlaceInnerNested);
impl_from!(PlaceInnerToilet);
impl_from!(PlaceInnerField);
impl_from!(PlaceInnerComputer);
impl_from!(PlaceInnerSign);
impl_from!(PlaceInnerGaspump);
impl_from!(PlaceInnerTraps);

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
        $(($place_field: ident, $place_field_singular: ident): $place_ty: ty),*
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
                    pub [<place_ $place_field>]: Vec<PlaceOuter<[<PlaceInner$place_field_singular: camel>]>>,
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
    flags: HashSet<MapDataFlag>

    [FIELDS_WITH_PLACE]
    (terrain, terrain): MapGenValue,
    (furniture, furniture): MapGenValue,
    (items, items): MeabyVec<MapGenItem>,
    (monsters, monsters): MeabyVec<MapGenMonster>,
    (nested, nested): MeabyVec<MapGenNestedIntermediate>,
    // Toilets do not have any data
    // TODO: we have to use Value here since there is a comment in one of the files with fails
    // to deserialize since a object with the key // cannot deserialize to a unit
    (toilets, toilet): Value,
    (fields, field): MapGenField,
    (computers, computer): MapGenComputer,
    (signs, sign): MapGenSign,
    (gaspumps, gaspump): MapGenGaspump,
    (traps, traps): MapGenTrap
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
pub struct CDDAMapDataIntermediate {
    pub method: String,

    pub update_mapgen_id: Option<CDDAIdentifier>,
    pub om_terrain: Option<OmTerrain>,
    pub nested_mapgen_id: Option<CDDAIdentifier>,

    pub weight: Option<Weight>,
    pub object: CDDAMapDataObjectIntermediate,
}

impl Into<MapData> for CDDAMapDataIntermediate {
    fn into(self) -> MapData {
        let mut cells = IndexMap::new();
        let mut mapgen_size = UVec2::new(DEFAULT_MAP_WIDTH as u32, DEFAULT_MAP_HEIGHT as u32);

        match &self.object.rows {
            None => {
                for y in 0..DEFAULT_MAP_HEIGHT {
                    for x in 0..DEFAULT_MAP_WIDTH {
                        cells.insert(
                            UVec2::new(x as u32, y as u32),
                            Cell {
                                character: SPECIAL_EMPTY_CHAR,
                            },
                        );
                    }
                }
            }
            Some(rows) => {
                mapgen_size.x = rows[0].len() as u32;
                mapgen_size.y = rows.len() as u32;

                for (row_index, row) in rows.into_iter().enumerate() {
                    for (column_index, character) in row.chars().enumerate() {
                        cells.insert(
                            UVec2::new(column_index as u32, row_index as u32),
                            Cell { character },
                        );
                    }
                }
            }
        }

        let mut set_vec: Vec<Arc<dyn Set>> = vec![];

        for set in self.object.common.set {
            if let Some(ty) = set.line {
                let x = skip_none!(set.x);
                let y = skip_none!(set.y);
                let x2 = skip_none!(set.x2);
                let y2 = skip_none!(set.y2);

                let operation = match ty.0.as_str() {
                    "terrain" | "furniture" | "trap" => {
                        let id = skip_none!(set.id.clone());
                        let ty = skip_err!(PlaceableSetType::from_str(ty.0.as_str()));

                        Some(SetOperation::Place { id, ty })
                    }
                    "trap_remove" | "item_remove" | "field_remove" | "creature_remove" => {
                        let ty = skip_err!(RemovableSetType::from_str(ty.0.as_str()));

                        Some(SetOperation::Remove { ty })
                    }
                    "radiation" => {
                        let amount = skip_none!(set.amount);

                        Some(SetOperation::Radiation { amount })
                    }
                    _ => {
                        warn!("Unknown set line type {}; Skipping", ty);
                        None
                    }
                };

                if let Some(operation) = operation {
                    let set_line = SetLine {
                        from_x: x,
                        from_y: y,
                        to_x: x2,
                        to_y: y2,
                        z: set.z.unwrap_or(0),
                        chance: set.chance.unwrap_or(1),
                        repeat: set.repeat.unwrap_or((0, 1)),
                        operation,
                    };

                    set_vec.push(Arc::new(set_line));
                }
            } else if let Some(ty) = set.point {
                let x = skip_none!(set.x);
                let y = skip_none!(set.y);

                let operation = match ty.0.as_str() {
                    "terrain" | "furniture" | "trap" => {
                        let id = skip_none!(set.id.clone());
                        let ty = skip_err!(PlaceableSetType::from_str(ty.0.as_str()));

                        Some(SetOperation::Place { id, ty })
                    }
                    "trap_remove" | "item_remove" | "field_remove" | "creature_remove" => {
                        let ty = skip_err!(RemovableSetType::from_str(ty.0.as_str()));

                        Some(SetOperation::Remove { ty })
                    }
                    "radiation" => {
                        let amount = skip_none!(set.amount);

                        Some(SetOperation::Radiation { amount })
                    }
                    "variable" => {
                        let id = skip_none!(set.id.clone());

                        Some(SetOperation::Variable { id })
                    }
                    "bash" => Some(SetOperation::Bash {}),
                    "burn" => Some(SetOperation::Burn {}),
                    _ => {
                        warn!("Unknown set point type {}; Skipping", ty);
                        None
                    }
                };

                if let Some(operation) = operation {
                    let set_point = SetPoint {
                        x,
                        y,
                        z: set.z.unwrap_or(0),
                        chance: set.chance.unwrap_or(1),
                        repeat: set.repeat.unwrap_or((1, 1)),
                        operation,
                    };

                    set_vec.push(Arc::new(set_point))
                }
            } else if let Some(ty) = set.square {
                let x = skip_none!(set.x);
                let y = skip_none!(set.y);
                let x2 = skip_none!(set.x2);
                let y2 = skip_none!(set.y2);

                let operation = match ty.0.as_str() {
                    "terrain" | "furniture" | "trap" => {
                        let id = skip_none!(set.id.clone());
                        let ty = skip_err!(PlaceableSetType::from_str(ty.0.as_str()));

                        Some(SetOperation::Place { id, ty })
                    }
                    "trap_remove" | "item_remove" | "field_remove" | "creature_remove" => {
                        let ty = skip_err!(RemovableSetType::from_str(ty.0.as_str()));

                        Some(SetOperation::Remove { ty })
                    }
                    "radiation" => Some(SetOperation::Radiation {
                        amount: set.amount.unwrap_or(NumberOrRange::Number(1)),
                    }),
                    _ => {
                        warn!("Unknown set square type {}; Skipping", ty);
                        None
                    }
                };

                if let Some(operation) = operation {
                    let set_square = SetSquare {
                        top_left_x: x,
                        top_left_y: y,
                        bottom_right_x: x2,
                        bottom_right_y: y2,
                        z: set.z.unwrap_or(0),
                        chance: set.chance.unwrap_or(1),
                        repeat: set.repeat.unwrap_or((1, 1)),
                        operation,
                    };

                    set_vec.push(Arc::new(set_square))
                }
            }
        }

        let mut properties = HashMap::new();

        let mut terrain_map = HashMap::new();
        for (char, terrain) in self.object.common.terrain {
            let ter_prop = Arc::new(TerrainProperty {
                mapgen_value: terrain,
            });

            terrain_map.insert(char, ter_prop as Arc<dyn Property>);
        }

        let mut furniture_map = HashMap::new();
        for (char, furniture) in self.object.common.furniture {
            let fur_prop = Arc::new(FurnitureProperty {
                mapgen_value: furniture,
            });

            furniture_map.insert(char, fur_prop as Arc<dyn Property>);
        }

        let mut toilet_map = HashMap::new();
        for (char, _) in self.object.common.toilets {
            let toilet_prop = Arc::new(FurnitureProperty {
                mapgen_value: MapGenValue::String("f_toilet".into()),
            });

            toilet_map.insert(char, toilet_prop as Arc<dyn Property>);
        }

        let mut computer_map = HashMap::new();
        for (char, _) in self.object.common.computers {
            let ter_prop = Arc::new(FurnitureProperty {
                mapgen_value: MapGenValue::String("f_console".into()),
            });

            computer_map.insert(char, ter_prop as Arc<dyn Property>);
        }

        let mut monster_map = HashMap::new();
        for (char, monster) in self.object.common.monsters {
            let monster_prop = Arc::new(MonstersProperty { monster });

            monster_map.insert(char, monster_prop as Arc<dyn Property>);
        }

        let mut nested_map = HashMap::new();
        for (char, nested) in self.object.common.nested {
            let nested_terrain_prop = Arc::new(NestedProperty {
                nested: nested
                    .clone()
                    .into_vec()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            });
            nested_map.insert(char, nested_terrain_prop as Arc<dyn Property>);
        }

        let mut field_map = HashMap::new();
        for (char, field) in self.object.common.fields {
            let field_prop = Arc::new(FieldProperty { field });
            field_map.insert(char, field_prop as Arc<dyn Property>);
        }

        let mut item_map = HashMap::new();
        for (char, items) in self.object.common.items {
            let item_prop = Arc::new(ItemsProperty {
                items: items.into_vec(),
            });
            item_map.insert(char, item_prop as Arc<dyn Property>);
        }

        let mut sign_map = HashMap::new();
        for (char, sign) in self.object.common.signs {
            let sign_prop = Arc::new(SignProperty {
                text: sign.signage,
                snippet: sign.snippet,
            });
            sign_map.insert(char, sign_prop as Arc<dyn Property>);
        }

        let mut gaspumps_map = HashMap::new();
        for (char, gaspump) in self.object.common.gaspumps {
            let gaspump_prop = Arc::new(GaspumpProperty {
                amount: gaspump.amount,
                fuel: gaspump.fuel,
            });
            gaspumps_map.insert(char, gaspump_prop as Arc<dyn Property>);
        }

        let mut trap_map = HashMap::new();
        for (char, trap) in self.object.common.traps {
            let transformed_trap = match trap {
                MapGenTrap::TrapRef { trap } => trap,
                MapGenTrap::MapGenValue(mg) => mg,
            };

            let trap_prop = Arc::new(TrapsProperty {
                trap: transformed_trap,
            });
            trap_map.insert(char, trap_prop as Arc<dyn Property>);
        }

        properties.insert(MappingKind::Terrain, terrain_map);
        properties.insert(MappingKind::Furniture, furniture_map);
        properties.insert(MappingKind::Monster, monster_map);
        properties.insert(MappingKind::Nested, nested_map);
        properties.insert(MappingKind::Field, field_map);
        properties.insert(MappingKind::ItemGroups, item_map);
        properties.insert(MappingKind::Computer, computer_map);
        properties.insert(MappingKind::Toilet, toilet_map);
        properties.insert(MappingKind::Sign, sign_map);
        properties.insert(MappingKind::Gaspump, gaspumps_map);
        properties.insert(MappingKind::Trap, trap_map);

        let mut place: HashMap<MappingKind, Vec<PlaceOuter<Arc<dyn Place>>>> = HashMap::new();

        place.insert(
            MappingKind::Furniture,
            self.object
                .common
                .place_furniture
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Toilet,
            self.object
                .common
                .place_toilets
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Terrain,
            self.object
                .common
                .place_terrain
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Computer,
            self.object
                .common
                .place_computers
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Sign,
            self.object
                .common
                .place_signs
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Trap,
            self.object
                .common
                .place_traps
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Gaspump,
            self.object
                .common
                .place_gaspumps
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Monster,
            self.object
                .common
                .place_monsters
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Nested,
            self.object
                .common
                .place_nested
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::Field,
            self.object
                .common
                .place_fields
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            MappingKind::ItemGroups,
            self.object
                .common
                .place_items
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        let mut map_data = MapData::default();

        map_data.cells = cells;
        map_data.set = set_vec;
        map_data.properties = properties;
        map_data.place = place;
        map_data.parameters = self.object.common.parameters;
        map_data.palettes = self.object.common.palettes;
        map_data.fill = self.object.fill_ter;
        map_data.map_size = self.object.mapgen_size.unwrap_or(mapgen_size);
        map_data.flags = self.object.common.flags;

        map_data
    }
}
