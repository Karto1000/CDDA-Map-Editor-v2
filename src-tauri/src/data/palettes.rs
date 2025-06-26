use crate::data::io::DeserializedCDDAJsonData;
use crate::data::map_data::{MapGenItem, MapGenMonsters};
use crate::data::GetIdentifier;
use crate::data::KnownCataVariant;
use crate::features::map::map_properties::{
    FurnitureProperty, MonstersProperty, TerrainProperty,
};
use crate::features::map::{
    CalculateParametersError, MapData, MappingKind, Property, SetTile,
};
use cdda_lib::types::{
    CDDADistributionInner, CDDAIdentifier, Comment, Distribution, MapGenValue,
    MeabyVec, MeabyWeighted, ParameterIdentifier, Switch,
};
use futures_lite::StreamExt;
use glam::IVec2;
use indexmap::IndexMap;
use log::{info, warn};
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::path::PathBuf;
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

    #[serde(skip)]
    pub source: Option<PathBuf>,

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

        properties.insert(MappingKind::Terrain, terrain_map);
        properties.insert(MappingKind::Furniture, furniture_map);
        properties.insert(MappingKind::Monsters, monster_map);

        CDDAPalette {
            id: self.id,
            properties,
            comment: self.comment,
            parameters: self.parameters,
            palettes: self.palettes,
            source: self.source,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAPalette {
    pub id: CDDAIdentifier,
    pub source: Option<PathBuf>,

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
        rng: &mut impl Rng,
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
                    .get_random_identifier(rng, &calculated_parameters)?,
            );
        }

        for mapgen_value in self.palettes.iter() {
            let id = mapgen_value
                .get_random_identifier(rng, &calculated_parameters)?;

            all_palettes
                .get(&id)
                .ok_or(CalculateParametersError::MissingPalette(id.0))?
                .calculate_parameters(rng, all_palettes)?
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
                .get_random_identifier(
                    &mut rng(),
                    &map_data.calculated_parameters,
                )
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

    fn get_path_from_ident(
        &self,
        id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<PathBuf> {
        match json_data.palettes.get(&id) {
            None => vec![],
            Some(p) => p.get_source(json_data),
        }
    }

    fn param_source(
        &self,
        param: &ParameterIdentifier,
        fallback: &Option<CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<PathBuf> {
        let parameter = match self.parameters.get(param) {
            None => {
                warn!("Parameter {} not found", param);
                return vec![];
            },
            Some(p) => p,
        };

        match parameter.ty {
            KnownCataVariant::Palette => {},
            _ => {
                warn!("Parameter {} is not a palette", param);
                return vec![];
            },
        }

        let mut source = vec![];

        match fallback {
            None => {},
            Some(f) => {
                source.append(&mut self.get_path_from_ident(f, json_data))
            },
        }

        let values = parameter.default.distribution.clone().into_values();

        for value in values {
            source.append(&mut self.get_path_from_ident(&value, json_data));
        }

        source
    }

    fn switch_source(
        &self,
        switch: &Switch,
        cases: &HashMap<CDDAIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<PathBuf> {
        let mut source = vec![];

        match json_data.palettes.get(&switch.fallback) {
            None => {},
            Some(p) => source.append(&mut p.get_source(json_data)),
        };

        for (_, case) in cases {
            match json_data.palettes.get(&case) {
                None => {},
                Some(p) => source.append(&mut p.get_source(json_data)),
            };
        }

        source
    }

    pub fn get_source(
        &self,
        json_data: &DeserializedCDDAJsonData,
    ) -> Vec<PathBuf> {
        let mut source = vec![];

        if let Some(source_path) = &self.source {
            source.push(source_path.clone());
        }

        for palette in self.palettes.iter() {
            let refs = palette.get_cdda_entry_refs();

            for palette in refs {
                let palette = match json_data.palettes.get(&palette) {
                    None => {
                        info!(
                            "Palette {} not found as ref in {}",
                            palette, self.id
                        );
                        continue;
                    },
                    Some(p) => p,
                };

                source.append(&mut palette.get_source(json_data));
            }
        }

        source
    }
}
