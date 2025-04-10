pub(crate) mod furniture;
pub(crate) mod io;
pub(crate) mod map_data;
pub(crate) mod palettes;
pub(crate) mod region_settings;
pub(crate) mod terrain;

use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::map_data::CDDAMapData;
use crate::cdda_data::palettes::CDDAPalette;
use crate::cdda_data::region_settings::CDDARegionSettings;
use crate::cdda_data::terrain::CDDATerrain;
use crate::util::{
    CDDAIdentifier, GetIdentifier, MeabyParam, MeabyVec, MeabyWeighted, ParameterIdentifier,
};
use derive_more::Display;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub fn extract_comments<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut comments = Vec::new();

    let map: BTreeMap<String, Value> = Deserialize::deserialize(deserializer)?;

    for (key, value) in &map {
        if key.starts_with("//") {
            if let Some(comment) = value.as_str() {
                comments.push(comment.to_string());
            }
        }
    }

    Ok(comments)
}

#[derive(Debug, Clone, Deserialize)]
pub enum IdOrAbstract {
    #[serde(rename = "id")]
    Id(CDDAIdentifier),
    #[serde(rename = "abstract")]
    Abstract(CDDAIdentifier),
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnknownEntry {
    #[serde(flatten)]
    identifier: IdOrAbstract,

    #[serde(rename = "type")]
    ty: CataVariant,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CDDAJsonEntry {
    Mapgen(CDDAMapData),
    RegionSettings(CDDARegionSettings),
    Palette(CDDAPalette),
    Terrain(CDDATerrain),
    Furniture(CDDAFurniture),
}

#[derive(Debug, Clone, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CataVariant {
    OvermapSpecialId,
    Palette,
    RegionSettings,
    Mapgen,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Switch {
    param: ParameterIdentifier,
    fallback: CDDAIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub distribution: MeabyVec<MeabyWeighted<CDDAIdentifier>>,
}

// https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/JSON/MAPGEN.md#mapgen-values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapGenValue {
    String(CDDAIdentifier),
    Param {
        param: ParameterIdentifier,
        fallback: Option<CDDAIdentifier>,
    },
    Switch {
        switch: Switch,
        cases: HashMap<CDDAIdentifier, CDDAIdentifier>,
    },
    Distribution(MeabyVec<MeabyWeighted<MeabyParam>>),
}

impl GetIdentifier for MapGenValue {
    fn get_identifier(
        &self,
        calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        match self {
            MapGenValue::String(s) => s.clone(),
            MapGenValue::Distribution(d) => d.get(calculated_parameters),
            MapGenValue::Param { param, fallback } => calculated_parameters
                .get(param)
                .map(|p| p.clone())
                .unwrap_or_else(|| fallback.clone().expect("Fallback to exist")),
            MapGenValue::Switch { switch, cases } => {
                let id = calculated_parameters
                    .get(&switch.param)
                    .map(|p| p.clone())
                    .unwrap_or_else(|| switch.fallback.clone());

                cases.get(&id).expect("MapTo to exist").clone()
            }
        }
    }
}
