pub(crate) mod handlers;
pub(crate) mod importing;
pub(crate) mod io;

use crate::cdda_data::palettes::{CDDAPalette, Parameter};
use crate::cdda_data::MapGenValue;
use crate::editor_data::Project;
use crate::util::{
    CDDAIdentifier, DistributionInner, GetIdentifier, Load, ParameterIdentifier, Save,
};
use glam::UVec2;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::str::FromStr;
use strum_macros::EnumString;

pub const SPECIAL_EMPTY_CHAR: char = ' ';
pub const DEFAULT_MAP_DATA_SIZE: UVec2 = UVec2::new(24, 24);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommonPointFields {
    pub coordinates: UVec2,
    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PlaceableSetType {
    Terrain,
    Furniture,
    Trap,
}

#[derive(Debug, Clone, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RemovableSetType {
    Item,
    Field,
    Trap,
    Creature,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SetPoint {
    Place {
        id: CDDAIdentifier,
        ty: PlaceableSetType,
        common: CommonPointFields,
    },
    Remove {
        ty: RemovableSetType,
        common: CommonPointFields,
    },
    Radiation {
        common: CommonPointFields,
        amount: (u32, u32),
    },
    Variable {
        id: CDDAIdentifier,
        common: CommonPointFields,
    },
    Bash {
        common: CommonPointFields,
    },
    Burn {
        common: CommonPointFields,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommonLineFields {
    pub coordinates_from: UVec2,
    pub coordinates_to: UVec2,

    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SetLine {
    Place {
        id: CDDAIdentifier,
        ty: PlaceableSetType,
        common: CommonLineFields,
    },
    Remove {
        ty: RemovableSetType,
        common: CommonLineFields,
    },
    Radiation {
        common: CommonLineFields,
        amount: (u32, u32),
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommonSquareFields {
    pub top_left: UVec2,
    pub bottom_right: UVec2,
    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SetSquare {
    Place {
        id: CDDAIdentifier,
        ty: PlaceableSetType,
        common: CommonSquareFields,
    },
    Remove {
        ty: RemovableSetType,
        common: CommonSquareFields,
    },
    Radiation {
        common: CommonSquareFields,
        amount: (u32, u32),
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    pub character: char,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub cells: HashMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,

    pub calculated_parameters: HashMap<ParameterIdentifier, CDDAIdentifier>,
    pub parameters: HashMap<ParameterIdentifier, Parameter>,

    pub terrain: HashMap<char, MapGenValue>,
    pub furniture: HashMap<char, MapGenValue>,

    pub palettes: Vec<MapGenValue>,

    pub set_point: Vec<SetPoint>,
    pub set_line: Vec<SetLine>,
    pub set_square: Vec<SetSquare>,
}

impl Default for MapData {
    fn default() -> Self {
        let mut cells = HashMap::new();

        for y in 0..24 {
            for x in 0..24 {
                cells.insert(UVec2::new(x, y), Cell { character: ' ' });
            }
        }
        let fill = Some(DistributionInner::Normal(CDDAIdentifier::from("t_grass")));

        Self {
            cells,
            fill,
            calculated_parameters: Default::default(),
            parameters: Default::default(),
            terrain: Default::default(),
            furniture: Default::default(),
            palettes: Default::default(),
            set_line: Default::default(),
            set_point: Default::default(),
            set_square: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CDDAIdentifierGroup {
    pub terrain: Option<CDDAIdentifier>,
    pub furniture: Option<CDDAIdentifier>,
}

impl MapData {
    pub fn new(
        fill: Option<DistributionInner>,
        cells: HashMap<UVec2, Cell>,
        terrain: HashMap<char, MapGenValue>,
        furniture: HashMap<char, MapGenValue>,
        palettes: Vec<MapGenValue>,
        parameters: HashMap<ParameterIdentifier, Parameter>,
        set_point: Vec<SetPoint>,
        set_line: Vec<SetLine>,
        set_square: Vec<SetSquare>,
    ) -> Self {
        Self {
            calculated_parameters: Default::default(),
            fill,
            parameters,
            palettes,
            terrain,
            furniture,
            cells,
            set_square,
            set_point,
            set_line,
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

        state.end()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProjectContainer {
    pub data: Vec<Project>,
    pub current_project: Option<usize>,
}
