pub(crate) mod handlers;
pub(crate) mod importing;
pub(crate) mod io;

use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::palettes::{CDDAPalette, Parameter};
use crate::cdda_data::{MapGenValue, TileLayer};
use crate::editor_data::Project;
use crate::map_data::handlers::{get_bg_from_sprite, get_fg_from_sprite, SpriteType};
use crate::tileset::legacy_tileset::MappedSprite;
use crate::tileset::{Tilesheet, TilesheetKind};
use crate::util::{
    CDDAIdentifier, DistributionInner, GetIdentifier, Load, ParameterIdentifier, Save,
};
use dyn_clone::{clone_trait_object, DynClone};
use glam::{IVec3, UVec2};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::EnumString;
use tokio::sync::MutexGuard;

pub const SPECIAL_EMPTY_CHAR: char = ' ';
pub const DEFAULT_MAP_DATA_SIZE: UVec2 = UVec2::new(24, 24);

pub trait Set: Debug + DynClone + Send + Sync {
    fn coordinates(&self) -> Vec<UVec2>;
    fn operation(&self) -> &SetOperation;

    fn map_ids(
        &self,
        coordinates: &UVec2,
        z: i32,
        mapped_sprites_lock: &mut MutexGuard<HashMap<IVec3, MappedSprite>>,
    ) {
        match self.operation() {
            SetOperation::Place { ty, id } => {
                let mut mapped_sprite = MappedSprite::default();

                match ty {
                    PlaceableSetType::Terrain => {
                        mapped_sprite.terrain = Some(id.clone());
                    }
                    PlaceableSetType::Furniture => {
                        mapped_sprite.furniture = Some(id.clone());
                    }
                    PlaceableSetType::Trap => {
                        mapped_sprite.trap = Some(id.clone());
                    }
                };

                mapped_sprites_lock.insert(
                    IVec3::new(coordinates.x as i32, coordinates.y as i32, z),
                    mapped_sprite.clone(),
                );
            }
            SetOperation::Remove { .. } => {}
            SetOperation::Radiation { .. } => {}
            SetOperation::Variable { .. } => {}
            SetOperation::Bash { .. } => {}
            SetOperation::Burn { .. } => {}
        }
    }
    fn get_fg_and_bg(
        &self,
        coordinates: UVec2,
        z: i32,
        tilesheet: &TilesheetKind,
        json_data: &DeserializedCDDAJsonData,
        mapped_sprites_lock: &mut MutexGuard<HashMap<IVec3, MappedSprite>>,
    ) -> (Option<SpriteType>, Option<SpriteType>) {
        match self.operation() {
            SetOperation::Place { ty, id } => {
                let sprite_kind = match tilesheet {
                    TilesheetKind::Legacy(l) => l.get_sprite(id, json_data),
                    TilesheetKind::Current(c) => c.get_sprite(id, json_data),
                };

                let layer = match ty {
                    PlaceableSetType::Terrain => TileLayer::Terrain,
                    PlaceableSetType::Furniture => TileLayer::Furniture,
                    PlaceableSetType::Trap => TileLayer::Trap,
                };

                let fg = get_fg_from_sprite(
                    id,
                    IVec3::new(coordinates.x as i32, coordinates.y as i32, z),
                    json_data,
                    layer.clone(),
                    &sprite_kind,
                    mapped_sprites_lock,
                );

                let bg = get_bg_from_sprite(
                    id,
                    IVec3::new(coordinates.x as i32, coordinates.y as i32, z),
                    json_data,
                    layer.clone(),
                    &sprite_kind,
                    mapped_sprites_lock,
                );

                (fg, bg)
            }
            SetOperation::Remove { .. } => (None, None),
            SetOperation::Radiation { .. } => (None, None),
            SetOperation::Variable { .. } => (None, None),
            SetOperation::Bash { .. } => (None, None),
            SetOperation::Burn { .. } => (None, None),
        }
    }

    fn get_sprites(
        &self,
        z: i32,
        tilesheet: &TilesheetKind,
        json_data: &DeserializedCDDAJsonData,
        mapped_sprites_lock: &mut MutexGuard<HashMap<IVec3, MappedSprite>>,
    ) -> Vec<SpriteType> {
        let mut sprites = vec![];

        // Like before, we need to map the ids before we generate the sprites
        for coordinates in self.coordinates() {
            self.map_ids(&coordinates, z, mapped_sprites_lock)
        }

        for coordinates in self.coordinates() {
            let (fg, bg) =
                self.get_fg_and_bg(coordinates, z, tilesheet, json_data, mapped_sprites_lock);

            if let Some(fg) = fg {
                sprites.push(fg);
            }

            if let Some(bg) = bg {
                sprites.push(bg)
            }
        }
        sprites
    }
}

clone_trait_object!(Set);

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

#[derive(Debug, Clone)]
pub enum SetOperation {
    Place {
        id: CDDAIdentifier,
        ty: PlaceableSetType,
    },
    Remove {
        ty: RemovableSetType,
    },
    Radiation {
        amount: (u32, u32),
    },
    Variable {
        id: CDDAIdentifier,
    },
    Bash {},
    Burn {},
}

#[derive(Debug, Clone)]
pub struct SetPoint {
    pub coordinates: UVec2,
    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

impl Set for SetPoint {
    fn coordinates(&self) -> Vec<UVec2> {
        vec![self.coordinates]
    }

    fn operation(&self) -> &SetOperation {
        &self.operation
    }
}

#[derive(Debug, Clone)]
pub struct SetLine {
    pub coordinates_from: UVec2,
    pub coordinates_to: UVec2,

    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

impl Set for SetLine {
    fn coordinates(&self) -> Vec<UVec2> {
        todo!()
    }

    fn operation(&self) -> &SetOperation {
        &self.operation
    }
}

#[derive(Debug, Clone)]
pub struct SetSquare {
    pub top_left: UVec2,
    pub bottom_right: UVec2,
    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

impl Set for SetSquare {
    fn coordinates(&self) -> Vec<UVec2> {
        let mut coordinates = vec![];

        for y in self.top_left.y..self.bottom_right.x {
            for x in self.top_left.x..self.bottom_right.x {
                coordinates.push(UVec2::new(x, y))
            }
        }

        coordinates
    }

    fn operation(&self) -> &SetOperation {
        &self.operation
    }
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

    #[serde(skip)]
    pub set: Vec<Arc<dyn Set>>,
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
            set: vec![],
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
        set: Vec<Arc<dyn Set>>,
    ) -> Self {
        Self {
            calculated_parameters: Default::default(),
            fill,
            parameters,
            palettes,
            terrain,
            furniture,
            cells,
            set,
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
