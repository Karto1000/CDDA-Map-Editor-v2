use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::map::map_properties::representative::ItemProperty;
use crate::map::map_properties::visible::{
    FieldProperty, FurnitureProperty, MonsterProperty, NestedProperty, TerrainProperty,
};
use crate::map::{
    MapData, Place, RepresentativeProperty, VisibleMappingCommand, VisibleMappingCommandKind,
    VisibleMappingKind, VisibleProperty,
};
use glam::IVec2;
use serde_json::Value;

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
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible.get_commands(position, map_data, json_data)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        self.visible.representation(json_data)
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
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible.get_commands(position, map_data, json_data)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        self.visible.representation(json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceItems {
    pub representative: ItemProperty,
}

impl Place for PlaceItems {}

#[derive(Debug, Clone)]
pub struct PlaceMonster {
    pub visible: MonsterProperty,
}

impl Place for PlaceMonster {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible.get_commands(position, map_data, json_data)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        self.visible.representation(json_data)
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
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.nested_property
            .get_commands(position, map_data, json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceToilets;

impl Place for PlaceToilets {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        let command = VisibleMappingCommand {
            id: "f_toilet".into(),
            mapping: VisibleMappingKind::Furniture,
            coordinates: position.clone(),
            kind: VisibleMappingCommandKind::Place,
        };

        Some(vec![command])
    }
}

#[derive(Debug, Clone)]
pub struct PlaceFields {
    pub visible: FieldProperty,
}

impl Place for PlaceFields {
    fn get_commands(
        &self,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible.get_commands(position, map_data, json_data)
    }
}
