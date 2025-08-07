use crate::data::item::{ItemEntry, ItemGroupSubtype};
use crate::data::map_data::{MapGenGaspumpFuelType, VehicleStatus};
use crate::data::vehicle_parts::{CDDAVehiclePart, Location};
use crate::data::vehicles::VehiclePart;
use crate::features::map::map_properties::{
    ComputersProperty, CorpsesProperty, FieldsProperty, FurnitureProperty,
    GaspumpsProperty, MonstersProperty, NestedProperty, SignsProperty,
    TerrainProperty, ToiletsProperty, TrapsProperty, VehiclesProperty,
};
use crate::features::map::*;
use crate::util::GetRandom;
use anyhow::{anyhow, Error};
use cdda_lib::{NULL_FIELD, NULL_NESTED, NULL_TRAP};
use log::error;
use num_traits::real::Real;
use rand::prelude::IndexedRandom;
use rand::random_range;
use serde_json::Map;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

impl Property for TerrainProperty {
    fn apply_to_instantiation(
        &self,
        _mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), anyhow::Error> {
        let mut rng = rng();

        let ident = self.mapgen_value.get_random_identifier(
            &mut rng,
            &instantiation.calculated_parameters.0,
        )?;

        if ident == CDDAIdentifier::from(NULL_TERRAIN) {
            instantiation.place_terrain(
                position.into(),
                TilesheetCDDAId::simple(NULL_TERRAIN),
            );

            return Ok(());
        }

        instantiation.place_terrain(
            MapgenCellCoordinates::from(position),
            MappedCDDAId::simple(TilesheetCDDAId::simple(
                json_data.replace_possible_region_settings(&mut rng, ident),
            )),
        );

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        let ident = self
            .mapgen_value
            .get_constant_identifier(calculated_parameters)
            .ok()?;

        Some(Representation {
            id: TilesheetCDDAId::simple(ident),
            tile_layer: TileLayer::Terrain,
        })
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.mapgen_value).unwrap()
    }
}

impl Property for MonstersProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), anyhow::Error> {
        let mut rng = rng();

        let monster = self.monster.get_random(&mut rng);

        let ident = match monster
            .chance
            .clone()
            .unwrap_or(NumberOrRange::Number(1))
            .is_random_hit(&mut rng, 100)
        {
            true => match &monster.id {
                MapGenMonsterType::Monster { monster } => monster
                    .get_random_identifier(
                        &mut rng,
                        &instantiation.calculated_parameters.0,
                    )
                    .ok(),
                MapGenMonsterType::MonsterGroup { group } => {
                    let id = group.get_random_identifier(
                        &mut rng,
                        &instantiation.calculated_parameters.0,
                    )?;
                    let mon_group = json_data.monster_groups.get(&id).ok_or(
                        anyhow::anyhow!("Could not find monster group {}", id),
                    )?;

                    let rand_monster = mon_group.get_random_monster(
                        &mut rng,
                        &json_data.monster_groups,
                        &instantiation.calculated_parameters.0,
                    )?;

                    Some(rand_monster.get_random_identifier(
                        &mut rng,
                        &instantiation.calculated_parameters.0,
                    )?)
                },
            },
            false => None,
        };

        match ident {
            None => Ok(()),
            Some(ident) => {
                instantiation.place_monster(
                    position.into(),
                    TilesheetCDDAId::simple(ident),
                );

                Ok(())
            },
        }
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        let first_id = self.monster.first()?;

        match &first_id.data.id {
            MapGenMonsterType::Monster { monster } => {
                let monster = monster
                    .get_constant_identifier(calculated_parameters)
                    .ok()?;
                Some(Representation {
                    id: TilesheetCDDAId::simple(monster),
                    tile_layer: TileLayer::Monster,
                })
            },
            // TODO
            MapGenMonsterType::MonsterGroup { .. } => None,
        }
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.monster).unwrap()
    }
}

impl Property for FurnitureProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();

        let ident = self.mapgen_value.get_random_identifier(
            &mut rng,
            &instantiation.calculated_parameters.0,
        )?;

        if ident == CDDAIdentifier::from(NULL_FURNITURE) {
            return Ok(());
        }

        instantiation
            .place_furniture(position.into(), TilesheetCDDAId::simple(ident));

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        let id = self
            .mapgen_value
            .get_constant_identifier(calculated_parameters)
            .ok()?;

        Some(Representation {
            id: TilesheetCDDAId::simple(id),
            tile_layer: TileLayer::Furniture,
        })
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.mapgen_value).unwrap()
    }
}

#[derive(Debug, Clone, Serialize)]
struct SignRepresentation {
    pub signage: String,
    pub snipped: String,
}

impl Property for SignsProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        instantiation.place_furniture(
            position.into(),
            TilesheetCDDAId::simple("f_sign"),
        );

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.signs).unwrap()
    }
}

