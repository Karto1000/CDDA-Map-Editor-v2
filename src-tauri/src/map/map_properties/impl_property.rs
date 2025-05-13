use crate::cdda_data::io::{NULL_FIELD, NULL_NESTED, NULL_TRAP};
use crate::cdda_data::item::{ItemEntry, ItemGroupSubtype};
use crate::cdda_data::map_data::{
    MapGenField, MapGenGaspumpFuelType, MapGenNestedIntermediate, ReferenceOrInPlace,
};
use crate::map::map_properties::{
    ComputersProperty, FieldsProperty, FurnitureProperty, GaspumpsProperty, ItemsProperty,
    MonstersProperty, NestedProperty, SignsProperty, TerrainProperty, ToiletsProperty,
    TrapsProperty,
};
use crate::map::*;
use crate::tileset::GetRandom;
use crate::util::{MeabyVec, MeabyWeighted, Weighted};
use log::error;
use rand::prelude::IndexedRandom;
use serde_json::json;

impl Property for TerrainProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let ident = self
            .mapgen_value
            .get_identifier(&map_data.calculated_parameters);

        if ident == CDDAIdentifier::from(NULL_TERRAIN) {
            return None;
        }

        let command = VisibleMappingCommand {
            id: ident,
            mapping: MappingKind::Terrain,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

impl Property for MonstersProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let monster = self.monster.get_random();

        let ident = match monster
            .chance
            .clone()
            .unwrap_or(NumberOrRange::Number(1))
            .is_random_hit(100)
        {
            true => match &monster.id {
                MapGenMonsterType::Monster { monster } => {
                    Some(monster.get_identifier(&map_data.calculated_parameters))
                }
                MapGenMonsterType::MonsterGroup { group } => {
                    let id = group.get_identifier(&map_data.calculated_parameters);
                    let mon_group = json_data.monstergroups.get(&id)?;
                    mon_group
                        .get_random_monster(
                            &json_data.monstergroups,
                            &map_data.calculated_parameters,
                        )
                        .map(|id| id.get_identifier(&map_data.calculated_parameters))
                }
            },
            false => None,
        };

        match ident {
            None => {}
            Some(ident) => {
                let command = VisibleMappingCommand {
                    id: ident,
                    mapping: MappingKind::Monster,
                    coordinates: position.clone(),
                    kind: VisibleMappingCommandKind::Place,
                };

                return Some(vec![command]);
            }
        }

        None
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

impl Property for FurnitureProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let ident = self
            .mapgen_value
            .get_identifier(&map_data.calculated_parameters);

        if ident == CDDAIdentifier::from(NULL_FURNITURE) {
            return None;
        }

        let command = VisibleMappingCommand {
            id: ident,
            mapping: MappingKind::Furniture,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
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
    ) -> Option<Vec<VisibleMappingCommand>> {
        let command = VisibleMappingCommand {
            id: "f_sign".into(),
            mapping: MappingKind::Sign,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        let sign = self.signs.get_random();

        serde_json::to_value(SignRepresentation {
            signage: sign.signage.clone().unwrap_or("".into()),
            snipped: sign.snippet.clone().unwrap_or("".into()),
        })
        .unwrap()
    }
}

impl Property for NestedProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let mut rng = rng();
        let nested_chunk = self.nested.get_random();

        let should_place = match &nested_chunk.neighbors {
            None => true,
            Some(neighbors) => neighbors.iter().all(|(dir, om_terrain_match)| {
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
            }),
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
            .get_identifier(&map_data.calculated_parameters);

        if selected_chunk == CDDAIdentifier::from(NULL_NESTED) {
            return None;
        }

        let nested_mapgen = match json_data.map_data.get(&selected_chunk) {
            None => {
                error!("Nested Mapgen {} not found", selected_chunk);
                return None;
            }
            Some(v) => v,
        };

        let mut commands = nested_mapgen.get_commands(json_data);

        commands.iter_mut().for_each(|c| {
            c.coordinates.x += position.x;
            c.coordinates.y = position.y + c.coordinates.y;
        });

        Some(commands)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

impl Property for FieldsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let field = self.field.get_random();

        if field.field == CDDAIdentifier::from(NULL_FIELD) {
            return None;
        }

        let command = VisibleMappingCommand {
            id: field.field.clone(),
            mapping: MappingKind::Field,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

impl Property for GaspumpsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let gaspump = self.gaspumps.get_random();

        let id = match &gaspump.fuel {
            None => "t_gas_pump",
            Some(fuel) => match fuel {
                MapGenGaspumpFuelType::Gasoline | MapGenGaspumpFuelType::Avgas => "t_gas_pump",
                MapGenGaspumpFuelType::Diesel => "t_diesel_pump",
                MapGenGaspumpFuelType::Jp8 => "t_jp8_pump",
            },
        };

        let command = VisibleMappingCommand {
            id: id.into(),
            mapping: MappingKind::Gaspump,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
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
            DisplayItemGroup::Collection { probability, .. } => probability.clone(),
            DisplayItemGroup::Distribution { probability, .. } => probability.clone(),
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
            ItemEntry::Distribution { probability, .. } => acc + probability.unwrap_or(100),
            ItemEntry::Collection { probability, .. } => acc + probability.unwrap_or(100),
        });

        for entry in entries.iter() {
            match entry {
                ItemEntry::Item(i) => {
                    let display_item = DisplayItemGroup::Single {
                        item: i.item.clone(),
                        probability: i.probability as f32 / weight_sum as f32 * group_probability,
                    };
                    display_item_groups.push(display_item);
                }
                ItemEntry::Group(g) => {
                    let other_group = &json_data
                        .item_groups
                        .get(&g.group)
                        .expect(format!("Item Group {} to exist", &g.group).as_str());

                    let probability = g.probability as f32 / weight_sum as f32 * group_probability;

                    let display_items = self.get_display_items_from_entries(
                        &other_group.common.entries,
                        json_data,
                        probability,
                    );

                    match other_group.common.subtype {
                        ItemGroupSubtype::Collection => {
                            display_item_groups.push(DisplayItemGroup::Collection {
                                items: display_items,
                                name: Some(other_group.id.clone().0),
                                probability,
                            });
                        }
                        ItemGroupSubtype::Distribution => {
                            display_item_groups.push(DisplayItemGroup::Distribution {
                                items: display_items,
                                name: Some(other_group.id.clone().0),
                                probability,
                            });
                        }
                    }
                }
                ItemEntry::Distribution {
                    distribution,
                    probability,
                } => {
                    let probability = probability
                        .map(|p| p as f32 / weight_sum as f32 * group_probability)
                        .unwrap_or(group_probability / weight_sum as f32);

                    let display_items =
                        self.get_display_items_from_entries(distribution, json_data, probability);

                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some("In-Place".to_string()),
                        items: display_items,
                        probability,
                    });
                }
                ItemEntry::Collection {
                    collection,
                    probability,
                } => {
                    let probability = probability
                        .map(|p| p as f32 / weight_sum as f32 * group_probability)
                        .unwrap_or(group_probability / weight_sum as f32);

                    let display_items =
                        self.get_display_items_from_entries(collection, json_data, probability);

                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some("In-Place".to_string()),
                        items: display_items,
                        probability,
                    });
                }
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

