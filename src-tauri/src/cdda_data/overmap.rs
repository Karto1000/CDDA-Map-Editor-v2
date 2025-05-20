use crate::cdda_data::{CDDADeleteOp, CDDAExtendOp, CDDAString, IdOrAbstract};
use crate::impl_merge_with_precedence;
use crate::util::{CDDAIdentifier, MeabyVec};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAOvermapLocation {
    pub id: CDDAIdentifier,

    #[serde(default)]
    pub terrains: HashSet<CDDAIdentifier>,

    #[serde(default)]
    pub flags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OvermapTerrainMapgenMethod {
    Builtin,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OvermapTerrainMapgen {
    pub method: OvermapTerrainMapgenMethod,
    pub name: CDDAIdentifier,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAOvermapTerrainIntermediate {
    #[serde(flatten)]
    pub identifier: IdOrAbstract<MeabyVec<CDDAIdentifier>>,
    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub flags: Option<Vec<String>>,
    pub extend: Option<CDDAExtendOp>,
    pub delete: Option<CDDADeleteOp>,

    pub name: Option<CDDAString>,

    #[serde(rename = "sym")]
    pub symbol: Option<char>,

    pub color: Option<String>,

    pub mapgen: Option<Vec<OvermapTerrainMapgen>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAOvermapTerrain {
    #[serde(flatten)]
    pub id: CDDAIdentifier,
    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub is_abstract: bool,
    pub name: Option<CDDAString>,
    pub symbol: Option<char>,
    pub flags: Option<Vec<String>>,
    pub extend: Option<CDDAExtendOp>,
    pub delete: Option<CDDADeleteOp>,
    pub mapgen: Option<Vec<OvermapTerrainMapgen>>,
}

impl_merge_with_precedence!(
    CDDAOvermapTerrain,
    id,
    is_abstract
    ;
    copy_from,
    name,
    symbol,
    flags,
    extend,
    delete,
    mapgen
);

impl Into<Vec<CDDAOvermapTerrain>> for CDDAOvermapTerrainIntermediate {
    fn into(self) -> Vec<CDDAOvermapTerrain> {
        let (ids, is_abstract) = match self.identifier {
            IdOrAbstract::Id(id) => (id.into_vec(), false),
            IdOrAbstract::Abstract(abs) => (vec![abs], true),
        };

        let mut terrain_list = vec![];

        for id in ids {
            terrain_list.push(CDDAOvermapTerrain {
                id,
                copy_from: self.copy_from.clone(),
                is_abstract,
                mapgen: self.mapgen.clone(),
                name: self.name.clone(),
                symbol: self.symbol.clone(),
                flags: self.flags.clone(),
                extend: self.extend.clone(),
                delete: self.delete.clone(),
            })
        }

        terrain_list
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAOvermapSpecialIntermediate {}
