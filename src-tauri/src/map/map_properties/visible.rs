use crate::cdda_data::io::NULL_NESTED;
use crate::cdda_data::map_data::MapGenNestedIntermediate;
use crate::map::*;
use crate::tileset::GetRandom;
use crate::util::{MeabyVec, MeabyWeighted, Weighted};
use log::error;

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
            mapping: VisibleMappingKind::Terrain,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
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
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        for monster in self.monster.clone().into_vec() {
            let ident = match monster
                .chance
                .clone()
                .unwrap_or(NumberOrRange::Number(1))
                .is_random_hit(100)
            {
                true => match monster.id {
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
                        mapping: VisibleMappingKind::Monster,
                        coordinates: position.clone(),
                        kind: VisibleMappingCommandKind::Place,
                    };

                    return Some(vec![command]);
                }
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
            mapping: VisibleMappingKind::Furniture,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }
}

#[derive(Debug, Clone)]
pub struct NestedProperty {
    pub nested: MapGenNested,
}

impl RepresentativeProperty for NestedProperty {
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        todo!()
    }
}

impl VisibleProperty for NestedProperty {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let should_place = match &self.nested.neighbors {
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

        if self.nested.invert_condition {
            if should_place {
                return None;
            }
        } else if !should_place {
            return None;
        }

        let selected_chunk = self
            .nested
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
}

#[cfg(test)]
mod tests {
    use crate::cdda_data::{CDDADistributionInner, MapGenValue};
    use crate::map::map_properties::visible::TerrainProperty;
    use crate::map::{
        MapData, VisibleMappingCommand, VisibleMappingCommandKind, VisibleMappingKind,
        VisibleProperty,
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
                    mapping: VisibleMappingKind::Terrain,
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