impl Property for ItemsProperty {
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        let mut display_item_groups: Vec<DisplayItemGroup> = Vec::new();

        for mapgen_item in self.items.iter() {
            let item_group_entries = match &mapgen_item.data.item {
                ReferenceOrInPlace::Reference(i) => {
                    &json_data
                        .item_groups
                        .get(&i)
                        .expect(format!("Item group {} to exist", i).as_str())
                        .common
                }
                ReferenceOrInPlace::InPlace(ip) => &ip.common,
            };

            let probability = mapgen_item
                .data
                .chance
                .clone()
                .map(|v| v.get_from_to().0)
                .unwrap_or(100) as f32
                // the default chance is 100, but we want to have a range from 0-1 so / 100
                / 100.;

            let items = self.get_display_items_from_entries(
                &item_group_entries.entries,
                json_data,
                probability,
            );

            match &item_group_entries.subtype {
                ItemGroupSubtype::Collection => {
                    display_item_groups.push(DisplayItemGroup::Collection {
                        name: Some(mapgen_item.data.item.ref_or("Unnamed Collection").0),
                        probability,
                        items,
                    });
                }
                ItemGroupSubtype::Distribution => {
                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some(mapgen_item.data.item.ref_or("Unnamed Distribution").0),
                        probability,
                        items,
                    });
                }
            }
        }

        display_item_groups.sort_by(|v1, v2| {
            v2.probability()
                .partial_cmp(&v1.probability())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        serde_json::to_value(display_item_groups).unwrap()
    }
}

impl Property for ComputersProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let command = VisibleMappingCommand {
            id: "f_console".into(),
            mapping: MappingKind::Furniture,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

impl Property for ToiletsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let command = VisibleMappingCommand {
            id: "f_toilet".into(),
            mapping: MappingKind::Furniture,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

impl Property for TrapsProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let trap = self.trap.get_random();
        let ident = trap.get_identifier(&map_data.calculated_parameters);

        if ident == CDDAIdentifier::from(NULL_TRAP) {
            return None;
        }

        let command = VisibleMappingCommand {
            id: ident,
            mapping: MappingKind::Trap,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use crate::cdda_data::{CDDADistributionInner, MapGenValue};
    use crate::map::map_properties::TerrainProperty;
    use crate::map::{
        MapData, MappingKind, Property, VisibleMappingCommand, VisibleMappingCommandKind,
    };
    use crate::util::{MeabyVec, MeabyWeighted};
    use crate::TEST_CDDA_DATA;
    use glam::IVec2;

    #[tokio::test]
    async fn test_get_terrain_commands() {
        let cdda_data = TEST_CDDA_DATA.get().await;
        let coordinates = IVec2::new(0, 0);
        let map_data = MapData::default();

        // Test it with a single string
        {
            let terrain_property = TerrainProperty {
                mapgen_value: MapGenValue::String("t_grass".into()),
            };

            let mut commands = terrain_property
                .get_commands(&coordinates, &map_data, &cdda_data)
                .unwrap();

            let first = commands.pop().unwrap();

            assert_eq!(
                first,
                VisibleMappingCommand {
                    id: "t_grass".into(),
                    mapping: MappingKind::Terrain,
                    coordinates,
                    kind: VisibleMappingCommandKind::Place,
                }
            );
        }

        // Test it with a distribution
        {
            let distribution: MeabyVec<MeabyWeighted<CDDADistributionInner>> = MeabyVec::Vec(vec![
                MeabyWeighted::NotWeighted("t_grass".into()),
                MeabyWeighted::NotWeighted("t_dirt".into()),
            ]);

            let terrain_property = TerrainProperty {
                mapgen_value: MapGenValue::Distribution(distribution),
            };

            let mut commands = terrain_property
                .get_commands(&coordinates, &map_data, &cdda_data)
                .unwrap();

            let first = commands.pop().unwrap();

            assert!(first.id == "t_grass".into() || first.id == "t_dirt".into());
        }
    }
}
