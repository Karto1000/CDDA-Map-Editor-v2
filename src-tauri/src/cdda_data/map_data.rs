use crate::cdda_data::palettes::Parameter;
use crate::cdda_data::MapGenValue;
use crate::map_data::{
    Cell, CommonLineFields, CommonPointFields, CommonSquareFields, MapData, PlaceableSetType,
    RemovableSetType, SetLine, SetPoint, SetSquare,
};
use crate::util::{CDDAIdentifier, DistributionInner, ParameterIdentifier};
use crate::{skip_err, skip_none};
use glam::{UVec2, Vec3};
use log::warn;
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

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

        let mut set_points = Vec::new();
        let mut set_line = Vec::new();
        let mut set_square = Vec::new();

        for set in self.object.set {
            if let Some(ty) = set.line {
                let x = skip_none!(set.x);
                let y = skip_none!(set.y);
                let x2 = skip_none!(set.x2);
                let y2 = skip_none!(set.y2);

                let common_fields = CommonLineFields {
                    coordinates_from: UVec2::new(x, y),
                    coordinates_to: UVec2::new(x2, y2),
                    z: set.z.unwrap_or(0),
                    chance: set.chance.unwrap_or(1),
                    repeat: set.repeat.unwrap_or((1, 1)),
                };

                match ty.0.as_str() {
                    "terrain" | "furniture" | "trap" => {
                        let id = skip_none!(set.id.clone());
                        let ty = skip_err!(PlaceableSetType::from_str(ty.0.as_str()));

                        set_line.push(SetLine::Place {
                            id,
                            ty,
                            common: common_fields,
                        });
                    }
                    "trap_remove" | "item_remove" | "field_remove" | "creature_remove" => {
                        let ty = skip_err!(RemovableSetType::from_str(ty.0.as_str()));

                        set_line.push(SetLine::Remove {
                            ty,
                            common: common_fields,
                        });
                    }
                    "radiation" => {
                        let amount = skip_none!(set.amount);

                        set_line.push(SetLine::Radiation {
                            common: common_fields,
                            amount,
                        })
                    }
                    _ => {
                        warn!("Unknown set line type {}; Skipping", ty);
                    }
                }
            }

            if let Some(ty) = set.point {
                let x = skip_none!(set.x);
                let y = skip_none!(set.y);

                let common_fields = CommonPointFields {
                    coordinates: UVec2::new(x, y),
                    z: set.z.unwrap_or(0),
                    chance: set.chance.unwrap_or(1),
                    repeat: set.repeat.unwrap_or((1, 1)),
                };

                match ty.0.as_str() {
                    "terrain" | "furniture" | "trap" => {
                        let id = skip_none!(set.id.clone());
                        let ty = skip_err!(PlaceableSetType::from_str(ty.0.as_str()));

                        set_points.push(SetPoint::Place {
                            id,
                            ty,
                            common: common_fields,
                        })
                    }
                    "trap_remove" | "item_remove" | "field_remove" | "creature_remove" => {
                        let ty = skip_err!(RemovableSetType::from_str(ty.0.as_str()));

                        set_points.push(SetPoint::Remove {
                            ty,
                            common: common_fields,
                        })
                    }
                    "radiation" => {
                        let amount = skip_none!(set.amount);

                        set_points.push(SetPoint::Radiation {
                            common: common_fields,
                            amount,
                        });
                    }
                    "variable" => {
                        let id = skip_none!(set.id.clone());

                        set_points.push(SetPoint::Variable {
                            id,
                            common: common_fields,
                        })
                    }
                    "bash" => set_points.push(SetPoint::Bash {
                        common: common_fields,
                    }),
                    "burn" => {
                        set_points.push(SetPoint::Burn {
                            common: common_fields,
                        });
                    }
                    _ => {
                        warn!("Unknown set point type {}; Skipping", ty)
                    }
                }
            }

            if let Some(ty) = set.square {
                let x = skip_none!(set.x);
                let y = skip_none!(set.y);
                let x2 = skip_none!(set.x2);
                let y2 = skip_none!(set.y2);

                let common_fields = CommonSquareFields {
                    top_left: UVec2::new(x, y),
                    bottom_right: UVec2::new(x2, y2),
                    z: set.z.unwrap_or(0),
                    chance: set.chance.unwrap_or(1),
                    repeat: set.repeat.unwrap_or((1, 1)),
                };

                match ty.0.as_str() {
                    "terrain" | "furniture" | "trap" => {
                        let id = skip_none!(set.id.clone());
                        let ty = skip_err!(PlaceableSetType::from_str(ty.0.as_str()));

                        set_square.push(SetSquare::Place {
                            id,
                            ty,
                            common: common_fields,
                        });
                    }
                    "trap_remove" | "item_remove" | "field_remove" | "creature_remove" => {
                        let ty = skip_err!(RemovableSetType::from_str(ty.0.as_str()));

                        set_square.push(SetSquare::Remove {
                            ty,
                            common: common_fields,
                        });
                    }
                    "radiation" => {
                        set_square.push(SetSquare::Radiation {
                            common: common_fields,
                            amount: set.amount.unwrap_or((1, 1)),
                        });
                    }
                    _ => {
                        warn!("Unknown set square type {}; Skipping", ty);
                    }
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
            set_points,
            set_line,
            set_square,
        )
    }
}
