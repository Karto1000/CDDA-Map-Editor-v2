use crate::cdda_data::item::{ItemEntry, ItemGroupSubtype};
use crate::cdda_data::map_data::{
    MapGenGaspumpFuelType, ReferenceOrInPlace, VehicleStatus,
};
use crate::cdda_data::vehicle_parts::{CDDAVehiclePart, Location};
use crate::cdda_data::vehicles::{CDDAVehicle, VehiclePart};
use crate::map::map_properties::{
    ComputersProperty, CorpsesProperty, FieldsProperty, FurnitureProperty,
    GaspumpsProperty, ItemsProperty, MonstersProperty, NestedProperty,
    SignsProperty, TerrainProperty, ToiletsProperty, TrapsProperty,
    VehiclesProperty,
};
use crate::map::*;
use crate::tileset::GetRandom;
use cdda_lib::{NULL_FIELD, NULL_NESTED, NULL_TRAP};
use log::error;
use num_traits::real::Real;
use rand::prelude::IndexedRandom;
use rand::random_range;
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

impl Property for TerrainProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let ident = self
            .mapgen_value
            .get_identifier(&map_data.calculated_parameters)
            .ok()?;

        if ident == CDDAIdentifier::from(NULL_TERRAIN) {
            return None;
        }

        let command = SetTile::terrain(
            TilesheetCDDAId::simple(ident),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );

        Some(vec![command])
    }
}

impl Property for MonstersProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let monster = self.monster.get_random();

        let ident = match monster
            .chance
            .clone()
            .unwrap_or(NumberOrRange::Number(1))
            .is_random_hit(100)
        {
            true => match &monster.id {
                MapGenMonsterType::Monster { monster } => {
                    monster.get_identifier(&map_data.calculated_parameters).ok()
                },
                MapGenMonsterType::MonsterGroup { group } => {
                    let id = group
                        .get_identifier(&map_data.calculated_parameters)
                        .ok()?;
                    let mon_group = json_data.monster_groups.get(&id)?;

                    let rand_monster = mon_group
                        .get_random_monster(
                            &json_data.monster_groups,
                            &map_data.calculated_parameters,
                        )
                        .ok();

                    rand_monster?
                        .get_identifier(&map_data.calculated_parameters)
                        .ok()
                },
            },
            false => None,
        };

        match ident {
            None => {},
            Some(ident) => {
                let command = SetTile::monster(
                    TilesheetCDDAId::simple(ident),
                    position.clone(),
                    Rotation::Deg0,
                    TileState::Normal,
                );

                return Some(vec![command]);
            },
        }

        None
    }
}

impl Property for FurnitureProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let ident = self
            .mapgen_value
            .get_identifier(&map_data.calculated_parameters)
            .ok()?;

        if ident == CDDAIdentifier::from(NULL_FURNITURE) {
            return None;
        }

        let command = SetTile::furniture(
            TilesheetCDDAId::simple(ident),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );

        Some(vec![command])
    }
}

#[derive(Debug, Clone, Serialize)]
struct SignRepresentation {
    pub signage: String,
    pub snipped: String,
}

impl Property for SignsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let command = SetTile::furniture(
            TilesheetCDDAId::simple("f_sign"),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );
        Some(vec![command])
    }
}

impl Property for NestedProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let rng = rng();
        let nested_chunk = self.nested.get_random();

        let should_place = match &nested_chunk.neighbors {
            None => true,
            Some(neighbors) => {
                neighbors.iter().all(|(dir, om_terrain_match)| {
                    let simulated_neighbor = map_data
                        .config
                        .simulated_neighbors
                        .get(dir)
                        .expect("Simulated neighbor must always exist");

                    om_terrain_match.iter().all(|om_terrain| {
                        if simulated_neighbor.is_empty() {
                            return false;
                        }

                        simulated_neighbor
                            .iter()
                            .all(|id| om_terrain.matches_identifier(id))
                    })
                })
            },
        };

        if nested_chunk.invert_condition {
            if should_place {
                return None;
            }
        } else if !should_place {
            return None;
        }

        let selected_chunk = nested_chunk
            .chunks
            .get_random()
            .get_identifier(&map_data.calculated_parameters)
            .ok()?;

        if selected_chunk == CDDAIdentifier::from(NULL_NESTED) {
            return None;
        }

        let nested_mapgen = match json_data.map_data.get(&selected_chunk) {
            None => {
                error!("Nested Mapgen {} not found", selected_chunk);
                return None;
            },
            Some(v) => v,
        };

        let mut commands = nested_mapgen.get_commands(json_data);

        commands.iter_mut().for_each(|c| {
            c.coordinates.x += position.x;
            c.coordinates.y = position.y + c.coordinates.y;
        });

        Some(commands)
    }
}

impl Property for FieldsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let field = self.field.get_random();

        if field.field == CDDAIdentifier::from(NULL_FIELD) {
            return None;
        }

        let command = SetTile::field(
            TilesheetCDDAId::simple(field.field.clone()),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );
        Some(vec![command])
    }
}

