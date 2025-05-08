use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::item::{
    CDDAItemGroup, CDDAItemGroupInPlace, CDDAItemGroupIntermediate, EntryGroupShortcut,
    EntryItemShortcut, Item, ItemEntry, ItemGroupSubtype,
};
use crate::cdda_data::palettes::Parameter;
use crate::cdda_data::{MapGenValue, NumberOrRange};
use crate::map::map_properties::representative::ItemProperty;
use crate::map::map_properties::visible::{
    FurnitureProperty, MonsterProperty, NestedProperty, TerrainProperty,
};
use crate::map::place::{PlaceFurniture, PlaceItems, PlaceMonster, PlaceNested, PlaceTerrain};
use crate::map::{
    Cell, MapData, MapDataFlag, MapGenNested, Place, PlaceableSetType, RemovableSetType,
    RepresentativeMappingKind, RepresentativeProperty, Set, SetLine, SetOperation, SetPoint,
    SetSquare, VisibleMappingKind, VisibleProperty, SPECIAL_EMPTY_CHAR,
};
use crate::util::{
    CDDAIdentifier, DistributionInner, MeabyVec, MeabyWeighted, ParameterIdentifier, Weighted,
};
use crate::warn;
use crate::{skip_err, skip_none};
use glam::{IVec2, UVec2};
use indexmap::IndexMap;
use paste::paste;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MapGenNestedIntermediate {
    Chunks {
        chunks: MeabyVec<MeabyWeighted<MapGenValue>>,
    },
    ElseChunks {
        else_chunks: MeabyVec<MeabyWeighted<MapGenValue>>,
    },
}

