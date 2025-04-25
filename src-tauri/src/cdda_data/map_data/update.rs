use crate::cdda_data::map_data::CDDAMapDataObjectCommonIntermediate;
use crate::util::CDDAIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAUpdateMapDataObjectIntermediate {
    pub rows: Option<Vec<String>>,

    #[serde(flatten)]
    pub common: CDDAMapDataObjectCommonIntermediate,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAUpdateMapDataIntermediate {
    pub method: String,

    pub update_mapgen_id: CDDAIdentifier,

    pub object: CDDAUpdateMapDataObjectIntermediate,
}
