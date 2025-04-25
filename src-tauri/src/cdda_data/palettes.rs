use crate::cdda_data::map_data::{MapGenItem, MapGenMonster, MapGenMonsterType};
use crate::cdda_data::monster::CDDAMonsterGroup;
use crate::cdda_data::{Distribution, KnownCataVariant, MapGenValue, NumberOrRange};
use crate::map::VisibleMapping;
use crate::util::{CDDAIdentifier, Comment, GetIdentifier, MeabyVec, ParameterIdentifier};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    pub monster: HashMap<char, MapGenMonster>,

    #[serde(default)]
    pub monsters: HashMap<char, Value>,

    #[serde(default)]
    pub npcs: HashMap<char, Value>,

    #[serde(default)]
    pub items: HashMap<char, MeabyVec<MapGenItem>>,

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

    pub fn get_items(
        &self,
        character: &char,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
    ) -> Option<Vec<MapGenItem>> {
        if let Some(items) = self.items.get(character) {
            return Some(items.clone().into_vec());
        }

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) = palette.get_items(character, calculated_parameters, all_palettes) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_monster(
        &self,
        character: &char,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        all_palettes: &HashMap<CDDAIdentifier, CDDAPalette>,
        monstergroups: &HashMap<CDDAIdentifier, CDDAMonsterGroup>,
    ) -> Option<CDDAIdentifier> {
        if let Some(mon) = self.monster.get(character) {
            return match mon
                .chance
                .clone()
                .unwrap_or(NumberOrRange::Number(1))
                // TODO: This is spawning wayyy to many monsters
                .is_random_hit(100)
            {
                true => match &mon.id {
                    MapGenMonsterType::Monster { monster } => {
                        Some(monster.get_identifier(calculated_parameters))
                    }
                    MapGenMonsterType::MonsterGroup { group } => {
                        let mon_group = monstergroups.get(group)?;
                        mon_group
                            .get_random_monster(monstergroups)
                            .map(|id| id.get_identifier(calculated_parameters))
                    }
                },
                false => None,
            };
        };

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
            let palette = all_palettes.get(&palette_id).expect("Palette to exist");

            if let Some(id) = palette.get_monster(
                character,
                calculated_parameters,
                all_palettes,
                monstergroups,
            ) {
                return Some(id);
            }
        }

        None
    }

    pub fn get_visible_mapping(
        &self,
        visible_mapping: &VisibleMapping,
        character: &char,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        all_palettes: &Palettes,
    ) -> Option<CDDAIdentifier> {
        match visible_mapping {
            VisibleMapping::Terrain => {
                if let Some(id) = self.terrain.get(character) {
                    return Some(id.get_identifier(calculated_parameters));
                };
            }
            VisibleMapping::Furniture => {
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

            if let Some(id) = palette.get_visible_mapping(
                visible_mapping,
                character,
                calculated_parameters,
                all_palettes,
            ) {
                return Some(id);
            }
        }

        None
    }
}