impl Property for GaspumpsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let gaspump = self.gaspumps.get_random();

        let id = match &gaspump.fuel {
            None => "t_gas_pump",
            Some(fuel) => match fuel {
                MapGenGaspumpFuelType::Gasoline
                | MapGenGaspumpFuelType::Avgas => "t_gas_pump",
                MapGenGaspumpFuelType::Diesel => "t_diesel_pump",
                MapGenGaspumpFuelType::Jp8 => "t_jp8_pump",
            },
        };

        let command = SetTile::furniture(
            TilesheetCDDAId::simple(id),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );
        Some(vec![command])
    }
}

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
            DisplayItemGroup::Collection { probability, .. } => {
                probability.clone()
            },
            DisplayItemGroup::Distribution { probability, .. } => {
                probability.clone()
            },
        }
    }
}

impl ItemsProperty {
    fn get_display_items_from_entries(
        &self,
        entries: &Vec<ItemEntry>,
        json_data: &DeserializedCDDAJsonData,
        group_probability: f32,
    ) -> Vec<DisplayItemGroup> {
        let mut display_item_groups: Vec<DisplayItemGroup> = Vec::new();

        let weight_sum = entries.iter().fold(0, |acc, v| match v {
            ItemEntry::Item(i) => acc + i.probability,
            ItemEntry::Group(g) => acc + g.probability,
            ItemEntry::Distribution { probability, .. } => {
                acc + probability.unwrap_or(100)
            },
            ItemEntry::Collection { probability, .. } => {
                acc + probability.unwrap_or(100)
            },
        });

        for entry in entries.iter() {
            match entry {
                ItemEntry::Item(i) => {
                    let display_item = DisplayItemGroup::Single {
                        item: i.item.clone(),
                        probability: i.probability as f32 / weight_sum as f32
                            * group_probability,
                    };
                    display_item_groups.push(display_item);
                },
                ItemEntry::Group(g) => {
                    let other_group =
                        &json_data.item_groups.get(&g.group).expect(
                            format!("Item Group {} to exist", &g.group)
                                .as_str(),
                        );

                    let probability = g.probability as f32 / weight_sum as f32
                        * group_probability;

                    let display_items = self.get_display_items_from_entries(
                        &other_group.common.entries,
                        json_data,
                        probability,
                    );

                    match other_group.common.subtype {
                        ItemGroupSubtype::Collection => {
                            display_item_groups.push(
                                DisplayItemGroup::Collection {
                                    items: display_items,
                                    name: Some(other_group.id.clone().0),
                                    probability,
                                },
                            );
                        },
                        ItemGroupSubtype::Distribution => {
                            display_item_groups.push(
                                DisplayItemGroup::Distribution {
                                    items: display_items,
                                    name: Some(other_group.id.clone().0),
                                    probability,
                                },
                            );
                        },
                    }
                },
                ItemEntry::Distribution {
                    distribution,
                    probability,
                } => {
                    let probability = probability
                        .map(|p| {
                            p as f32 / weight_sum as f32 * group_probability
                        })
                        .unwrap_or(group_probability / weight_sum as f32);

                    let display_items = self.get_display_items_from_entries(
                        distribution,
                        json_data,
                        probability,
                    );

                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some("In-Place".to_string()),
                        items: display_items,
                        probability,
                    });
                },
                ItemEntry::Collection {
                    collection,
                    probability,
                } => {
                    let probability = probability
                        .map(|p| {
                            p as f32 / weight_sum as f32 * group_probability
                        })
                        .unwrap_or(group_probability / weight_sum as f32);

                    let display_items = self.get_display_items_from_entries(
                        collection,
                        json_data,
                        probability,
                    );

                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some("In-Place".to_string()),
                        items: display_items,
                        probability,
                    });
                },
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

impl Property for ItemsProperty {}

impl Property for ComputersProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let command = SetTile::furniture(
            TilesheetCDDAId::simple("f_console"),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );

        Some(vec![command])
    }
}

impl Property for ToiletsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let command = SetTile::furniture(
            TilesheetCDDAId::simple("f_toilet"),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );

        Some(vec![command])
    }
}

impl Property for TrapsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let trap = self.trap.get_random();
        let ident =
            trap.get_identifier(&map_data.calculated_parameters).ok()?;

        if ident == CDDAIdentifier::from(NULL_TRAP) {
            return None;
        }

        let command = SetTile::furniture(
            TilesheetCDDAId::simple(ident),
            position.clone(),
            Rotation::Deg0,
            TileState::Normal,
        );

        Some(vec![command])
    }
}

#[derive(Debug, Clone)]
struct VehiclePartSpriteVariant {
    pub variant: String,
}

impl VehiclePartSpriteVariant {
    pub fn new(variant: impl Into<String>) -> Self {
        Self {
            variant: variant.into(),
        }
    }
}

impl Display for VehiclePartSpriteVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.variant.as_str())
    }
}

