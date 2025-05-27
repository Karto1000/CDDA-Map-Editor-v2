use cdda_lib::types::CDDAIdentifier;
use cdda_lib::types::CDDAString;
use cdda_macros::cdda_entry;
use serde::{Deserialize, Serialize};

// What is the part of the vehicle at x, y made of?
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum VehiclePart {
    Inline(CDDAIdentifier),
    Object {
        part: CDDAIdentifier,
        // TODO: Add other fields
    },
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct VehiclePartPlacement {
    pub x: i32,
    pub y: i32,
    pub parts: Vec<VehiclePart>,
}

#[cdda_entry]
#[derive(Default, Debug, Serialize, Clone)]
pub struct CDDAVehicle {
    pub id: CDDAIdentifier,
    pub name: Option<CDDAString>,
    pub parts: Vec<VehiclePartPlacement>,
    pub flags: Vec<String>,
}
