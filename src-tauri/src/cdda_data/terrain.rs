use crate::util::{CDDAIdentifier, MeabyVec};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CDDATerrain {
    pub id: CDDAIdentifier,
    pub name: Option<String>,
    pub description: Option<String>,
    pub symbol: char,
    pub looks_like: Option<CDDAIdentifier>,
    pub color: MeabyVec<String>,
    pub connect_groups: Option<MeabyVec<String>>,
    pub connects_to: Option<MeabyVec<String>>,
}
