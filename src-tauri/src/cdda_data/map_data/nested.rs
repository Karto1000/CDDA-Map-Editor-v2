use crate::cdda_data::map_data::CDDAMapDataObjectCommonIntermediate;
use crate::util::CDDAIdentifier;
use glam::UVec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDANestedMapDataObjectIntermediate {
    pub rows: Option<Vec<String>>,

    #[serde(rename = "mapgensize")]
    pub mapgen_size: UVec2,

    #[serde(flatten)]
    pub common: CDDAMapDataObjectCommonIntermediate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDANestedMapDataIntermediate {
    pub method: String,

    pub nested_mapgen_id: CDDAIdentifier,

    pub object: CDDANestedMapDataObjectIntermediate,
}
