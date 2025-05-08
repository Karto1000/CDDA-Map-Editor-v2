use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::map::{Place, RepresentativeProperty, VisibleMappingCommand, VisibleProperty};
use crate::util::{CDDAIdentifier, ParameterIdentifier};
use glam::UVec2;
use indexmap::IndexMap;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PlaceTerrain {
    pub visible: Arc<dyn VisibleProperty>,
}

impl Place for PlaceTerrain {
    fn get_commands(
        &self,
        position: &UVec2,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible
            .get_commands(calculated_parameters, position, json_data)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        self.visible.representation(json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceFurniture {
    pub visible: Arc<dyn VisibleProperty>,
}

impl Place for PlaceFurniture {
    fn get_commands(
        &self,
        position: &UVec2,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible
            .get_commands(calculated_parameters, position, json_data)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        self.visible.representation(json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceItems {
    pub representative: Arc<dyn RepresentativeProperty>,
}

impl Place for PlaceItems {}

#[derive(Debug, Clone)]
pub struct PlaceMonster {
    pub visible: Arc<dyn VisibleProperty>,
}

impl Place for PlaceMonster {
    fn get_commands(
        &self,
        position: &UVec2,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<VisibleMappingCommand>> {
        self.visible
            .get_commands(calculated_parameters, position, json_data)
    }

    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        self.visible.representation(json_data)
    }
}

#[derive(Debug, Clone)]
pub struct PlaceNested {}

impl Place for PlaceNested {}
