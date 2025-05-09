use crate::cdda_data::map_data::{
    MapGenComputer, MapGenField, MapGenGaspump, MapGenItem, MapGenMonster,
    MapGenSign, MapGenTrap, PlaceInnerComputer, PlaceInnerField, PlaceInnerFurniture,
    PlaceInnerGaspump, PlaceInnerItems, PlaceInnerMonsters, PlaceInnerSign, PlaceInnerTerrain,
    PlaceInnerToilet, PlaceInnerTraps,
};
use crate::cdda_data::MapGenValue;
use crate::map::MapGenNested;
use crate::util::Weighted;

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
pub struct SignProperty {
    pub signs: Vec<Weighted<MapGenSign>>,
}

impl From<PlaceInnerSign> for SignProperty {
    fn from(value: PlaceInnerSign) -> Self {
        Self {
            signs: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct GaspumpProperty {
    pub gaspumps: Vec<Weighted<MapGenGaspump>>,
}

impl From<PlaceInnerGaspump> for GaspumpProperty {
    fn from(value: PlaceInnerGaspump) -> Self {
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
pub struct FieldProperty {
    pub field: Vec<Weighted<MapGenField>>,
}

impl From<PlaceInnerField> for FieldProperty {
    fn from(value: PlaceInnerField) -> Self {
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
pub struct ComputerProperty {
    computer: Vec<Weighted<MapGenComputer>>,
}

impl From<PlaceInnerComputer> for ComputerProperty {
    fn from(value: PlaceInnerComputer) -> Self {
        Self {
            computer: vec![Weighted::new(value.value, 1)],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToiletProperty;

impl From<PlaceInnerToilet> for ToiletProperty {
    fn from(value: PlaceInnerToilet) -> Self {
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
