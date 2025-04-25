pub(crate) mod handlers;
pub(crate) mod importing;

use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::item::{CDDDAItemGroup, EntryItem, ItemGroupSubtype};
use crate::cdda_data::map_data::{MapGenItem, MapGenMonster, MapGenMonsterType};
use crate::cdda_data::palettes::{CDDAPalette, Parameter};
use crate::cdda_data::{MapGenValue, NumberOrRange, TileLayer};
use crate::editor_data::Project;
use crate::map::handlers::{get_sprite_type_from_sprite, SpriteType};
use crate::tileset::legacy_tileset::MappedSprite;
use crate::tileset::{AdjacentSprites, Tilesheet, TilesheetKind};
use crate::util::{
    bresenham_line, CDDAIdentifier, DistributionInner, GetIdentifier, ParameterIdentifier,
};
use crate::RANDOM;
use dyn_clone::{clone_trait_object, DynClone};
use glam::{IVec3, UVec2};
use indexmap::IndexMap;
use rand::Rng;
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
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

// Things like items or whatever else will be represented in the sidebar panel
pub trait RepresentativeProperty: Debug + DynClone + Send + Sync {
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value;
}

clone_trait_object!(RepresentativeProperty);

// Things like terrain, furniture, monsters This allows us to get the Identifier
pub trait VisibleProperty: RepresentativeProperty {
    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<CDDAIdentifier>;
}

clone_trait_object!(VisibleProperty);

#[derive(Debug, Clone, Deserialize, Hash, PartialOrd, PartialEq, Eq, Ord)]
#[serde(rename_all = "snake_case")]
pub enum VisibleMapping {
    Terrain,
    Furniture,
    Traps,
    Monster,
}

#[derive(Debug, Clone, Deserialize, Hash, PartialOrd, PartialEq, Eq, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RepresentativeMapping {
    Monster,
    ItemGroups,
}

pub mod visible_properties {
    use super::*;
    use crate::util::MeabyVec;

    #[derive(Debug, Clone)]
    pub struct TerrainProperty {
        pub mapgen_value: MapGenValue,
    }

    impl RepresentativeProperty for TerrainProperty {
        fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
            todo!()
        }
    }

    impl VisibleProperty for TerrainProperty {
        fn get_identifier(
            &self,
            calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
            json_data: &DeserializedCDDAJsonData,
        ) -> Option<CDDAIdentifier> {
            Some(self.mapgen_value.get_identifier(calculated_parameters))
        }
    }

    #[derive(Debug, Clone)]
    pub struct MonsterProperty {
        pub monster: MeabyVec<MapGenMonster>,
    }

    impl RepresentativeProperty for MonsterProperty {
        fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
            todo!()
        }
    }

    impl VisibleProperty for MonsterProperty {
        fn get_identifier(
            &self,
            calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
            json_data: &DeserializedCDDAJsonData,
        ) -> Option<CDDAIdentifier> {
            for monster in self.monster.clone().into_vec() {
                match monster
                    .chance
                    .clone()
                    .unwrap_or(NumberOrRange::Number(1))
                    .is_random_hit(100)
                {
                    true => {
                        return match monster.id {
                            MapGenMonsterType::Monster { monster } => {
                                Some(monster.get_identifier(calculated_parameters))
                            }
                            MapGenMonsterType::MonsterGroup { group } => {
                                let mon_group = json_data.monstergroups.get(&group)?;
                                mon_group
                                    .get_random_monster(&json_data.monstergroups)
                                    .map(|id| id.get_identifier(calculated_parameters))
                            }
                        }
                    }
                    false => {}
                }
            }

            None
        }
    }

    #[derive(Debug, Clone)]
    pub struct FurnitureProperty {
        pub mapgen_value: MapGenValue,
    }

    impl RepresentativeProperty for FurnitureProperty {
        fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
            todo!()
        }
    }

    impl VisibleProperty for FurnitureProperty {
        fn get_identifier(
            &self,
            calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
            json_data: &DeserializedCDDAJsonData,
        ) -> Option<CDDAIdentifier> {
            Some(self.mapgen_value.get_identifier(calculated_parameters))
        }
    }
}

