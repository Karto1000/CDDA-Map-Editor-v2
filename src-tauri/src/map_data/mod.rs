pub(crate) mod handlers;
pub(crate) mod importing;
pub(crate) mod io;

use crate::cdda_data::palettes::{CDDAPalette, Parameter};
use crate::cdda_data::MapGenValue;
use crate::util::{
    CDDAIdentifier, DistributionInner, GetIdentifier, JSONSerializableUVec2, Load,
    ParameterIdentifier, Save,
};
use glam::UVec2;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    pub character: char,
}

#[derive(Debug, Clone)]
pub struct MapData {
    pub name: String,
    pub cells: HashMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,

    pub calculated_parameters: HashMap<ParameterIdentifier, CDDAIdentifier>,
    pub parameters: HashMap<ParameterIdentifier, Parameter>,

    pub terrain: HashMap<char, MapGenValue>,
    pub furniture: HashMap<char, MapGenValue>,

    pub palettes: Vec<MapGenValue>,
}

#[derive(Debug, Clone)]
pub struct CDDAIdentifierGroup {
    pub terrain: Option<CDDAIdentifier>,
    pub furniture: Option<CDDAIdentifier>,
}

impl MapData {
    pub fn new(
        name: String,
        fill: Option<DistributionInner>,
        cells: HashMap<UVec2, Cell>,
        terrain: HashMap<char, MapGenValue>,
        furniture: HashMap<char, MapGenValue>,
        palettes: Vec<MapGenValue>,
        parameters: HashMap<ParameterIdentifier, Parameter>,
    ) -> Self {
        Self {
            calculated_parameters: Default::default(),
            fill,
            parameters,
            palettes,
            terrain,
            furniture,
            name,
            cells,
        }
    }

    pub fn calculate_parameters(&mut self, all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>) {
        let mut calculated_parameters = HashMap::new();

        for (id, parameter) in self.parameters.iter() {
            calculated_parameters.insert(
                id.clone(),
                parameter.default.distribution.get(&calculated_parameters),
            );
        }

        for mapgen_value in self.palettes.iter() {
            let id = mapgen_value.get_identifier(&calculated_parameters);
            let palette = all_palettes.get(&id).unwrap();

            palette
                .calculate_parameters(all_palettes)
                .into_iter()
                .for_each(|(palette_id, ident)| {
                    calculated_parameters.insert(palette_id, ident);
                });
        }

        self.calculated_parameters = calculated_parameters
    }

    pub fn get_terrain(
        &self,
        character: &char,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
    ) -> Option<CDDAIdentifier> {
        // If we find the terrain in the current map's terrain field, return that
        if let Some(id) = self.terrain.get(character) {
            return Some(id.get_identifier(&self.calculated_parameters));
        };

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(&self.calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) =
                palette.get_terrain(character, &self.calculated_parameters, all_palettes)
            {
                return Some(id);
            }
        }

        None
    }

    pub fn get_furniture(
        &self,
        character: &char,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
    ) -> Option<CDDAIdentifier> {
        if let Some(id) = self.furniture.get(character) {
            return Some(id.get_identifier(&self.calculated_parameters));
        };

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(&self.calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) =
                palette.get_furniture(character, &self.calculated_parameters, all_palettes)
            {
                return Some(id);
            }
        }

        None
    }

    pub fn get_identifiers(
        &self,
        character: &char,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
    ) -> CDDAIdentifierGroup {
        let terrain = self.get_terrain(character, all_palettes);
        let furniture = self.get_furniture(character, all_palettes);

        CDDAIdentifierGroup { terrain, furniture }
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
    pub fill: Option<DistributionInner>,
    pub cells: HashMap<JSONSerializableUVec2, Cell>,
    pub parameters: Option<HashMap<ParameterIdentifier, Parameter>>,
    pub palettes: Vec<MapGenValue>,
    pub terrain: HashMap<char, MapGenValue>,
    pub furniture: HashMap<char, MapGenValue>,
}

impl Into<MapData> for MapDataIntermediate {
    fn into(self) -> MapData {
        let cells = self
            .cells
            .into_iter()
            .map(|(key, value)| (key.0, value))
            .collect::<HashMap<UVec2, Cell>>();

        MapData::new(
            self.name,
            self.fill,
            cells,
            self.terrain,
            self.furniture,
            self.palettes,
            self.parameters.unwrap_or_else(|| HashMap::new()),
        )
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