impl Property for VehiclesProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let mapgen_vehicle = self.vehicles.get_random();

        let vehicle = match json_data.vehicles.get(&mapgen_vehicle.vehicle) {
            None => {
                warn!("Vehicle {} not found", mapgen_vehicle.vehicle);
                return None;
            },
            Some(v) => v,
        };

        let mut commands = Vec::new();

        let mut highest_priority_parts: HashMap<
            IVec2,
            (&CDDAVehiclePart, Option<VehiclePartSpriteVariant>, usize),
        > = HashMap::new();

        let random_rotation = mapgen_vehicle
            .rotation
            .clone()
            .into_vec()
            .choose(&mut rng())
            .map(Clone::clone)
            .unwrap_or(0);

        let rotation_radians = (random_rotation as f32).to_radians();

        for part in vehicle.parts.iter() {
            // Positive y -> right
            // Negative y -> left
            // Positive x -> up
            // Negative x -> down
            let base_position = IVec2::new(part.x, part.y);

            // TODO: This rotation looks pretty munted for any rotation other than 0, 90, 180 and 270 degrees
            let rotated_x = (base_position.x as f32 * rotation_radians.cos()
                - base_position.y as f32 * rotation_radians.sin())
            .round() as i32;
            let rotated_y = (base_position.x as f32 * rotation_radians.sin()
                + base_position.y as f32 * rotation_radians.cos())
            .round() as i32;

            let part_position = IVec2::new(rotated_x, rotated_y);

            for vp in part.parts.iter() {
                let ident = match vp {
                    VehiclePart::Inline(id) => id.clone(),
                    VehiclePart::Object { part, .. } => part.clone(),
                };

                let (raw_ident, ty) = match ident.0.split_once('#') {
                    None => (ident.clone(), None),
                    Some((ident, ty)) => (
                        ident.into(),
                        Some(VehiclePartSpriteVariant::new(ty.to_string())),
                    ),
                };

                let vp_entry = match json_data.vehicle_parts.get(&raw_ident) {
                    None => {
                        warn!(
                            "Vehicle Part {} does not exist in cdda data",
                            raw_ident
                        );
                        continue;
                    },
                    Some(vp) => vp,
                };

                let location: Location = Location::from_str(
                    &vp_entry
                        .location
                        .clone()
                        .unwrap_or("structure".to_string()),
                )
                .unwrap_or(Location::Structure);

                let highest_priority_part =
                    match highest_priority_parts.get(&part_position) {
                        None => {
                            highest_priority_parts.insert(
                                part_position,
                                (vp_entry, ty.clone(), location.priority()),
                            );

                            highest_priority_parts.get(&part_position).unwrap()
                        },
                        Some(p) => p,
                    };

                if location.priority() > highest_priority_part.2 {
                    highest_priority_parts.insert(
                        part_position,
                        (vp_entry, ty, location.priority()),
                    );
                }
            }
        }

        // Generate visible mapping commands
        for (pos, (part, ty, _)) in highest_priority_parts {
            let rotation = match random_rotation % 360 {
                0..90 => Rotation::Deg270,
                180..270 => Rotation::Deg90,
                // TODO: dirty hack to make the rotation work "counter clockwise"
                n => Rotation::from(n + 90),
            };

            // TODO: Not that accurate to what it will look like in game since the status can also
            // remove tiles and do other things,
            // but for the purposes of this editor i think this i enough
            let tile_state = match mapgen_vehicle.status {
                VehicleStatus::LightDamage => {
                    if random_range(0..3) == 0 {
                        TileState::Broken
                    } else {
                        TileState::Normal
                    }
                },
                VehicleStatus::HeavilyDamaged => {
                    if random_range(0..5) == 0 {
                        TileState::Normal
                    } else {
                        TileState::Broken
                    }
                },
                VehicleStatus::Perfect | VehicleStatus::Undamaged => {
                    TileState::Normal
                },
            };

            commands.push(SetTile::furniture(
                TilesheetCDDAId {
                    id: part.id.clone(),
                    prefix: Some("vp".to_string()),
                    postfix: ty.map(|t| t.variant),
                },
                position + pos,
                rotation,
                tile_state,
            ));
        }

        Some(commands)
    }
}

impl Property for CorpsesProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let mapgen_corpse = self.corpses.get_random();

        let group = match json_data.monster_groups.get(&mapgen_corpse.group) {
            None => {
                warn!("Could not find monstergroup {}", mapgen_corpse.group);
                return None;
            },
            Some(g) => g,
        };

        let monster = match group.get_random_monster(
            &json_data.monster_groups,
            &map_data.calculated_parameters,
        ) {
            Ok(m) => m,
            Err(e) => {
                warn!("Could not get random monster {}", e);
                return None;
            },
        };

        Some(vec![SetTile {
            id: TilesheetCDDAId {
                id: monster,
                prefix: Some("corpse".into()),
                postfix: None,
            },
            layer: TileLayer::Monster,
            coordinates: position.clone(),
            rotation: Rotation::Deg0,
            state: TileState::Normal,
        }])
    }
}
