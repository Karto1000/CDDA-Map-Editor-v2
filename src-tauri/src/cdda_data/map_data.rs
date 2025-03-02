use crate::cdda_data::palettes::Parameter;
use crate::cdda_data::MapGenValue;
use crate::map_data::{Cell, MapData};
use crate::util::{MeabyParam, MeabyVec, ParameterIdentifier};
use glam::UVec2;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAMapDataObject {
    pub fill_ter: Option<MeabyParam>,
    pub rows: Vec<String>,
    #[serde(default)]
    pub palettes: Vec<MapGenValue>,
    #[serde(default)]
    pub terrain: HashMap<char, MapGenValue>,
    #[serde(default)]
    pub furniture: HashMap<char, MapGenValue>,
    #[serde(default)]
    pub parameters: HashMap<ParameterIdentifier, Parameter>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAMapData {
    pub method: String,
    pub om_terrain: MeabyVec<String>,
    pub weight: Option<i32>,
    pub object: CDDAMapDataObject,
}

impl CDDAMapData {
    pub fn into(self, name: String) -> MapData {
        let mut cells = HashMap::new();

        for (row_index, row) in self.object.rows.into_iter().enumerate() {
            for (column_index, character) in row.chars().enumerate() {
                cells.insert(
                    UVec2::new(column_index as u32, row_index as u32),
                    Cell { character },
                );
            }
        }

        MapData::new(
            name,
            self.object.fill_ter,
            cells,
            self.object.terrain,
            self.object.furniture,
            self.object.palettes,
            self.object.parameters,
        )
    }
}
