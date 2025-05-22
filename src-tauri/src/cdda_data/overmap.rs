use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::impl_merge_with_precedence;
use cdda_lib::types::{
    CDDADeleteOp, CDDAExtendOp, CDDAIdentifier, CDDAString, IdOrAbstract,
    MeabyVec,
};
use glam::IVec3;
use log::warn;
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

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAOvermapSpecialIntermediate {
    #[serde(flatten)]
    pub identifier: IdOrAbstract<CDDAIdentifier>,
    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub flags: Option<Vec<String>>,
    pub extend: Option<CDDAExtendOp>,
    pub delete: Option<CDDADeleteOp>,

    #[serde(flatten)]
    pub ty: OvermapSpecialSubType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAOvermapSpecial {
    pub id: CDDAIdentifier,

    #[serde(flatten)]
    pub ty: OvermapSpecialSubType,

    pub is_abstract: bool,

    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub flags: Option<Vec<String>>,
    pub extend: Option<CDDAExtendOp>,
    pub delete: Option<CDDADeleteOp>,
}

impl CDDAOvermapSpecial {
    pub fn calculate_copy(
        &self,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> CDDAOvermapSpecial {
        match &self.copy_from {
            None => self.clone(),
            Some(copy_from_id) => {
                let mut copy_from_special =
                    match cdda_data.overmap_specials.get(copy_from_id) {
                        None => {
                            warn!(
                            "Could not copy {} for {} due to it not existing",
                            copy_from_id, self.id
                        );
                            return self.clone();
                        },
                        Some(t) => t.clone(),
                    };

                if copy_from_special.copy_from.is_some() {
                    copy_from_special = self.calculate_copy(cdda_data);
                }

                CDDAOvermapSpecial::merge_with_precedence(
                    &copy_from_special,
                    self,
                )
            },
        }
    }
}

impl_merge_with_precedence!(
    CDDAOvermapSpecial,
    id,
    is_abstract,
    ty
    ;
    copy_from,
    flags,
    extend,
    delete
);

impl Into<CDDAOvermapSpecial> for CDDAOvermapSpecialIntermediate {
    fn into(self) -> CDDAOvermapSpecial {
        let (id, is_abstract) = match self.identifier {
            IdOrAbstract::Id(id) => (id, false),
            IdOrAbstract::Abstract(abs) => (abs, true),
        };

        CDDAOvermapSpecial {
            id,
            copy_from: self.copy_from,
            is_abstract,
            ty: self.ty,
            flags: self.flags,
            extend: self.extend,
            delete: self.delete,
        }
    }
}
