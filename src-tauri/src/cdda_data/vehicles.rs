use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::vehicle_parts::CDDAVehiclePart;
use crate::impl_merge_with_precedence;
use cdda_lib::types::CDDAIdentifier;
use cdda_lib::types::{CDDAString, IdOrAbstract};
use log::warn;
use serde::{Deserialize, Serialize};

fn part_and_variant_from_ident(
    cdda_ident: CDDAIdentifier,
) -> (CDDAIdentifier, Option<VehiclePartVariant>) {
    let (part, variant) = match cdda_ident.0.split_once("#") {
        None => return (cdda_ident, None),
        Some((part, variant)) => (part, variant),
    };

    let variant = match serde_json::from_str::<VehiclePartVariant>(variant) {
        Ok(v) => v,
        Err(e) => {
            warn!("{}", e);
            return (cdda_ident, None);
        },
    };

    (part.into(), Some(variant))
}

// What is the part of the vehicle at x, y made of?
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum VehiclePartCompositionIntermediate {
    Inline(CDDAIdentifier),
    Object {
        part: CDDAIdentifier,
        // TODO: Add other fields
    },
}

impl Into<VehiclePartComposition> for VehiclePartCompositionIntermediate {
    fn into(self) -> VehiclePartComposition {
        match self {
            VehiclePartCompositionIntermediate::Inline(part) => {
                VehiclePartComposition {
                    part,
                    variant: None,
                }
            },
            VehiclePartCompositionIntermediate::Object { part } => {
                VehiclePartComposition {
                    part,
                    variant: None,
                }
            },
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct VehiclePartPlacementIntermediate {
    pub x: i32,
    pub y: i32,
    pub parts: Vec<VehiclePartCompositionIntermediate>,
}

impl Into<VehiclePartPlacement> for VehiclePartPlacementIntermediate {
    fn into(self) -> VehiclePartPlacement {
        VehiclePartPlacement {
            x: self.x,
            y: self.y,
            parts: self.parts.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CDDAVehicleIntermediate {
    #[serde(flatten)]
    pub identifier: IdOrAbstract<CDDAIdentifier>,

    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,

    pub name: Option<CDDAString>,
    pub parts: Vec<VehiclePartPlacementIntermediate>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VehiclePartVariant {
    Cross,
    Vertical2,
    Vertical,
    Pedal,
    Read,
    BikeRead,
    Motor,
    Horizontal,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VehiclePartComposition {
    pub part: CDDAIdentifier,
    pub variant: Option<VehiclePartVariant>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct VehiclePartPlacement {
    pub x: i32,
    pub y: i32,
    pub parts: Vec<VehiclePartComposition>,
}

#[derive(Default, Serialize, Clone)]
pub struct CDDAVehicle {
    pub id: CDDAIdentifier,

    pub is_abstract: bool,

    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,

    pub name: Option<CDDAString>,
    pub parts: Vec<VehiclePartPlacement>,
}

impl CDDAVehicle {
    pub fn calculate_copy(
        &self,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> CDDAVehicle {
        match &self.copy_from {
            None => self.clone(),
            Some(copy_from_id) => {
                let mut copy_from_special =
                    match cdda_data.vehicles.get(copy_from_id) {
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

                CDDAVehicle::merge_with_precedence(&copy_from_special, self)
            },
        }
    }
}

impl_merge_with_precedence!(
    CDDAVehicle,
    id,
    is_abstract,
    copy_from,
    parts
    ;
    name
);

impl Into<CDDAVehicle> for CDDAVehicleIntermediate {
    fn into(self) -> CDDAVehicle {
        let (id, is_abstract) = match self.identifier {
            IdOrAbstract::Id(id) => (id, false),
            IdOrAbstract::Abstract(abs) => (abs, true),
        };

        CDDAVehicle {
            id,
            is_abstract,
            copy_from: self.copy_from,
            name: self.name,
            parts: self.parts.into_iter().map(Into::into).collect(),
        }
    }
}
