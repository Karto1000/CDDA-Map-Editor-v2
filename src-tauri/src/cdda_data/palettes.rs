use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::map_data::{MapGenItem, MapGenMonsters};
use crate::cdda_data::GetIdentifier;
use crate::cdda_data::KnownCataVariant;
use crate::map::map_properties::ItemsProperty;
use crate::map::map_properties::{
    FurnitureProperty, MonstersProperty, TerrainProperty,
};
use crate::map::{
    CalculateParametersError, MapData, MappingKind, Property, SetTile,
};
use cdda_lib::types::{
    CDDAIdentifier, Comment, Distribution, MapGenValue, MeabyVec,
    MeabyWeighted, ParameterIdentifier,
};
use futures_lite::StreamExt;
use glam::IVec2;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

pub type Palettes = HashMap<CDDAIdentifier, CDDAPalette>;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum ParameterScope {
    // https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/JSON/MAPGEN.md#mapgen-parameters
    // "By default, the scope of a parameter is the overmap_special being generated."
    #[serde(rename = "overmap_special")]
    #[default]
    OvermapSpecial,

    #[serde(rename = "nest")]
    Nest,

    #[serde(rename = "omt")]
    Omt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub ty: KnownCataVariant,

    #[serde(rename = "//")]
    pub comment: Comment,

    pub scope: Option<ParameterScope>,

    pub default: Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAPaletteIntermediate {
    pub id: CDDAIdentifier,

    #[serde(rename = "//")]
    pub comment: Comment,

    #[serde(default)]
    pub parameters: HashMap<ParameterIdentifier, Parameter>,

    #[serde(default)]
    pub palettes: Vec<MapGenValue>,

    #[serde(default)]
    pub terrain: HashMap<char, MapGenValue>,

    #[serde(default)]
    pub furniture: HashMap<char, MapGenValue>,

    #[serde(default)]
    pub monster: HashMap<char, MeabyVec<MeabyWeighted<MapGenMonsters>>>,

    #[serde(default)]
    pub monsters: HashMap<char, Value>,

    #[serde(default)]
    pub npcs: HashMap<char, Value>,

    #[serde(default)]
    pub items: HashMap<char, MeabyVec<MeabyWeighted<MapGenItem>>>,

    #[serde(default)]
    pub loot: HashMap<char, Value>,

    #[serde(default)]
    pub sealed_item: HashMap<char, Value>,

    #[serde(default)]
    pub fields: HashMap<char, Value>,

    #[serde(default)]
    pub signs: HashMap<char, Value>,

    #[serde(default)]
    pub rubble: HashMap<char, Value>,

    #[serde(default)]
    pub liquids: HashMap<char, Value>,

    #[serde(default)]
    pub corpses: HashMap<char, Value>,

    #[serde(default)]
    pub computers: HashMap<char, Value>,

    #[serde(default)]
    pub nested: HashMap<char, Value>,

    #[serde(default)]
    pub toilets: HashMap<char, Value>,

    #[serde(default)]
    pub gaspumps: HashMap<char, Value>,

    #[serde(default)]
    pub vehicles: HashMap<char, Value>,

    #[serde(default)]
    pub traps: HashMap<char, Value>,

    #[serde(default)]
    pub graffiti: HashMap<char, Value>,
}

impl Into<CDDAPalette> for CDDAPaletteIntermediate {
    fn into(self) -> CDDAPalette {
        let mut properties = HashMap::new();

        let mut terrain_map = HashMap::new();
        for (char, terrain) in self.terrain {
            let ter_prop = Arc::new(TerrainProperty {
                mapgen_value: terrain,
            });

            terrain_map.insert(char, ter_prop as Arc<dyn Property>);
        }

        let mut furniture_map = HashMap::new();
        for (char, furniture) in self.furniture {
            let fur_prop = Arc::new(FurnitureProperty {
                mapgen_value: furniture,
            });

            furniture_map.insert(char, fur_prop as Arc<dyn Property>);
        }

        let mut monster_map = HashMap::new();
        for (char, monster) in self.monster {
            let monster_prop = Arc::new(MonstersProperty {
                monster: monster
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });

            monster_map.insert(char, monster_prop as Arc<dyn Property>);
        }

        let mut item_map = HashMap::new();
        for (char, items) in self.items {
            let item_prop = Arc::new(ItemsProperty {
                items: items
                    .into_vec()
                    .into_iter()
                    .map(MeabyWeighted::to_weighted)
                    .collect(),
            });
            item_map.insert(char, item_prop as Arc<dyn Property>);
        }

        properties.insert(MappingKind::Terrain, terrain_map);
        properties.insert(MappingKind::Furniture, furniture_map);
        properties.insert(MappingKind::Monsters, monster_map);
        properties.insert(MappingKind::ItemGroups, item_map);

        CDDAPalette {
            id: self.id,
            properties,
            comment: self.comment,
            parameters: self.parameters,
            palettes: self.palettes,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAPalette {
    pub id: CDDAIdentifier,

    #[serde(skip)]
    pub properties: HashMap<MappingKind, HashMap<char, Arc<dyn Property>>>,

    #[serde(rename = "//")]
    pub comment: Comment,

    #[serde(default)]
    pub parameters: HashMap<ParameterIdentifier, Parameter>,

    #[serde(default)]
    pub palettes: Vec<MapGenValue>,
}

impl CDDAPalette {
    pub fn calculate_parameters(
        &self,
        all_palettes: &Palettes,
    ) -> Result<
        IndexMap<ParameterIdentifier, CDDAIdentifier>,
        CalculateParametersError,
    > {
        let mut calculated_parameters: IndexMap<
            ParameterIdentifier,
            CDDAIdentifier,
        > = IndexMap::new();

        for (id, parameter) in self.parameters.iter() {
            calculated_parameters.insert(
                id.clone(),
                parameter
                    .default
                    .distribution
                    .get_identifier(&calculated_parameters)?,
            );
        }

        for mapgen_value in self.palettes.iter() {
            let id = mapgen_value.get_identifier(&calculated_parameters)?;

            all_palettes
                .get(&id)
                .ok_or(CalculateParametersError::MissingPalette(id.0))?
                .calculate_parameters(all_palettes)?
                .into_iter()
                .for_each(|(child_id, child_param)| {
                    calculated_parameters.insert(child_id, child_param);
                })
        }

        Ok(calculated_parameters)
    }

    pub fn get_visible_mapping(
        &self,
        mapping_kind: impl Borrow<MappingKind>,
        character: impl Borrow<char>,
        position: &IVec2,
        map_data: &MapData,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Vec<SetTile>> {
        let mapping = self.properties.get(mapping_kind.borrow())?;

        if let Some(id) = mapping.get(character.borrow()) {
            return id.get_commands(position, map_data, json_data);
        }

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value
                .get_identifier(&map_data.calculated_parameters)
                .ok()?;
            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) = palette.get_visible_mapping(
                mapping_kind.borrow(),
                character.borrow(),
                position,
                map_data,
                json_data,
            ) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_representative_mapping(
        &self,
        mapping_kind: impl Borrow<MappingKind>,
        character: impl Borrow<char>,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Value> {
        let mapping = self.properties.get(mapping_kind.borrow())?;

        match mapping.get(character.borrow()) {
            None => {},
            Some(s) => return Some(s.representation(json_data)),
        }

        for mapgen_value in self.palettes.iter() {
            let palette_id =
                mapgen_value.get_identifier(calculated_parameters).ok()?;
            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) = palette.get_representative_mapping(
                mapping_kind.borrow(),
                character.borrow(),
                calculated_parameters,
                json_data,
            ) {
                return Some(id);
            }
        }

        None
    }
}
