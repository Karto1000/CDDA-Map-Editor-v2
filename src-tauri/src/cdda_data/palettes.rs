use crate::cdda_data::{CataVariant, Distribution, MapGenValue};
use crate::util::{CDDAIdentifier, Comment, GetIdentifier, ParameterIdentifier};
use indexmap::IndexMap;
use serde::Deserialize;
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
    pub furniture: HashMap<char, MapGenValue>,

    #[serde(default)]
    pub terrain: HashMap<char, MapGenValue>,
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

    pub fn get_terrain(
        &self,
        character: &char,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        all_palettes: &Palettes,
    ) -> Option<CDDAIdentifier> {
        if let Some(id) = self.terrain.get(character) {
            return Some(id.get_identifier(calculated_parameters));
        };

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) = palette.get_terrain(character, calculated_parameters, all_palettes) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_furniture(
        &self,
        character: &char,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        all_palettes: &Palettes,
    ) -> Option<CDDAIdentifier> {
        if let Some(id) = self.furniture.get(character) {
            return Some(id.get_identifier(calculated_parameters));
        };

        // If we don't find it, search the palettes from top to bottom
        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) = palette.get_furniture(character, calculated_parameters, all_palettes)
            {
                return Some(id);
            }
        }

        None
    }
}
