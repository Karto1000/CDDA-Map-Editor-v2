use crate::cdda_data::map_data::{
    MapGenComputer, MapGenField, MapGenGaspumpFuelType, MapGenItem, MapGenMonster,
    PlaceInnerComputer, PlaceInnerField, PlaceInnerFurniture, PlaceInnerGaspump, PlaceInnerItems,
    PlaceInnerMonsters, PlaceInnerSign, PlaceInnerTerrain, PlaceInnerToilet,
};
use crate::cdda_data::MapGenValue;
use crate::map::MapGenNested;
use crate::util::MeabyVec;

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
    pub monster: MeabyVec<MapGenMonster>,
}

impl From<PlaceInnerMonsters> for MonstersProperty {
    fn from(value: PlaceInnerMonsters) -> Self {
        Self {
            monster: MeabyVec::Single(value.value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignProperty {
    pub text: Option<String>,
    pub snippet: Option<String>,
}

impl From<PlaceInnerSign> for SignProperty {
    fn from(value: PlaceInnerSign) -> Self {
        Self {
            text: value.value.signage,
            snippet: value.value.snippet,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GaspumpProperty {
    pub fuel: Option<MapGenGaspumpFuelType>,
    pub amount: Option<i32>,
}

impl From<PlaceInnerGaspump> for GaspumpProperty {
    fn from(value: PlaceInnerGaspump) -> Self {
        Self {
            fuel: value.value.fuel,
            amount: value.value.amount,
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
    pub nested: MapGenNested,
}

#[derive(Debug, Clone)]
pub struct FieldProperty {
    pub field: MapGenField,
}

impl From<PlaceInnerField> for FieldProperty {
    fn from(value: PlaceInnerField) -> Self {
        Self { field: value.value }
    }
}

#[derive(Debug, Clone)]
pub struct ItemsProperty {
    pub items: Vec<MapGenItem>,
}

impl From<PlaceInnerItems> for ItemsProperty {
    fn from(value: PlaceInnerItems) -> Self {
        Self {
            items: vec![value.value],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComputerProperty {
    computer: MapGenComputer,
}

impl From<PlaceInnerComputer> for ComputerProperty {
    fn from(value: PlaceInnerComputer) -> Self {
        Self {
            computer: value.value,
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
