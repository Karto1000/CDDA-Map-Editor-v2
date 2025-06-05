use cdda_lib::types::{CDDAIdentifier, CDDAString};
use cdda_macros::cdda_entry;
use glam::IVec3;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cdda_entry]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAOvermapLocation {
    pub id: CDDAIdentifier,

    #[serde(default)]
    pub terrains: HashSet<CDDAIdentifier>,

    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OvermapTerrainMapgenMethod {
    Builtin,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OvermapTerrainMapgen {
    pub builtin: CDDAIdentifier,
}

#[cdda_entry]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAOvermapTerrain {
    pub id: CDDAIdentifier,
    pub name: Option<CDDAString>,
    pub symbol: Option<char>,
    pub mapgen: Option<Vec<OvermapTerrainMapgen>>,
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvermapSpecialOvermap {
    pub point: IVec3,
    pub overmap: Option<CDDAIdentifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum OvermapSpecialSubType {
    Fixed {
        overmaps: Vec<OvermapSpecialOvermap>,
    },
    Mutable {
        subtype: String,
    },
}

#[cdda_entry]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAOvermapSpecial {
    pub id: CDDAIdentifier,
    #[serde(flatten)]
    pub ty: OvermapSpecialSubType,
    pub flags: Vec<String>,
}