impl Property for NestedProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();
        let nested_chunk = self.nested.get_random(&mut rng);

        let selected_chunk = nested_chunk
            .chunks
            .get_random(&mut rng)
            .get_random_identifier(
                &mut rng,
                &instantiation.calculated_parameters.0,
            )?;

        if selected_chunk == CDDAIdentifier::from(NULL_NESTED) {
            return Ok(());
        }

        let nested_mapgen = match json_data.map_data.get(&selected_chunk) {
            None => {
                return Err(anyhow!(
                    "Nested Mapgen {} not found",
                    selected_chunk
                ));
            },
            Some(v) => v,
        };

        let instantiated_mapgen = InstantiatedMapgen::new()
            .calculate_parameters(
                CalculateRandomParameters,
                nested_mapgen,
                json_data,
            )?;

        instantiation.place_nested(position.into(), instantiated_mapgen);

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.nested).unwrap()
    }
}

impl Property for FieldsProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();
        let field = self.field.get_random(&mut rng);

        if field.field == CDDAIdentifier::from(NULL_FIELD) {
            return Ok(());
        }

        instantiation.place_field(
            position.into(),
            TilesheetCDDAId::simple(field.field.clone()),
        );

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.field).unwrap()
    }
}

impl Property for GaspumpsProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();
        let gaspump = self.gaspumps.get_random(&mut rng);

        let id = match &gaspump.fuel {
            None => "t_gas_pump",
            Some(fuel) => match fuel {
                MapGenGaspumpFuelType::Gasoline
                | MapGenGaspumpFuelType::Avgas => "t_gas_pump",
                MapGenGaspumpFuelType::Diesel => "t_diesel_pump",
                MapGenGaspumpFuelType::Jp8 => "t_jp8_pump",
            },
        };

        instantiation
            .place_furniture(position.into(), TilesheetCDDAId::simple(id));

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.gaspumps).unwrap()
    }
}

impl Property for ComputersProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        instantiation.place_furniture(
            position.into(),
            TilesheetCDDAId::simple("f_console"),
        );

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.computer).unwrap()
    }
}

impl Property for ToiletsProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        instantiation.place_furniture(
            position.into(),
            TilesheetCDDAId::simple("f_toilet"),
        );

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        Value::Object(Map::new())
    }
}

impl Property for TrapsProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();
        let trap = self.trap.get_random(&mut rng);
        let ident = trap.get_random_identifier(
            &mut rng,
            &instantiation.calculated_parameters.0,
        )?;

        if ident == CDDAIdentifier::from(NULL_TRAP) {
            return Ok(());
        }

        instantiation
            .place_furniture(position.into(), TilesheetCDDAId::simple(ident));

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.trap).unwrap()
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
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();
        let mapgen_vehicle = self.vehicles.get_random(&mut rng);

        let vehicle = match json_data.vehicles.get(&mapgen_vehicle.vehicle) {
            None => {
                return Err(anyhow!(
                    "Vehicle {} not found",
                    mapgen_vehicle.vehicle
                ));
            },
            Some(v) => v,
        };

        let mut highest_priority_parts: HashMap<
            IVec2,
            (&CDDAVehiclePart, Option<VehiclePartSpriteVariant>, usize),
        > = HashMap::new();

        let random_rotation = mapgen_vehicle
            .rotation
            .clone()
            .into_vec()
            .choose(&mut rng)
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

            instantiation.place_furniture(
                (position.as_ivec2() + pos).as_uvec2().into(),
                MappedCDDAId {
                    tilesheet_id: TilesheetCDDAId {
                        id: part.id.clone(),
                        prefix: Some("vp".to_string()),
                        postfix: ty.map(|t| t.variant),
                    },
                    rotation,
                    is_broken: tile_state == TileState::Broken,
                    is_open: false,
                },
            );
        }

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.vehicles).unwrap()
    }
}

impl Property for CorpsesProperty {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        let mut rng = rng();
        let mapgen_corpse = self.corpses.get_random(&mut rng);

        let group = match json_data.monster_groups.get(&mapgen_corpse.group) {
            None => {
                return Err(anyhow!(
                    "Could not find monstergroup {}",
                    mapgen_corpse.group
                ));
            },
            Some(g) => g,
        };

        let monster = match group.get_random_monster(
            &mut rng,
            &json_data.monster_groups,
            &instantiation.calculated_parameters.0,
        ) {
            Ok(m) => m,
            Err(e) => {
                return Err(anyhow!("Could not get random monster {}", e));
            },
        };

        instantiation.place_monster(
            position.into(),
            TilesheetCDDAId {
                id: monster.clone(),
                prefix: Some("corpse".into()),
                postfix: None,
            },
        );

        Ok(())
    }

    fn representation(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<Representation> {
        None
    }

    fn value(&self) -> Value {
        serde_json::to_value(&self.corpses).unwrap()
    }
}
