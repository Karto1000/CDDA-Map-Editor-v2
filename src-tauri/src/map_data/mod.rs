pub(crate) mod handlers;

use crate::util::{JSONSerializableUVec2, Load, Save};
use glam::UVec2;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error};
use std::path::PathBuf;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    character: char,
}

#[derive(Debug, Clone)]
pub struct MapData {
    name: String,
    cells: HashMap<UVec2, Cell>,
}

impl MapData {
    pub fn new(name: String) -> Self {
        MapData {
            cells: Default::default(),
            name,
        }
    }
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

#[derive(Debug, Clone, Deserialize)]
pub struct MapDataIntermediate {
    pub name: String,
    pub cells: HashMap<JSONSerializableUVec2, Cell>,
}

impl Into<MapData> for MapDataIntermediate {
    fn into(self) -> MapData {
        let cells = self
            .cells
            .into_iter()
            .map(|(key, value)| (key.0, value))
            .collect::<HashMap<UVec2, Cell>>();

        MapData {
            name: self.name,
            cells,
        }
    }
}

impl<'de> Deserialize<'de> for MapData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let intermediate: MapDataIntermediate = Deserialize::deserialize(deserializer)?;
        Ok(intermediate.into())
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct MapDataContainer {
    pub data: Vec<MapData>,
    pub current_map: Option<usize>,
}

pub struct MapDataLoader {
    pub path: PathBuf,
}

impl Load<MapData> for MapDataLoader {
    fn load(&self) -> Result<MapData, Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        serde_json::from_reader(reader).map_err(|e| e.into())
    }
}

pub struct MapDataSaver {
    pub path: PathBuf,
}

impl Save<MapData> for MapDataSaver {
    fn save(&self, data: &MapData) -> Result<(), Error> {
        let mut file = File::create(&self.path.join(&data.name))?;
        serde_json::to_writer(&mut file, data).map_err(|e| e.into())
    }
}
