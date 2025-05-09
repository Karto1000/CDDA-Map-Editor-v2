pub(crate) mod handlers;
pub(crate) mod importing;
pub(crate) mod map_properties;
pub(crate) mod place;

use crate::cdda_data::io::{DeserializedCDDAJsonData, NULL_FURNITURE, NULL_TERRAIN};
use crate::cdda_data::map_data::{
    MapGenMonster, MapGenMonsterType, NeighborDirection, OmTerrainMatch, OmTerrainMatchType,
    PlaceOuter,
};
use crate::cdda_data::palettes::{CDDAPalette, Parameter};
use crate::cdda_data::region_settings::CDDARegionSettings;
use crate::cdda_data::{MapGenValue, NumberOrRange, TileLayer};
use crate::editor_data::{Project, ZLevel};
use crate::map::handlers::{get_sprite_type_from_sprite, SpriteType};
use crate::tileset::legacy_tileset::MappedCDDAIds;
use crate::tileset::{AdjacentSprites, Tilesheet, TilesheetKind};
use crate::util::{
    bresenham_line, CDDAIdentifier, DistributionInner, GetIdentifier, ParameterIdentifier, Weighted,
};
use downcast_rs::{impl_downcast, Downcast, DowncastSend, DowncastSync};
use dyn_clone::{clone_trait_object, DynClone};
use glam::{IVec2, IVec3, UVec2};
use indexmap::IndexMap;
use log::{error, warn};
use rand::{rng, Rng};
use serde::ser::{SerializeMap, SerializeStruct};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::Arc;
use strum_macros::EnumString;

pub const SPECIAL_EMPTY_CHAR: char = ' ';
pub const DEFAULT_MAP_DATA_SIZE: UVec2 = UVec2::new(24, 24);

pub trait Set: Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend {
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

