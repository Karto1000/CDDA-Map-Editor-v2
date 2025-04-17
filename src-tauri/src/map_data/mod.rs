pub(crate) mod handlers;
pub(crate) mod importing;
pub(crate) mod io;

use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::palettes::{CDDAPalette, Parameter};
use crate::cdda_data::{MapGenValue, NumberOrRange, TileLayer};
use crate::editor_data::Project;
use crate::map_data::handlers::{get_sprite_type_from_sprite, SpriteType};
use crate::tileset::legacy_tileset::MappedSprite;
use crate::tileset::{AdjacentSprites, Tilesheet, TilesheetKind};
use crate::util::{
    bresenham_line, CDDAIdentifier, DistributionInner, GetIdentifier, Load, ParameterIdentifier,
    Save,
};
use crate::RANDOM;
use dyn_clone::{clone_trait_object, DynClone};
use glam::{IVec3, UVec2};
use indexmap::IndexMap;
use rand::Rng;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use strum_macros::EnumString;

pub const SPECIAL_EMPTY_CHAR: char = ' ';
pub const DEFAULT_MAP_DATA_SIZE: UVec2 = UVec2::new(24, 24);

pub trait Set: Debug + DynClone + Send + Sync {
    fn coordinates(&self) -> Vec<UVec2>;
    fn operation(&self) -> &SetOperation;
    fn tile_layer(&self) -> TileLayer {
        match self.operation() {
            SetOperation::Place { ty, .. } => match ty {
                PlaceableSetType::Terrain => TileLayer::Terrain,
                PlaceableSetType::Furniture => TileLayer::Furniture,
                PlaceableSetType::Trap => TileLayer::Trap,
            },
            // TODO: Default to terrain, change
            _ => TileLayer::Terrain,
        }
    }