pub mod representative_properties {
    use super::*;

    #[derive(Debug, Serialize)]
    #[serde(tag = "type")]
    pub enum DisplayItemGroup {
        Single {
            item: CDDAIdentifier,
            probability: f32,
        },
        Collection {
            name: Option<String>,
            items: Vec<DisplayItemGroup>,
            probability: f32,
        },
        Distribution {
            name: Option<String>,
            items: Vec<DisplayItemGroup>,
            probability: f32,
        },
    }

    impl DisplayItemGroup {
        pub fn probability(&self) -> f32 {
            match self {
                DisplayItemGroup::Single { probability, .. } => probability.clone(),
                DisplayItemGroup::Collection { probability, .. } => probability.clone(),
                DisplayItemGroup::Distribution { probability, .. } => probability.clone(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ItemProperty {
        pub items: Vec<MapGenItem>,
    }

    impl ItemProperty {
        fn get_display_item_group_from_item_group(
            &self,
            item_group: &CDDDAItemGroup,
            json_data: &DeserializedCDDAJsonData,
            group_probability: f32,
        ) -> Vec<DisplayItemGroup> {
            let mut display_item_groups: Vec<DisplayItemGroup> = Vec::new();

            let weight_sum = item_group.entries.iter().fold(0, |acc, v| match v {
                EntryItem::Item(i) => acc + i.probability,
                EntryItem::Group(g) => acc + g.probability,
                EntryItem::Distribution { probability, .. } => acc + probability.unwrap_or(100),
                EntryItem::Collection { probability, .. } => acc + probability.unwrap_or(100),
            });

            for entry in item_group.entries.iter() {
                match entry {
                    EntryItem::Item(i) => {
                        let display_item = DisplayItemGroup::Single {
                            item: i.item.clone(),
                            probability: i.probability as f32 / weight_sum as f32
                                * group_probability,
                        };
                        display_item_groups.push(display_item);
                    }
                    EntryItem::Group(g) => {
                        let other_group = json_data
                            .item_groups
                            .get(&g.group)
                            .expect("Item Group to exist");
                        let probability =
                            g.probability as f32 / weight_sum as f32 * group_probability;
                        let display_item = self.get_display_item_group_from_item_group(
                            other_group,
                            json_data,
                            probability,
                        );

                        match other_group.subtype {
                            ItemGroupSubtype::Collection => {
                                display_item_groups.push(DisplayItemGroup::Collection {
                                    items: display_item,
                                    name: Some(other_group.id.clone().0),
                                    probability,
                                });
                            }
                            ItemGroupSubtype::Distribution => {
                                let probability = g.probability as f32 / weight_sum as f32;
                                display_item_groups.push(DisplayItemGroup::Distribution {
                                    items: display_item,
                                    name: Some(other_group.id.clone().0),
                                    probability,
                                });
                            }
                        }
                    }
                    EntryItem::Distribution {
                        distribution,
                        probability,
                    } => {}
                    EntryItem::Collection {
                        collection,
                        probability,
                    } => {}
                }
            }

            display_item_groups.sort_by(|v1, v2| {
                v2.probability()
                    .partial_cmp(&v1.probability())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            display_item_groups
        }
    }

    impl RepresentativeProperty for ItemProperty {
        fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
            let mut display_item_groups: Vec<DisplayItemGroup> = Vec::new();

            for mapgen_item in self.items.iter() {
                let item_group = json_data
                    .item_groups
                    .get(&mapgen_item.item)
                    .expect("Item group to exist");

                let probability = mapgen_item
                    .chance
                    .clone()
                    .map(|v| v.get_from_to().0)
                    .unwrap_or(100) as f32
                    // the default chance is 100, but we want to have a range from 0-1 so / 100
                    / 100.;

                let items =
                    self.get_display_item_group_from_item_group(item_group, json_data, probability);

                display_item_groups.push(DisplayItemGroup::Distribution {
                    name: Some(mapgen_item.item.clone().0),
                    probability,
                    items,
                });
            }

            display_item_groups.sort_by(|v1, v2| {
                v2.probability()
                    .partial_cmp(&v1.probability())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            serde_json::to_value(display_item_groups).unwrap()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Cell {
    pub character: char,
}

// The struct which holds the data that will be shown in the side panel in the ui
#[derive(Debug, Serialize)]
pub struct CellRepresentation {
    item_groups: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub cells: IndexMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,

    pub calculated_parameters: IndexMap<ParameterIdentifier, CDDAIdentifier>,
    pub parameters: IndexMap<ParameterIdentifier, Parameter>,
    pub palettes: Vec<MapGenValue>,

    #[serde(skip)]
    pub visible: HashMap<VisibleMapping, HashMap<char, Arc<dyn VisibleProperty>>>,

    #[serde(skip)]
    pub representative:
        HashMap<RepresentativeMapping, HashMap<char, Arc<dyn RepresentativeProperty>>>,

    #[serde(skip)]
    pub set: Vec<Arc<dyn Set>>,

    #[serde(skip)]
    pub place: HashMap<VisibleMapping, Vec<Arc<dyn Place>>>,
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
            visible: Default::default(),
            palettes: Default::default(),
            set: vec![],
            place: Default::default(),
            representative: Default::default(),
        }
    }
}

impl MapData {
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

    pub fn get_visible_mapping(
        &self,
        mapping_kind: &VisibleMapping,
        character: &char,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<CDDAIdentifier> {
        let mapping = self.visible.get(mapping_kind)?;

        if let Some(id) = mapping.get(character) {
            return id.get_identifier(&self.calculated_parameters, json_data);
        }

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(&self.calculated_parameters);
            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) = palette.get_visible_mapping(
                mapping_kind,
                character,
                &self.calculated_parameters,
                json_data,
            ) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_representative_mapping(
        &self,
        mapping_kind: &RepresentativeMapping,
        character: &char,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Value> {
        let mapping = self.representative.get(mapping_kind)?;

        match mapping.get(character) {
            None => {}
            Some(s) => return Some(s.representation(json_data)),
        }

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(&self.calculated_parameters);
            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) = palette.get_representative_mapping(
                mapping_kind,
                character,
                &self.calculated_parameters,
                json_data,
            ) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_cell_data(
        &self,
        character: &char,
        json_data: &DeserializedCDDAJsonData,
    ) -> CellRepresentation {
        let item_groups = self
            .get_representative_mapping(&RepresentativeMapping::ItemGroups, character, json_data)
            .unwrap_or(Value::Array(vec![]));

        CellRepresentation { item_groups }
    }

    pub fn get_visible_mappings(
        &self,
        character: &char,
        json_data: &DeserializedCDDAJsonData,
    ) -> CDDAIdentifierGroup {
        let terrain = self.get_visible_mapping(&VisibleMapping::Terrain, character, json_data);
        let furniture = self.get_visible_mapping(&VisibleMapping::Furniture, character, json_data);
        let monster = self.get_visible_mapping(&VisibleMapping::Monster, character, json_data);

        CDDAIdentifierGroup {
            terrain,
            furniture,
            monster,
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

        state.end()
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
        UVec2::new(self.x.rand_number(), self.y.rand_number())
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

            let coordinates = UVec2::new(self.x.rand_number(), self.y.rand_number());
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

            let from_x = self.from_x.rand_number();
            let from_y = self.from_y.rand_number();
            let to_x = self.to_x.rand_number();
            let to_y = self.to_y.rand_number();

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

        let top_left_chosen_y = self.top_left_y.rand_number();
        let top_left_chosen_x = self.top_left_x.rand_number();

        let bottom_right_chosen_y = self.bottom_right_y.rand_number();
        let bottom_right_chosen_x = self.bottom_right_x.rand_number();

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
    pub monster: Option<CDDAIdentifier>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProjectContainer {
    pub data: Vec<Project>,
    pub current_project: Option<usize>,
}
