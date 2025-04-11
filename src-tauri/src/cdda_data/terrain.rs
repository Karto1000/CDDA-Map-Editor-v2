use crate::cdda_data::{CDDAString, ConnectGroup};
use crate::util::{CDDAIdentifier, MeabyVec};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CDDATerrain {
    pub id: CDDAIdentifier,
    pub name: Option<CDDAString>,
    pub description: Option<CDDAString>,
    pub symbol: Option<char>,
    pub looks_like: Option<CDDAIdentifier>,
    pub color: Option<MeabyVec<String>>,
    pub connect_groups: Option<MeabyVec<ConnectGroup>>,
    pub connects_to: Option<MeabyVec<ConnectGroup>>,
}
