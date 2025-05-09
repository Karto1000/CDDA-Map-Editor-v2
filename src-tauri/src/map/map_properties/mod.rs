use crate::cdda_data::map_data::{MapGenField, MapGenGaspumpFuelType, MapGenItem, MapGenMonster};
use crate::cdda_data::MapGenValue;
use crate::map::MapGenNested;
use crate::util::MeabyVec;

pub(crate) mod impl_property;

#[derive(Debug, Clone)]
pub struct TerrainProperty {
    pub mapgen_value: MapGenValue,
}

#[derive(Debug, Clone)]
pub struct MonsterProperty {
    pub monster: MeabyVec<MapGenMonster>,
}

#[derive(Debug, Clone)]
pub struct SignProperty {
    pub text: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GaspumpProperty {
    pub fuel: Option<MapGenGaspumpFuelType>,
    pub amount: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct FurnitureProperty {
    pub mapgen_value: MapGenValue,
}

#[derive(Debug, Clone)]
pub struct NestedProperty {
    pub nested: MapGenNested,
}

#[derive(Debug, Clone)]
pub struct FieldProperty {
    pub field: MapGenField,
}

#[derive(Debug, Clone)]
pub struct ItemProperty {
    pub items: Vec<MapGenItem>,
}
