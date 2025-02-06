pub(crate) mod handlers;

use glam::UVec2;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

pub type Identifier = String;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    character: char,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MapData {
    name: String,
    cells: HashMap<UVec2, Cell>,
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

        let mut state = serializer.serialize_struct("MapData", 2 + serialized_cells.len())?;

        state.serialize_field("cells", &serialized_cells)?;
        state.serialize_field("name", &self.name)?;

        state.end()
    }
}

impl MapData {
    pub fn new(name: String) -> Self {
        MapData {
            cells: Default::default(),
            name,
        }
    }
}
