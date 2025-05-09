use crate::cdda_data::map_data::{MapGenField, MapGenItem, MapGenMonster};
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
pub enum FurniturePropertySubtype {
    Furniture,
    Computer,
    Sign,
    Toilet,
    Gaspump,
}

#[derive(Debug, Clone)]
pub struct FurnitureProperty {
    pub mapgen_value: MapGenValue,
    pub subtype: FurniturePropertySubtype,
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
