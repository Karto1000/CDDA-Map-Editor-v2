use cdda_lib::types::{CDDAIdentifier, CDDAString, MeabyVec};
use cdda_macros::cdda_entry;
use serde::{Deserialize, Serialize};

#[cdda_entry]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDATerrain {
    pub id: CDDAIdentifier,
    pub name: Option<CDDAString>,
    pub description: Option<CDDAString>,
    pub symbol: Option<char>,
    pub looks_like: Option<CDDAIdentifier>,
    pub color: Option<MeabyVec<String>>,
    pub connect_groups: Option<MeabyVec<CDDAIdentifier>>,
    pub connects_to: Option<MeabyVec<CDDAIdentifier>>,
    pub flags: Vec<String>,
}
