use crate::data::io::DeserializedCDDAJsonData;
use crate::features::map::map_properties::{
    FurnitureProperty, NestedProperty, TerrainProperty,
};
use crate::features::map::{MapData, Place, Property, SetTile};
use glam::IVec2;

#[derive(Debug, Clone)]
pub struct PlaceTerrain {
    pub visible: TerrainProperty,
}

impl Place for PlaceTerrain {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        self.visible.get_commands(position, map_data, json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceFurniture {
    pub visible: FurnitureProperty,
}

impl Place for PlaceFurniture {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        self.visible.get_commands(position, map_data, json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceNested {
    pub nested_property: NestedProperty,
}

impl Place for PlaceNested {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        self.nested_property
            .get_commands(position, map_data, json_data)
    }
}
