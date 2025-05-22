use crate::cdda_data::{CDDADeleteOp, CDDAExtendOp, CDDAString, IdOrAbstract};
use crate::impl_merge_with_precedence;
use crate::util::{CDDAIdentifier, MeabyVec};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CDDATerrainIntermediate {
    #[serde(flatten)]
    pub identifier: IdOrAbstract<CDDAIdentifier>,
    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub name: Option<CDDAString>,
    pub description: Option<CDDAString>,
    pub symbol: Option<char>,
    pub looks_like: Option<CDDAIdentifier>,
    pub color: Option<MeabyVec<String>>,
    pub connect_groups: Option<MeabyVec<CDDAIdentifier>>,
    pub connects_to: Option<MeabyVec<CDDAIdentifier>>,
    pub flags: Option<Vec<String>>,
    pub extend: Option<CDDAExtendOp>,
    pub delete: Option<CDDADeleteOp>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDATerrain {
    pub id: CDDAIdentifier,
    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub is_abstract: bool,
    pub name: Option<CDDAString>,
    pub description: Option<CDDAString>,
    pub symbol: Option<char>,
    pub looks_like: Option<CDDAIdentifier>,
    pub color: Option<MeabyVec<String>>,
    pub connect_groups: Option<MeabyVec<CDDAIdentifier>>,
    pub connects_to: Option<MeabyVec<CDDAIdentifier>>,
    pub flags: Option<Vec<String>>,
    pub extend: Option<CDDAExtendOp>,
    pub delete: Option<CDDADeleteOp>,
}

impl_merge_with_precedence!(
    CDDATerrain,
    id, is_abstract;
    copy_from, name, description, symbol, looks_like, color, connect_groups, connects_to, flags, extend, delete
);

impl Into<CDDATerrain> for CDDATerrainIntermediate {
    fn into(self) -> CDDATerrain {
        let (id, is_abstract) = match self.identifier {
            IdOrAbstract::Id(id) => (id, false),
            IdOrAbstract::Abstract(abs) => (abs, true),
        };

        CDDATerrain {
            id,
            copy_from: self.copy_from,
            is_abstract,
            name: self.name,
            description: self.description,
            symbol: self.symbol,
            looks_like: self.looks_like,
            color: self.color,
            connect_groups: self.connect_groups,
            connects_to: self.connects_to,
            flags: self.flags,
            extend: self.extend,
            delete: self.delete,
        }
    }
}
