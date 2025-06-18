use crate::data::map_data::{
    MapGenComputer, MapGenField, MapGenGaspump, MapGenMonsters, MapGenSign,
    MapGenTrap, PlaceInnerComputers, PlaceInnerFields, PlaceInnerFurniture,
    PlaceInnerGaspumps, PlaceInnerMonster, PlaceInnerMonsters, PlaceInnerSigns,
    PlaceInnerTerrain, PlaceInnerToilets, PlaceInnerTraps, PlaceInnerVehicles,
};
use crate::data::map_data::{MapGenCorpse, MapGenVehicle, PlaceInnerCorpses};
use crate::features::map::{MapGenNested, MappingKind, Property};
use cdda_lib::types::MapGenValue;
use cdda_lib::types::Weighted;
use serde_json::Value;
use std::sync::Arc;

pub(crate) mod impl_property;

#[derive(Debug, Clone)]
pub struct TerrainProperty {
    pub mapgen_value: MapGenValue,
}

impl From<PlaceInnerTerrain> for TerrainProperty {
    fn from(value: PlaceInnerTerrain) -> Self {
        Self {
            mapgen_value: value.terrain_id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonstersProperty {
    pub monster: Vec<Weighted<MapGenMonsters>>,
}

impl From<PlaceInnerMonsters> for MonstersProperty {
    fn from(value: PlaceInnerMonsters) -> Self {
        Self {
            monster: vec![Weighted::new(value.value, 1)],
        }
    }
}

impl From<PlaceInnerMonster> for MonstersProperty {
    fn from(value: PlaceInnerMonster) -> Self {
        Self {
            monster: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignsProperty {
    pub signs: Vec<Weighted<MapGenSign>>,
}

impl From<PlaceInnerSigns> for SignsProperty {
    fn from(value: PlaceInnerSigns) -> Self {
        Self {
            signs: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GaspumpsProperty {
    pub gaspumps: Vec<Weighted<MapGenGaspump>>,
}

impl From<PlaceInnerGaspumps> for GaspumpsProperty {
    fn from(value: PlaceInnerGaspumps) -> Self {
        Self {
            gaspumps: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct FurnitureProperty {
    pub mapgen_value: MapGenValue,
}

impl From<PlaceInnerFurniture> for FurnitureProperty {
    fn from(value: PlaceInnerFurniture) -> Self {
        Self {
            mapgen_value: value.furniture_id.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NestedProperty {
    pub nested: Vec<Weighted<MapGenNested>>,
}

#[derive(Debug, Clone)]
pub struct FieldsProperty {
    pub field: Vec<Weighted<MapGenField>>,
}

impl From<PlaceInnerFields> for FieldsProperty {
    fn from(value: PlaceInnerFields) -> Self {
        Self {
            field: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComputersProperty {
    computer: Vec<Weighted<MapGenComputer>>,
}

impl From<PlaceInnerComputers> for ComputersProperty {
    fn from(value: PlaceInnerComputers) -> Self {
        Self {
            computer: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToiletsProperty;

impl From<PlaceInnerToilets> for ToiletsProperty {
    fn from(value: PlaceInnerToilets) -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
pub struct TrapsProperty {
    pub trap: Vec<Weighted<MapGenValue>>,
}

impl From<PlaceInnerTraps> for TrapsProperty {
    fn from(value: PlaceInnerTraps) -> Self {
        let mgv = match value.value {
            MapGenTrap::TrapRef { trap } => trap,
            MapGenTrap::MapGenValue(mgv) => mgv,
        };

        Self {
            trap: vec![Weighted::new(mgv, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct VehiclesProperty {
    pub vehicles: Vec<Weighted<MapGenVehicle>>,
}

impl From<PlaceInnerVehicles> for VehiclesProperty {
    fn from(value: PlaceInnerVehicles) -> Self {
        Self {
            vehicles: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct CorpsesProperty {
    pub corpses: Vec<Weighted<MapGenCorpse>>,
}

impl From<PlaceInnerCorpses> for CorpsesProperty {
    fn from(value: PlaceInnerCorpses) -> Self {
        Self {
            corpses: vec![Weighted::new(value.value, 1)],
        }
    }
}

pub fn value_to_property(
    kind: MappingKind,
    value: Value,
) -> serde_json::Result<Arc<dyn Property>> {
    match kind {
        MappingKind::Terrain => {
            let mapgen_value = serde_json::from_value::<MapGenValue>(value)?;
            Ok(Arc::new(TerrainProperty { mapgen_value }))
        },
        MappingKind::Furniture => {
            let mapgen_value = serde_json::from_value::<MapGenValue>(value)?;
            Ok(Arc::new(FurnitureProperty { mapgen_value }))
        },
        MappingKind::Trap => {
            let trap =
                serde_json::from_value::<Vec<Weighted<MapGenValue>>>(value)?;
            Ok(Arc::new(TrapsProperty { trap }))
        },
        MappingKind::ItemGroups => unimplemented!(),
        MappingKind::Computer => {
            let computer =
                serde_json::from_value::<Vec<Weighted<MapGenComputer>>>(value)?;
            Ok(Arc::new(ComputersProperty { computer }))
        },
        MappingKind::Sign => {
            let signs =
                serde_json::from_value::<Vec<Weighted<MapGenSign>>>(value)?;
            Ok(Arc::new(SignsProperty { signs }))
        },
        MappingKind::Toilet => Ok(Arc::new(ToiletsProperty)),
        MappingKind::Gaspump => {
            let gaspumps =
                serde_json::from_value::<Vec<Weighted<MapGenGaspump>>>(value)?;
            Ok(Arc::new(GaspumpsProperty { gaspumps }))
        },
        MappingKind::Monsters => {
            let monster =
                serde_json::from_value::<Vec<Weighted<MapGenMonsters>>>(value)?;
            Ok(Arc::new(MonstersProperty { monster }))
        },
        MappingKind::Monster => {
            let monster =
                serde_json::from_value::<Vec<Weighted<MapGenMonsters>>>(value)?;
            Ok(Arc::new(MonstersProperty { monster }))
        },
        MappingKind::Field => {
            let field =
                serde_json::from_value::<Vec<Weighted<MapGenField>>>(value)?;
            Ok(Arc::new(FieldsProperty { field }))
        },
        MappingKind::Nested => {
            let nested =
                serde_json::from_value::<Vec<Weighted<MapGenNested>>>(value)?;
            Ok(Arc::new(NestedProperty { nested }))
        },
        MappingKind::Vehicle => {
            let vehicles =
                serde_json::from_value::<Vec<Weighted<MapGenVehicle>>>(value)?;
            Ok(Arc::new(VehiclesProperty { vehicles }))
        },
        MappingKind::Corpse => {
            let corpses =
                serde_json::from_value::<Vec<Weighted<MapGenCorpse>>>(value)?;
            Ok(Arc::new(CorpsesProperty { corpses }))
        },
    }
}
