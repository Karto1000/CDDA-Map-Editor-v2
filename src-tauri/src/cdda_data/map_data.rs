use crate::cdda_data::palettes::Parameter;
use crate::cdda_data::{MapGenValue, NumberOrRange};
use crate::map::{
    Cell, MapData, Place, PlaceFurniture, PlaceableSetType, RemovableSetType, Set, SetLine,
    SetOperation, SetPoint, SetSquare, VisibleMapping,
};
use crate::util::{CDDAIdentifier, DistributionInner, MeabyVec, ParameterIdentifier};
use crate::{skip_err, skip_none};
use glam::{UVec2, Vec3};
use indexmap::IndexMap;
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tauri::async_runtime::set;

pub const DEFAULT_MAP_WIDTH: usize = 24;
pub const DEFAULT_MAP_HEIGHT: usize = 24;

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
    amount: Option<(u32, u32)>,
    chance: Option<u32>,
    repeat: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MapGenItem {
    pub item: CDDAIdentifier,
    pub chance: Option<NumberOrRange<u32>>,
    pub repeat: Option<NumberOrRange<u32>>,
    pub faction: Option<CDDAIdentifier>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMapDataObject {
    pub fill_ter: Option<DistributionInner>,
    pub rows: Vec<String>,

    #[serde(default)]
    pub palettes: Vec<MapGenValue>,

    #[serde(default)]
    pub terrain: HashMap<char, MapGenValue>,

    #[serde(default)]
    pub furniture: HashMap<char, MapGenValue>,

    #[serde(default)]
    pub items: HashMap<char, MeabyVec<MapGenItem>>,

    #[serde(default)]
    pub place_furniture: Vec<PlaceFurniture>,

    #[serde(default)]
    pub monster: HashMap<char, Value>,

    #[serde(default)]
    pub monsters: HashMap<char, Value>,

    #[serde(default)]
    pub npcs: HashMap<char, Value>,

    #[serde(default)]
    pub loot: HashMap<char, Value>,

    #[serde(default)]
    pub sealed_item: HashMap<char, Value>,

    #[serde(default)]
    pub fields: HashMap<char, Value>,

    #[serde(default)]
    pub signs: HashMap<char, Value>,

    #[serde(default)]
    pub rubble: HashMap<char, Value>,

    #[serde(default)]
    pub liquids: HashMap<char, Value>,

    #[serde(default)]
    pub corpses: HashMap<char, Value>,

    #[serde(default)]
    pub computers: HashMap<char, Value>,

    #[serde(default)]
    pub nested: HashMap<char, Value>,

    #[serde(default)]
    pub toilets: HashMap<char, Value>,

    #[serde(default)]
    pub gaspumps: HashMap<char, Value>,

    #[serde(default)]
    pub vehicles: HashMap<char, Value>,

    #[serde(default)]
    pub traps: HashMap<char, Value>,

    #[serde(default)]
    pub graffiti: HashMap<char, Value>,

    #[serde(default)]
    pub parameters: IndexMap<ParameterIdentifier, Parameter>,

    #[serde(default)]
    pub set: Vec<SetIntermediate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMapData {
    pub method: String,
    pub om_terrain: OmTerrain,
    pub weight: Option<i32>,
    pub object: CDDAMapDataObject,
}

impl Into<MapData> for CDDAMapData {
    fn into(self) -> MapData {
        let mut cells = IndexMap::new();

        // We need to reverse the iterators direction since we want the last row of the rows to
        // be at the bottom left so basically 0, 0
        for (row_index, row) in self.object.rows.into_iter().rev().enumerate() {
            for (column_index, character) in row.chars().enumerate() {
                cells.insert(
                    UVec2::new(column_index as u32, row_index as u32),
                    Cell { character },
                );
            }
        }

        let mut set_vec: Vec<Arc<dyn Set>> = vec![];

        for set in self.object.set {
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
                        amount: set.amount.unwrap_or((1, 1)),
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

        let mut mappings = HashMap::new();

        mappings.insert(VisibleMapping::Terrain, self.object.terrain);
        mappings.insert(VisibleMapping::Furniture, self.object.furniture);

        let mut place = HashMap::new();
        place.insert(
            VisibleMapping::Furniture,
            self.object
                .place_furniture
                .into_iter()
                .map(|f| Arc::new(f) as Arc<dyn Place>)
                .collect(),
        );

        MapData::new(
            self.object.fill_ter,
            cells,
            mappings,
            self.object.palettes,
            HashMap::from_iter(
                self.object
                    .items
                    .into_iter()
                    .map(|(k, v)| (k, v.into_vec())),
            ),
            self.object.parameters,
            set_vec,
            place,
        )
    }
}
