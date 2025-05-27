use crate::cdda_data::map_data::{
    MapGenComputer, MapGenField, MapGenGaspump, MapGenItem, MapGenMonster,
    MapGenSign, MapGenTrap, PlaceInnerComputers, PlaceInnerFields,
    PlaceInnerFurniture, PlaceInnerGaspumps, PlaceInnerItems,
    PlaceInnerMonsters, PlaceInnerSigns, PlaceInnerTerrain, PlaceInnerToilets,
    PlaceInnerTraps, PlaceInnerVehicles,
};
use crate::cdda_data::map_data::{
    MapGenCorpse, MapGenVehicle, PlaceInnerCorpses,
};
use crate::map::MapGenNested;
use cdda_lib::types::MapGenValue;
use cdda_lib::types::Weighted;

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
    pub monster: Vec<Weighted<MapGenMonster>>,
}

impl From<PlaceInnerMonsters> for MonstersProperty {
    fn from(value: PlaceInnerMonsters) -> Self {
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
pub struct ItemsProperty {
    pub items: Vec<Weighted<MapGenItem>>,
}

impl From<PlaceInnerItems> for ItemsProperty {
    fn from(value: PlaceInnerItems) -> Self {
        Self {
            items: vec![Weighted::new(value.value, 1)],
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