    fn get_mapped_sprites(&self, chosen_coordinates: Vec<IVec3>) -> HashMap<IVec3, MappedCDDAIds> {
        let mut new_mapped_sprites = HashMap::new();

        for coordinates in chosen_coordinates {
            match self.operation() {
                SetOperation::Place { ty, id } => {
                    let mut mapped_sprite = MappedCDDAIds::default();

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
impl_downcast!(sync Set);

pub trait Place: Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        None
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

clone_trait_object!(Place);
impl_downcast!(sync Place);

// Things like items or whatever else will be represented in the sidebar panel
pub trait RepresentativeProperty:
    Debug + DynClone + Send + Sync + Downcast + DowncastSync + DowncastSend
{
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value;
}

clone_trait_object!(RepresentativeProperty);
impl_downcast!(sync RepresentativeProperty);

// Things like terrain, furniture, monsters This allows us to get the Identifier
pub trait VisibleProperty: RepresentativeProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>>;
}

clone_trait_object!(VisibleProperty);
impl_downcast!(sync VisibleProperty);

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialOrd, PartialEq, Eq, Ord)]
#[serde(rename_all = "snake_case")]
pub enum VisibleMappingKind {
    Terrain = 0,
    Furniture = 1,
    Traps = 2,
    Monster = 3,
    Nested = 4,
    Field = 5,
}

#[derive(Debug, Clone, Deserialize, Hash, PartialOrd, PartialEq, Eq, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RepresentativeMappingKind {
    Monster,
    ItemGroups,
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

#[derive(Debug, Eq, PartialEq, Serialize)]
pub enum VisibleMappingCommandKind {
    Place,
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct VisibleMappingCommand {
    id: CDDAIdentifier,
    mapping: VisibleMappingKind,
    coordinates: IVec2,
    kind: VisibleMappingCommandKind,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapDataFlag {
    EraseAllBeforePlacingTerrain,
    AllowTerrainUnderOtherData,

    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapGenNested {
    pub neighbors: Option<HashMap<NeighborDirection, Vec<OmTerrainMatch>>>,
    pub joins: Option<HashMap<NeighborDirection, Vec<OmTerrainMatch>>>,

    pub chunks: Vec<Weighted<MapGenValue>>,

    #[serde(default)]
    // This is basically just any "else_chunks"
    pub invert_condition: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapDataConfig {
    pub simulated_neighbors: HashMap<NeighborDirection, Vec<CDDAIdentifier>>,
}

impl Default for MapDataConfig {
    fn default() -> Self {
        let mut simulated_neighbors = HashMap::new();
        simulated_neighbors.insert(NeighborDirection::Above, vec![]);
        simulated_neighbors.insert(NeighborDirection::Below, vec![]);
        simulated_neighbors.insert(NeighborDirection::East, vec![]);
        simulated_neighbors.insert(NeighborDirection::West, vec![]);
        simulated_neighbors.insert(NeighborDirection::North, vec![]);
        simulated_neighbors.insert(NeighborDirection::South, vec![]);
        simulated_neighbors.insert(NeighborDirection::NorthEast, vec![]);
        simulated_neighbors.insert(NeighborDirection::NorthWest, vec![]);
        simulated_neighbors.insert(NeighborDirection::SouthEast, vec![]);
        simulated_neighbors.insert(NeighborDirection::SouthWest, vec![]);

        MapDataConfig {
            simulated_neighbors,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub cells: IndexMap<UVec2, Cell>,
    pub fill: Option<DistributionInner>,
    pub map_size: UVec2,

    pub config: MapDataConfig,

    pub calculated_parameters: IndexMap<ParameterIdentifier, CDDAIdentifier>,
    pub parameters: IndexMap<ParameterIdentifier, Parameter>,
    pub palettes: Vec<MapGenValue>,
    pub flags: HashSet<MapDataFlag>,

    #[serde(skip)]
    pub visible: HashMap<VisibleMappingKind, HashMap<char, Arc<dyn VisibleProperty>>>,

    #[serde(skip)]
    pub representative:
        HashMap<RepresentativeMappingKind, HashMap<char, Arc<dyn RepresentativeProperty>>>,

    #[serde(skip)]
    pub set: Vec<Arc<dyn Set>>,

    #[serde(skip)]
    pub place: HashMap<VisibleMappingKind, Vec<PlaceOuter<Arc<dyn Place>>>>,
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
            map_size: DEFAULT_MAP_DATA_SIZE,
            config: Default::default(),
            calculated_parameters: Default::default(),
            parameters: Default::default(),
            visible: Default::default(),
            palettes: Default::default(),
            set: vec![],
            place: Default::default(),
            representative: Default::default(),
            flags: Default::default(),
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

    pub fn get_mapped_cdda_ids(
        &self,
        json_data: &DeserializedCDDAJsonData,
        z: ZLevel,
    ) -> HashMap<IVec3, MappedCDDAIds> {
        let mut local_mapped_cdda_ids = HashMap::new();

        let region_settings = json_data
            .region_settings
            .get(&CDDAIdentifier("default".into()))
            .expect("Region settings to exist");

        let fill_terrain_sprite = match &self.fill {
            None => None,
            Some(id) => {
                let id = id.get_identifier(&self.calculated_parameters);

                Some(id.as_final_id(&region_settings, &json_data.terrain, &json_data.furniture))
            }
        };

        self.cells.iter().for_each(|(p, _)| {
            let mut mapped_ids = MappedCDDAIds::default();
            mapped_ids.terrain = fill_terrain_sprite.clone();

            local_mapped_cdda_ids.insert(IVec3::new(p.x as i32, p.y as i32, z), mapped_ids);
        });

        let all_commands = self.get_commands(&json_data);

        for command in all_commands {
            let command_3d_coords = IVec3::new(command.coordinates.x, command.coordinates.y, z);

            match command.kind {
                VisibleMappingCommandKind::Place => {
                    let id = command.id.as_final_id(
                        region_settings,
                        &json_data.terrain,
                        &json_data.furniture,
                    );

                    let ident_mut = match local_mapped_cdda_ids.get_mut(&command_3d_coords) {
                        None => {
                            local_mapped_cdda_ids
                                .insert(command_3d_coords.clone(), MappedCDDAIds::default());
                            local_mapped_cdda_ids.get_mut(&command_3d_coords).unwrap()
                        }
                        Some(i) => i,
                    };

                    match command.mapping {
                        VisibleMappingKind::Terrain => {
                            ident_mut.terrain = Some(id.clone());
                        }
                        VisibleMappingKind::Furniture => {
                            ident_mut.furniture = Some(id.clone());
                        }
                        VisibleMappingKind::Traps => {
                            ident_mut.trap = Some(id.clone());
                        }
                        VisibleMappingKind::Monster => ident_mut.monster = Some(id.clone()),
                        VisibleMappingKind::Field => ident_mut.field = Some(id.clone()),
                        VisibleMappingKind::Nested => unreachable!(),
                    }
                }
            }
        }

        for set in self.set.iter() {
            let chosen_coordinates: Vec<IVec3> = set
                .coordinates()
                .into_iter()
                .map(|c| IVec3::new(c.x as i32, c.y as i32, z))
                .collect();

            set.get_mapped_sprites(chosen_coordinates.clone())
                .into_iter()
                .for_each(
                    |(coords, ids)| match local_mapped_cdda_ids.get_mut(&coords) {
                        None => {
                            warn!("Coordinates {:?} for set are out of bounds", coords);
                        }
                        Some(existing_mapped) => {
                            existing_mapped.update_override(ids);
                        }
                    },
                );
        }

        local_mapped_cdda_ids
    }

    pub fn get_commands(&self, json_data: &DeserializedCDDAJsonData) -> Vec<VisibleMappingCommand> {
        // We need to store all commands in this list here so we can sort it and act them out in
        // the order the VisibleMappingCommandKind enum has
        let mut all_commands: Vec<VisibleMappingCommand> = vec![];

        // We need to insert the mapped_sprite before we get the fg and bg of this sprite since
        // the function relies on the mapped sprite of this sprite to already exist
        self.cells.iter().for_each(|(p, cell)| {
            let ident_commands =
                self.get_identifier_change_commands(&cell.character, &p.as_ivec2(), &json_data);

            all_commands.extend(ident_commands)
        });

        for (_, place_vec) in self.place.iter() {
            for place in place_vec {
                let upper_bound = place.repeat.rand_number();

                for _ in 0..upper_bound {
                    let position = place.coordinates();

                    // We only want to place one in place.chance times
                    let rand_chance_num = rng().random_range(0..=100);
                    if rand_chance_num > place.chance {
                        continue;
                    }

                    match place.inner.get_commands(&position, self, json_data) {
                        None => {}
                        Some(commands) => {
                            all_commands.extend(commands);
                        }
                    }
                }
            }
        }

        all_commands.sort_by(|a, b| a.mapping.cmp(&b.mapping));
        all_commands
    }

    pub fn get_visible_mapping(
        &self,
        mapping_kind: &VisibleMappingKind,
        character: &char,
        position: &IVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let mapping = self.visible.get(mapping_kind)?;

        if let Some(id) = mapping.get(character) {
            return id.get_commands(position, self, json_data);
        }

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(&self.calculated_parameters);
            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) =
                palette.get_visible_mapping(mapping_kind, character, position, self, json_data)
            {
                return Some(id);
            }
        }

        None
    }

    pub fn get_representative_mapping(
        &self,
        mapping_kind: &RepresentativeMappingKind,
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
            .get_representative_mapping(
                &RepresentativeMappingKind::ItemGroups,
                character,
                json_data,
            )
            .unwrap_or(Value::Array(vec![]));

        CellRepresentation { item_groups }
    }

    pub fn get_identifier_change_commands(
        &self,
        character: &char,
        position: &IVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<VisibleMappingCommand> {
        let mut commands = Vec::new();

        let nested_commands = self
            .get_visible_mapping(&VisibleMappingKind::Nested, character, position, json_data)
            .unwrap_or_default();

        let terrain_commands = self
            .get_visible_mapping(&VisibleMappingKind::Terrain, character, position, json_data)
            .unwrap_or_default();

        let furniture_commands = self
            .get_visible_mapping(
                &VisibleMappingKind::Furniture,
                character,
                position,
                json_data,
            )
            .unwrap_or_default();

        let monster_commands = self
            .get_visible_mapping(&VisibleMappingKind::Monster, character, position, json_data)
            .unwrap_or_default();

        commands.extend(terrain_commands);
        commands.extend(furniture_commands);
        commands.extend(monster_commands);
        commands.extend(nested_commands);

        commands
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
    ItemRemove,
    FieldRemove,
    TrapRemove,
    CreatureRemove,
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
        amount: NumberOrRange<u32>,
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
                let mut rng = rng();
                //let mut random = RANDOM.write().unwrap();

                if rng.random_range(1..=self.chance) != 1 {
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
                let mut rng = rng();
                //let mut random = RANDOM.write().unwrap();

                if rng.random_range(1..=self.chance) != 1 {
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct ProjectContainer {
    pub data: Vec<Project>,
    pub current_project: Option<usize>,
}

#[cfg(test)]
mod tests {
    use crate::cdda_data::{CDDADistributionInner, Distribution, MapGenValue, Switch};
    use crate::map::importing::MapDataImporter;
    use crate::map::map_properties::visible::TerrainProperty;
    use crate::map::VisibleMappingKind;
    use crate::util::{
        CDDAIdentifier, DistributionInner, Load, MeabyVec, MeabyWeighted, ParameterIdentifier,
        Weighted,
    };
    use crate::TEST_CDDA_DATA;
    use glam::UVec2;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tokio;

    const TEST_DATA_PATH: &str = "test_data";

    #[tokio::test]
    async fn test_fill_ter() {
        let mut map_loader = MapDataImporter {
            path: PathBuf::from(TEST_DATA_PATH).join("test_fill_ter.json"),
            om_terrain: "test_fill_ter".into(),
        };

        let project_data = map_loader.load().await.unwrap();
        let map_data = project_data.maps.get(&0).unwrap();

        for (coords, cell) in map_data.cells.iter() {
            assert_eq!(cell.character, ' ');
            assert!(coords.x < 24 && coords.y < 24);
        }

        assert_eq!(project_data.maps.len(), 1);
        assert_eq!(project_data.size, UVec2::new(24, 24));
        assert_eq!(project_data.name, "test_fill_ter");
        assert_eq!(
            map_data.fill,
            Some(DistributionInner::Normal("t_grass".into()))
        );
    }

    #[tokio::test]
    async fn test_parameters() {
        let cdda_data = TEST_CDDA_DATA.get().await;

        let mut map_loader = MapDataImporter {
            path: PathBuf::from(TEST_DATA_PATH).join("test_terrain.json"),
            om_terrain: "test_terrain".into(),
        };

        let mut project = map_loader.load().await.unwrap();
        let map_data = project.maps.get_mut(&0).unwrap();
        map_data.calculate_parameters(&cdda_data.palettes);

        let parameter_identifier = ParameterIdentifier("terrain_type".to_string());
        let parameter = map_data.parameters.get(&parameter_identifier).unwrap();

        let weighted_grass = Weighted::new("t_grass", 10);
        let weighted_grass_dead = Weighted::new("t_grass_dead", 1);

        let expected_distribution = Distribution {
            distribution: MeabyVec::Vec(vec![
                MeabyWeighted::Weighted(weighted_grass),
                MeabyWeighted::Weighted(weighted_grass_dead),
            ]),
        };

        assert_eq!(parameter.default, expected_distribution);

        let calculated_parameter = map_data
            .calculated_parameters
            .get(&parameter_identifier)
            .unwrap();

        assert!(
            calculated_parameter.0 == "t_grass".to_string()
                || calculated_parameter.0 == "t_grass_dead".to_string()
        )
    }

    #[tokio::test]
    async fn test_terrain() {
        const SINGLE_CHAR: char = '.';
        const NOT_WEIGHTED_DISTRIBUTION_CHAR: char = '1';
        const WEIGHTED_DISTRIBUTION_CHAR: char = '2';
        const WEIGHTED_DISTRIBUTION_WITH_KEYWORD_CHAR: char = '3';
        const PARAMETER_CHAR: char = '4';
        const SWITCH_CHAR: char = '5';

        let cdda_data = TEST_CDDA_DATA.get().await;

        let mut map_loader = MapDataImporter {
            path: PathBuf::from(TEST_DATA_PATH).join("test_terrain.json"),
            om_terrain: "test_terrain".into(),
        };

        let mut project = map_loader.load().await.unwrap();
        let map_data = project.maps.get_mut(&0).unwrap();
        map_data.calculate_parameters(&cdda_data.palettes);

        // Test the terrain mapped to a single sprite
        {
            let single_terrain = map_data.cells.get(&UVec2::new(0, 1)).unwrap();
            assert_eq!(single_terrain.character, SINGLE_CHAR);

            let terrain_property = map_data
                .visible
                .get(&VisibleMappingKind::Terrain)
                .unwrap()
                .get(&SINGLE_CHAR)
                .unwrap()
                .clone();

            let terrain_property = terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::String("t_grass".into())
            )
        }

        // Test the distribution that is not weighted
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(0, 0)).unwrap();
            assert_eq!(distr_terrain.character, NOT_WEIGHTED_DISTRIBUTION_CHAR);

            let terrain_property = map_data
                .visible
                .get(&VisibleMappingKind::Terrain)
                .unwrap()
                .get(&NOT_WEIGHTED_DISTRIBUTION_CHAR)
                .unwrap()
                .clone();

            let terrain_property = terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let expected_distribution = vec![
                MeabyWeighted::NotWeighted("t_grass".into()),
                MeabyWeighted::NotWeighted("t_grass_dead".into()),
            ];

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::Distribution(MeabyVec::Vec(expected_distribution))
            );
        }

        // Test the distribution that is weighted
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(1, 0)).unwrap();
            assert_eq!(distr_terrain.character, WEIGHTED_DISTRIBUTION_CHAR);

            let terrain_property = map_data
                .visible
                .get(&VisibleMappingKind::Terrain)
                .unwrap()
                .get(&WEIGHTED_DISTRIBUTION_CHAR)
                .unwrap()
                .clone();

            let terrain_property = terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let weighted_grass = Weighted::new("t_grass", 10);
            let weighted_grass_dead = Weighted::new("t_grass_dead", 1);

            let expected_distribution = vec![
                MeabyWeighted::Weighted(weighted_grass),
                MeabyWeighted::Weighted(weighted_grass_dead),
            ];

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::Distribution(MeabyVec::Vec(expected_distribution))
            );
        }

        // Test the weighted distribution with the "distribution" keyword
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(2, 0)).unwrap();
            assert_eq!(
                distr_terrain.character,
                WEIGHTED_DISTRIBUTION_WITH_KEYWORD_CHAR
            );

            let terrain_property = map_data
                .visible
                .get(&VisibleMappingKind::Terrain)
                .unwrap()
                .get(&WEIGHTED_DISTRIBUTION_WITH_KEYWORD_CHAR)
                .unwrap()
                .clone();

            let terrain_property = terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let weighted_grass = Weighted::new("t_grass", 1);
            let weighted_grass_dead = Weighted::new("t_grass_dead", 10);

            let expected_distribution = Distribution {
                distribution: MeabyVec::Vec(vec![
                    MeabyWeighted::Weighted(weighted_grass),
                    MeabyWeighted::Weighted(weighted_grass_dead),
                ]),
            };

            assert_eq!(
                terrain_property.mapgen_value,
                MapGenValue::Distribution(MeabyVec::Single(MeabyWeighted::NotWeighted(
                    CDDADistributionInner::Distribution(expected_distribution)
                )))
            );
        }

        // Test if a set parameter works
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(3, 0)).unwrap();
            assert_eq!(distr_terrain.character, PARAMETER_CHAR);

            let terrain_property = map_data
                .visible
                .get(&VisibleMappingKind::Terrain)
                .unwrap()
                .get(&PARAMETER_CHAR)
                .unwrap()
                .clone();

            let terrain_property = terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let to_eq = MapGenValue::Param {
                param: ParameterIdentifier("terrain_type".to_string()),
                fallback: Some("t_grass".into()),
            };

            assert_eq!(terrain_property.mapgen_value, to_eq);
        }

        // Test if a switch works
        {
            let distr_terrain = map_data.cells.get(&UVec2::new(4, 0)).unwrap();
            assert_eq!(distr_terrain.character, SWITCH_CHAR);

            let terrain_property = map_data
                .visible
                .get(&VisibleMappingKind::Terrain)
                .unwrap()
                .get(&SWITCH_CHAR)
                .unwrap()
                .clone();

            let terrain_property = terrain_property.downcast_arc::<TerrainProperty>().unwrap();

            let mut to_eq_cases: HashMap<CDDAIdentifier, CDDAIdentifier> = HashMap::new();
            to_eq_cases.insert("t_grass".into(), "t_concrete_railing".into());
            to_eq_cases.insert("t_grass_dead".into(), "t_concrete_wall".into());

            let to_eq = MapGenValue::Switch {
                switch: Switch {
                    param: ParameterIdentifier("terrain_type".into()),
                    fallback: "t_grass".into(),
                },
                cases: to_eq_cases,
            };

            assert_eq!(terrain_property.mapgen_value, to_eq);
        }
    }
}
