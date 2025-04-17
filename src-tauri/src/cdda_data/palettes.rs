use crate::cdda_data::{CataVariant, Distribution, MapGenValue};
use crate::map::Mapping;
use crate::util::{CDDAIdentifier, Comment, GetIdentifier, ParameterIdentifier};
use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub type Palettes = HashMap<CDDAIdentifier, CDDAPalette>;

#[derive(Debug, Clone, Default, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub ty: CataVariant,

    #[serde(rename = "//")]
    pub comment: Comment,

    pub scope: Option<ParameterScope>,

    pub default: Distribution,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAPalette {
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
    pub monster: HashMap<char, Value>,

    #[serde(default)]
    pub monsters: HashMap<char, Value>,

    #[serde(default)]
    pub npcs: HashMap<char, Value>,

    #[serde(default)]
    pub items: HashMap<char, Value>,

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

impl CDDAPalette {
    pub fn calculate_parameters(
        &self,
        all_palettes: &Palettes,
    ) -> IndexMap<ParameterIdentifier, CDDAIdentifier> {
        let mut calculated_parameters: IndexMap<ParameterIdentifier, CDDAIdentifier> =
            IndexMap::new();

        for (id, parameter) in self.parameters.iter() {
            calculated_parameters.insert(
                id.clone(),
                parameter.default.distribution.get(&calculated_parameters),
            );
        }

        for mapgen_value in self.palettes.iter() {
            let id = mapgen_value.get_identifier(&calculated_parameters);

            all_palettes
                .get(&id)
                .expect("Palette to exist")
                .calculate_parameters(all_palettes)
                .into_iter()
                .for_each(|(child_id, child_param)| {
                    calculated_parameters.insert(child_id, child_param);
                })
        }

        calculated_parameters
    }

    pub fn get_mapping(
        &self,
        mapping_kind: &Mapping,
        character: &char,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        all_palettes: &Palettes,
    ) -> Option<CDDAIdentifier> {
        match mapping_kind {
            Mapping::Terrain => {
                if let Some(id) = self.terrain.get(character) {
                    return Some(id.get_identifier(calculated_parameters));
                };
            }
            Mapping::Furniture => {
                if let Some(id) = self.furniture.get(character) {
                    return Some(id.get_identifier(calculated_parameters));
                };
            }
            _ => todo!(),
        }

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) =
                palette.get_mapping(mapping_kind, character, calculated_parameters, all_palettes)
            {
                return Some(id);
            }
        }

        None
    }
}
