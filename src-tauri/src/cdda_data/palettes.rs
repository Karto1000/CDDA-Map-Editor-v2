use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::map_data::{MapGenItem, MapGenMonster};
use crate::cdda_data::{Distribution, KnownCataVariant, MapGenValue};
use crate::map::representative_properties::ItemProperty;
use crate::map::visible_properties::{FurnitureProperty, MonsterProperty, TerrainProperty};
use crate::map::{RepresentativeMapping, RepresentativeProperty, VisibleMapping, VisibleProperty};
use crate::util::{CDDAIdentifier, Comment, GetIdentifier, MeabyVec, ParameterIdentifier};
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

impl Into<CDDAPalette> for CDDAPaletteIntermediate {
    fn into(self) -> CDDAPalette {
        let mut visible = HashMap::new();

        let mut terrain_map = HashMap::new();
        for (char, terrain) in self.terrain {
            let ter_prop = Arc::new(TerrainProperty {
                mapgen_value: terrain,
            });

            terrain_map.insert(char, ter_prop as Arc<dyn VisibleProperty>);
        }

        let mut furniture_map = HashMap::new();
        for (char, furniture) in self.furniture {
            let fur_prop = Arc::new(FurnitureProperty {
                mapgen_value: furniture,
            });

            furniture_map.insert(char, fur_prop as Arc<dyn VisibleProperty>);
        }

        let mut monster_map = HashMap::new();
        for (char, monster) in self.monster {
            let monster_prop = Arc::new(MonsterProperty { monster });

            monster_map.insert(char, monster_prop as Arc<dyn VisibleProperty>);
        }

        visible.insert(VisibleMapping::Terrain, terrain_map);
        visible.insert(VisibleMapping::Furniture, furniture_map);
        visible.insert(VisibleMapping::Monster, monster_map);

        let mut representative = HashMap::new();

        let mut item_map = HashMap::new();
        for (char, items) in self.items {
            let item_prop = Arc::new(ItemProperty {
                items: items.into_vec(),
            });
            item_map.insert(char, item_prop as Arc<dyn RepresentativeProperty>);
        }

        representative.insert(RepresentativeMapping::ItemGroups, item_map);

        CDDAPalette {
            id: self.id,
            visible,
            representative,
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
    pub visible: HashMap<VisibleMapping, HashMap<char, Arc<dyn VisibleProperty>>>,

    #[serde(skip)]
    pub representative:
        HashMap<RepresentativeMapping, HashMap<char, Arc<dyn RepresentativeProperty>>>,

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

    pub fn get_visible_mapping(
        &self,
        mapping_kind: impl Borrow<VisibleMapping>,
        character: impl Borrow<char>,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<CDDAIdentifier> {
        let mapping = self.visible.get(mapping_kind.borrow())?;

        if let Some(id) = mapping.get(character.borrow()) {
            return id.get_identifier(calculated_parameters, json_data);
        }

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
            let palette = json_data.palettes.get(&palette_id)?;

            if let Some(id) = palette.get_visible_mapping(
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

    pub fn get_representative_mapping(
        &self,
        mapping_kind: impl Borrow<RepresentativeMapping>,
        character: impl Borrow<char>,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Value> {
        let mapping = self.representative.get(mapping_kind.borrow())?;

        match mapping.get(character.borrow()) {
            None => {}
            Some(s) => return Some(s.representation(json_data)),
        }

        for mapgen_value in self.palettes.iter() {
            let palette_id = mapgen_value.get_identifier(calculated_parameters);
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
