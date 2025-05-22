use cdda_lib::types::CDDAIdentifier;
use cdda_lib::types::CDDAString;
use cdda_macros::cdda_entry;
use serde::{Deserialize, Serialize};

#[cdda_entry]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAVehiclePart {
    pub id: CDDAIdentifier,
    pub looks_like: Option<CDDAIdentifier>,
    pub name: Option<CDDAString>,
    pub flags: Vec<String>,
}
