use crate::cdda_data::palettes::Parameter;
use crate::cdda_data::MapGenValue;
use crate::map_data::{
    Cell, MapData, PlaceableSetType, RemovableSetType, Set, SetLine, SetOperation, SetPoint,
    SetSquare,
};
use crate::util::{CDDAIdentifier, DistributionInner, ParameterIdentifier};
use crate::{skip_err, skip_none};
use glam::{UVec2, Vec3};
use log::warn;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tauri::async_runtime::set;

pub const DEFAULT_MAP_WIDTH: usize = 24;
pub const DEFAULT_MAP_HEIGHT: usize = 24;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OmTerrain {
    Single(String),
    Duplicate(Vec<String>),
    Nested(Vec<Vec<String>>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetIntermediate {
    line: Option<CDDAIdentifier>,
    point: Option<CDDAIdentifier>,
    square: Option<CDDAIdentifier>,
    id: Option<CDDAIdentifier>,
    x: Option<u32>,
    y: Option<u32>,
    z: Option<i32>,
    x2: Option<u32>,
    y2: Option<u32>,
    amount: Option<(u32, u32)>,
    chance: Option<u32>,
    repeat: Option<(u32, u32)>,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub parameters: HashMap<ParameterIdentifier, Parameter>,
    #[serde(default)]
    pub set: Vec<SetIntermediate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAMapData {
    pub method: String,
    pub om_terrain: OmTerrain,
    pub weight: Option<i32>,
    pub object: CDDAMapDataObject,
}

impl Into<MapData> for CDDAMapData {
    fn into(self) -> MapData {
        let mut cells = HashMap::new();

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
                        coordinates_from: UVec2::new(x, y),
                        coordinates_to: UVec2::new(x2, y2),
                        z: set.z.unwrap_or(0),
                        chance: set.chance.unwrap_or(1),
                        repeat: set.repeat.unwrap_or((1, 1)),
                        operation,
                    };

                    set_vec.push(Arc::new(set_line));
                }
            }

            if let Some(ty) = set.point {
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
                        coordinates: UVec2::new(x, y),
                        z: set.z.unwrap_or(0),
                        chance: set.chance.unwrap_or(1),
                        repeat: set.repeat.unwrap_or((1, 1)),
                        operation,
                    };

                    set_vec.push(Arc::new(set_point))
                }
            }

            if let Some(ty) = set.square {
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
                        top_left: UVec2::new(x, y),
                        bottom_right: UVec2::new(x2, y2),
                        z: set.z.unwrap_or(0),
                        chance: set.chance.unwrap_or(1),
                        repeat: set.repeat.unwrap_or((1, 1)),
                        operation,
                    };

                    set_vec.push(Arc::new(set_square))
                }
            }
        }

        MapData::new(
            self.object.fill_ter,
            cells,
            self.object.terrain,
            self.object.furniture,
            self.object.palettes,
            self.object.parameters,
            set_vec,
        )
    }
}