    fn get_mapped_sprites(&self, chosen_coordinates: Vec<IVec3>) -> HashMap<IVec3, MappedSprite> {
        let mut new_mapped_sprites = HashMap::new();

        for coordinates in chosen_coordinates {
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

                    new_mapped_sprites.insert(coordinates, mapped_sprite.clone());
                }
                _ => {}
            }
        }

        new_mapped_sprites
    }

    fn get_sprites(
        &self,
        chosen_coordinates: Vec<IVec3>,
        adjacent_sprites: Vec<AdjacentSprites>,
        tilesheet: &TilesheetKind,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<SpriteType> {
        let mut sprites = vec![];

        for (coordinates, adjacent_sprites) in chosen_coordinates.into_iter().zip(adjacent_sprites)
        {
            let (fg, bg) = match self.operation() {
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

                    let fg_bg = get_sprite_type_from_sprite(
                        id,
                        coordinates,
                        &adjacent_sprites,
                        layer.clone(),
                        &sprite_kind,
                        json_data,
                    );

                    fg_bg
                }
                _ => (None, None),
            };

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

pub trait Place: Debug + DynClone + Send + Sync {
    fn coordinates(&self) -> UVec2;
    fn tile_layer(&self) -> TileLayer;

    fn get_sprites(
        &self,
        coordinates: IVec3,
        adjacent_sprites: &AdjacentSprites,
        tilesheet: &TilesheetKind,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<SpriteType>;

    fn get_mapped_sprites(
        &self,
        chosen_coordinates: &UVec2,
        z: i32,
    ) -> HashMap<IVec3, MappedSprite>;
}

clone_trait_object!(Place);

#[derive(Debug, Clone, Deserialize, Hash, PartialOrd, PartialEq, Eq, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Mapping {
    Terrain,
    Furniture,
    Monster,
    Monsters,
    Npcs,
    Items,
    Loot,
    SealedItem,
    Fields,
    Signs,
    Rubble,
    Liquids,
    Corpses,
    Computers,
    Nested,
    Toilets,
    Gaspumps,
    Vehicles,
    Traps,
    Graffiti,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    pub character: char,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub cells: IndexMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,

    pub calculated_parameters: IndexMap<ParameterIdentifier, CDDAIdentifier>,
    pub parameters: IndexMap<ParameterIdentifier, Parameter>,

    pub mappings: HashMap<Mapping, HashMap<char, MapGenValue>>,

    pub palettes: Vec<MapGenValue>,

    #[serde(skip)]
    pub set: Vec<Arc<dyn Set>>,

    #[serde(skip)]
    pub place: HashMap<Mapping, Vec<Arc<dyn Place>>>,
}

impl Default for MapData {
    fn default() -> Self {
        let mut cells = IndexMap::new();

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
            palettes: Default::default(),
            mappings: Default::default(),
            set: vec![],
            place: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaceFurniture {
    furn: CDDAIdentifier,
    x: NumberOrRange<u32>,
    y: NumberOrRange<u32>,
}

impl Place for PlaceFurniture {
    fn coordinates(&self) -> UVec2 {
        UVec2::new(self.x.number(), self.y.number())
    }

    fn tile_layer(&self) -> TileLayer {
        TileLayer::Furniture
    }

    fn get_sprites(
        &self,
        coordinates: IVec3,
        adjacent_sprites: &AdjacentSprites,
        tilesheet: &TilesheetKind,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<SpriteType> {
        let sprite_kind = match tilesheet {
            TilesheetKind::Legacy(l) => l.get_sprite(&self.furn, json_data),
            TilesheetKind::Current(c) => c.get_sprite(&self.furn, json_data),
        };

        let (fg, bg) = get_sprite_type_from_sprite(
            &self.furn,
            coordinates,
            adjacent_sprites,
            TileLayer::Furniture,
            &sprite_kind,
            json_data,
        );

        let mut sprite_types = vec![];

        if let Some(fg) = fg {
            sprite_types.push(fg)
        }

        if let Some(bg) = bg {
            sprite_types.push(bg)
        }

        sprite_types
    }

    fn get_mapped_sprites(
        &self,
        chosen_coordinates: &UVec2,
        z: i32,
    ) -> HashMap<IVec3, MappedSprite> {
        let mut mapped_sprites = HashMap::new();

        let mut mapped_sprite = MappedSprite::default();
        mapped_sprite.furniture = Some(self.furn.clone());

        mapped_sprites.insert(
            IVec3::new(chosen_coordinates.x as i32, chosen_coordinates.y as i32, z),
            mapped_sprite,
        );

        mapped_sprites
    }
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
    pub x: NumberOrRange<u32>,
    pub y: NumberOrRange<u32>,
    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

impl Set for SetPoint {
    fn coordinates(&self) -> Vec<UVec2> {
        let mut coords = HashSet::new();

        for _ in self.repeat.0..self.repeat.1 {
            // Block here to release the lock on RANDOM since the number() function also uses RANDOM
            {
                let mut random = RANDOM.write().unwrap();

                if random.random_range(1..=self.chance) != 1 {
                    continue;
                }
            }

            let coordinates = UVec2::new(self.x.number(), self.y.number());
            coords.insert(coordinates);
        }

        Vec::from_iter(coords)
    }

    fn operation(&self) -> &SetOperation {
        &self.operation
    }
}

#[derive(Debug, Clone)]
pub struct SetLine {
    pub from_x: NumberOrRange<u32>,
    pub from_y: NumberOrRange<u32>,

    pub to_x: NumberOrRange<u32>,
    pub to_y: NumberOrRange<u32>,

    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

impl Set for SetLine {
    fn coordinates(&self) -> Vec<UVec2> {
        let mut coords = HashSet::new();

        for _ in self.repeat.0..self.repeat.1 {
            {
                let mut random = RANDOM.write().unwrap();

                if random.random_range(1..=self.chance) != 1 {
                    continue;
                }
            }

            let from_x = self.from_x.number();
            let from_y = self.from_y.number();
            let to_x = self.to_x.number();
            let to_y = self.to_y.number();

            let line = bresenham_line(from_x as i32, from_y as i32, to_x as i32, to_y as i32);

            for (x, y) in line {
                coords.insert(UVec2::new(x as u32, y as u32));
            }
        }

        Vec::from_iter(coords)
    }

    fn operation(&self) -> &SetOperation {
        &self.operation
    }
}

#[derive(Debug, Clone)]
pub struct SetSquare {
    pub top_left_x: NumberOrRange<u32>,
    pub top_left_y: NumberOrRange<u32>,

    pub bottom_right_x: NumberOrRange<u32>,
    pub bottom_right_y: NumberOrRange<u32>,

    pub z: i32,
    pub chance: u32,
    pub repeat: (u32, u32),
    pub operation: SetOperation,
}

impl Set for SetSquare {
    fn coordinates(&self) -> Vec<UVec2> {
        let mut coordinates = vec![];

        let top_left_chosen_y = self.top_left_y.number();
        let top_left_chosen_x = self.top_left_x.number();

        let bottom_right_chosen_y = self.bottom_right_y.number();
        let bottom_right_chosen_x = self.bottom_right_x.number();

        for y in top_left_chosen_y..bottom_right_chosen_y {
            for x in top_left_chosen_x..bottom_right_chosen_x {
                coordinates.push(UVec2::new(x, y))
            }
        }

        coordinates
    }

    fn operation(&self) -> &SetOperation {
        &self.operation
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
        cells: IndexMap<UVec2, Cell>,
        mappings: HashMap<Mapping, HashMap<char, MapGenValue>>,
        palettes: Vec<MapGenValue>,
        parameters: IndexMap<ParameterIdentifier, Parameter>,
        set: Vec<Arc<dyn Set>>,
        place: HashMap<Mapping, Vec<Arc<dyn Place>>>,
    ) -> Self {
        Self {
            calculated_parameters: Default::default(),
            fill,
            parameters,
            palettes,
            mappings,
            cells,
            set,
            place,
        }
    }

    pub fn calculate_parameters(&mut self, all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>) {
        let mut calculated_parameters = IndexMap::new();

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

    pub fn get_mapping(
        &self,
        mapping_kind: &Mapping,
        character: &char,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
    ) -> Option<CDDAIdentifier> {
        // If we find the mapping in the current map's terrain field, return that
        if let Some(id) = self
            .mappings
            .get(mapping_kind)
            .map(|v| v.get(character))
            .flatten()
        {
            return Some(id.get_identifier(&self.calculated_parameters));
        };

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(&self.calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) = palette.get_mapping(
                mapping_kind,
                character,
                &self.calculated_parameters,
                all_palettes,
            ) {
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
        let terrain = self.get_mapping(&Mapping::Terrain, character, all_palettes);
        let furniture = self.get_mapping(&Mapping::Furniture, character, all_palettes);

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
