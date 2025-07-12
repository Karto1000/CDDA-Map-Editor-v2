use crate::data::io::DeserializedCDDAJsonData;
use crate::features::map::map_properties::{
    FurnitureProperty, NestedProperty, TerrainProperty,
};
use crate::features::map::{
    InstantiatedMapgen, MapGen, ParametersCalculated, Place, Property,
};
use anyhow::Error;
use glam::UVec2;

#[derive(Debug, Clone)]
pub struct PlaceTerrain {
    pub visible: TerrainProperty,
}

impl Place for PlaceTerrain {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        self.visible.apply_to_instantiation(
            mapgen,
            instantiation,
            position,
            json_data,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PlaceFurniture {
    pub visible: FurnitureProperty,
}

impl Place for PlaceFurniture {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        self.visible.apply_to_instantiation(
            mapgen,
            instantiation,
            position,
            json_data,
        )
    }
}

#[derive(Debug, Clone)]
pub struct PlaceNested {
    pub nested_property: NestedProperty,
}

impl Place for PlaceNested {
    fn apply_to_instantiation(
        &self,
        mapgen: &MapGen,
        instantiation: &mut InstantiatedMapgen<ParametersCalculated>,
        position: UVec2,
        json_data: &DeserializedCDDAJsonData,
    ) -> Result<(), Error> {
        self.nested_property.apply_to_instantiation(
            mapgen,
            instantiation,
            position,
            json_data,
        )
    }
}
