use cdda_lib::types::CDDAIdentifier;
use cdda_lib::types::CDDAString;
use cdda_macros::cdda_entry;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Default, EnumString, Clone, Serialize, Deserialize)]
pub enum Location {
    OnWindshield,
    OnCeiling,
    Roof,
    Internal,
    Armor,
    Anywhere,
    Axle,
    EngineBlock,
    FuelSource,
    OnBatteryMount,

    // Any fields above are never shown
    // First
    #[default]
    #[strum(serialize = "structure")]
    Structure,
    // Second
    #[strum(serialize = "under")]
    Under,
    // Third
    #[strum(serialize = "center")]
    Center,
    // Fourth
    #[strum(serialize = "on_roof")]
    OnRoof,
}

impl Location {
    pub fn priority(&self) -> usize {
        match self {
            Location::Structure => 1,
            Location::Under => 2,
            Location::Center => 3,
            Location::OnRoof => 4,
            _ => 0,
        }
    }
}

#[cdda_entry]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAVehiclePart {
    pub id: CDDAIdentifier,
    pub looks_like: Option<CDDAIdentifier>,
    pub name: Option<CDDAString>,
    pub flags: Vec<String>,
    #[serde(default)]
    pub location: Option<String>,
}