impl Into<MapGenNested> for MapGenNestedIntermediate {
    fn into(self) -> MapGenNested {
        let transformed_chunks = match self {
            MapGenNestedIntermediate::Chunks { chunks } => chunks
                .into_vec()
                .into_iter()
                .map(MeabyWeighted::to_weighted)
                .collect(),
            MapGenNestedIntermediate::ElseChunks { else_chunks } => else_chunks
                .into_vec()
                .into_iter()
                .map(MeabyWeighted::to_weighted)
                .collect(),
        };

        MapGenNested {
            chunks: transformed_chunks,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerFurniture {
    #[serde(rename = "furn")]
    furniture_id: CDDAIdentifier,
}

impl Into<Arc<dyn Place>> for PlaceInnerFurniture {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceFurniture {
            visible: FurnitureProperty {
                mapgen_value: MapGenValue::String(self.furniture_id.clone()),
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerTerrain {
    #[serde(rename = "ter")]
    terrain_id: CDDAIdentifier,
}

impl Into<Arc<dyn Place>> for PlaceInnerTerrain {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceTerrain {
            visible: TerrainProperty {
                mapgen_value: MapGenValue::String(self.terrain_id.clone()),
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerItems {
    item: CDDAIdentifier,
}

impl Into<Arc<dyn Place>> for PlaceInnerItems {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceItems {
            representative: ItemProperty { items: vec![] },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceInnerMonsters {
    monster: CDDAIdentifier,
    chance: Option<NumberOrRange<u32>>,
    density: Option<f32>,
}

impl Into<Arc<dyn Place>> for PlaceInnerMonsters {
    fn into(self) -> Arc<dyn Place> {
        Arc::new(PlaceMonster {
            visible: MonsterProperty {
                monster: MeabyVec::Single(MapGenMonster {
                    id: MapGenMonsterType::MonsterGroup {
                        group: MapGenValue::String(self.monster),
                    },
                    chance: self.chance,
                    pack_size: None,
                }),
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
                nested: self.chunks.into(),
            },
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceOuter<T> {
    pub x: NumberOrRange<i32>,
    pub y: NumberOrRange<i32>,

    #[serde(flatten)]
    pub inner: T,
}

// TODO: For some reason i cannot implement <T: Into<Arc<dyn Place>> From<PlaceOuter<T>> for PlaceOuter<Arc<dyn Place>>
// Since it conflicts with inbuilt implementations?
impl From<PlaceOuter<PlaceInnerFurniture>> for PlaceOuter<Arc<dyn Place>> {
    fn from(value: PlaceOuter<PlaceInnerFurniture>) -> Self {
        PlaceOuter {
            x: value.x,
            y: value.y,
            inner: value.inner.into(),
        }
    }
}

impl From<PlaceOuter<PlaceInnerTerrain>> for PlaceOuter<Arc<dyn Place>> {
    fn from(value: PlaceOuter<PlaceInnerTerrain>) -> Self {
        PlaceOuter {
            x: value.x,
            y: value.y,
            inner: value.inner.into(),
        }
    }
}

impl From<PlaceOuter<PlaceInnerMonsters>> for PlaceOuter<Arc<dyn Place>> {
    fn from(value: PlaceOuter<PlaceInnerMonsters>) -> Self {
        PlaceOuter {
            x: value.x,
            y: value.y,
            inner: value.inner.into(),
        }
    }
}

impl From<PlaceOuter<PlaceInnerNested>> for PlaceOuter<Arc<dyn Place>> {
    fn from(value: PlaceOuter<PlaceInnerNested>) -> Self {
        PlaceOuter {
            x: value.x,
            y: value.y,
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
    flags: HashSet<MapDataFlag>

    [FIELDS_WITH_PLACE]
    terrain: MapGenValue,
    furniture: MapGenValue,
    items: MeabyVec<MapGenItem>,
    monsters: MeabyVec<MapGenMonster>,
    nested: MapGenNestedIntermediate
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
pub struct CDDAMapDataIntermediate {
    pub method: String,

    pub update_mapgen_id: Option<CDDAIdentifier>,
    pub om_terrain: Option<OmTerrain>,
    pub nested_mapgen_id: Option<CDDAIdentifier>,

    pub weight: Option<i32>,
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

                // We need to reverse the iterators direction since we want the last row of the rows to
                // be at the bottom left so basically 0, 0
                for (row_index, row) in rows.into_iter().rev().enumerate() {
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

        let mut visible = HashMap::new();

        let mut terrain_map = HashMap::new();
        for (char, terrain) in self.object.common.terrain {
            let ter_prop = Arc::new(TerrainProperty {
                mapgen_value: terrain,
            });

            terrain_map.insert(char, ter_prop as Arc<dyn VisibleProperty>);
        }

        let mut furniture_map = HashMap::new();
        for (char, furniture) in self.object.common.furniture {
            let fur_prop = Arc::new(FurnitureProperty {
                mapgen_value: furniture,
            });

            furniture_map.insert(char, fur_prop as Arc<dyn VisibleProperty>);
        }

        let mut monster_map = HashMap::new();
        for (char, monster) in self.object.common.monsters {
            let monster_prop = Arc::new(MonsterProperty { monster });

            monster_map.insert(char, monster_prop as Arc<dyn VisibleProperty>);
        }

        let mut nested_map = HashMap::new();

        for (char, nested) in self.object.common.nested {
            let nested_terrain_prop = Arc::new(NestedProperty {
                nested: nested.clone().into(),
            });
            nested_map.insert(char, nested_terrain_prop as Arc<dyn VisibleProperty>);
        }

        visible.insert(VisibleMappingKind::Terrain, terrain_map);
        visible.insert(VisibleMappingKind::Furniture, furniture_map);
        visible.insert(VisibleMappingKind::Monster, monster_map);
        visible.insert(VisibleMappingKind::Nested, nested_map);

        let mut representative = HashMap::new();

        let mut item_map = HashMap::new();
        for (char, items) in self.object.common.items {
            let item_prop = Arc::new(ItemProperty {
                items: items.into_vec(),
            });
            item_map.insert(char, item_prop as Arc<dyn RepresentativeProperty>);
        }

        representative.insert(RepresentativeMappingKind::ItemGroups, item_map);

        let mut place = HashMap::new();

        place.insert(
            VisibleMappingKind::Furniture,
            self.object
                .common
                .place_furniture
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            VisibleMappingKind::Terrain,
            self.object
                .common
                .place_terrain
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            VisibleMappingKind::Monster,
            self.object
                .common
                .place_monsters
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        place.insert(
            VisibleMappingKind::Nested,
            self.object
                .common
                .place_nested
                .into_iter()
                .map(Into::into)
                .collect(),
        );

        let mut map_data = MapData::default();

        map_data.cells = cells;
        map_data.set = set_vec;
        map_data.visible = visible;
        map_data.representative = representative;
        map_data.place = place;
        map_data.parameters = self.object.common.parameters;
        map_data.palettes = self.object.common.palettes;
        map_data.fill = self.object.fill_ter;
        map_data.map_size = self.object.mapgen_size.unwrap_or(mapgen_size);
        map_data.flags = self.object.common.flags;

        map_data
    }
}
