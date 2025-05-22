use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::overmap::CDDAOvermapSpecial;
use crate::cdda_data::{CDDAString, IdOrAbstract};
use crate::impl_merge_with_precedence;
use crate::util::CDDAIdentifier;
use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAVehiclePartIntermediate {
    #[serde(flatten)]
    pub identifier: IdOrAbstract<CDDAIdentifier>,

    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,
    pub looks_like: Option<CDDAIdentifier>,

    pub name: Option<CDDAString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDDAVehiclePart {
    pub id: CDDAIdentifier,

    pub is_abstract: bool,

    #[serde(rename = "copy-from")]
    pub copy_from: Option<CDDAIdentifier>,

    pub looks_like: Option<CDDAIdentifier>,
    pub name: Option<CDDAString>,
}

impl CDDAVehiclePart {
    pub fn calculate_copy(
        &self,
        cdda_data: &DeserializedCDDAJsonData,
    ) -> CDDAVehiclePart {
        match &self.copy_from {
            None => self.clone(),
            Some(copy_from_id) => {
                let mut copy_from_special =
                    match cdda_data.vehicle_parts.get(copy_from_id) {
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

                CDDAVehiclePart::merge_with_precedence(&copy_from_special, self)
            },
        }
    }
}

impl_merge_with_precedence!(
    CDDAVehiclePart,
    id,
    is_abstract,
    copy_from
    ;
    looks_like,
    name
);

impl Into<CDDAVehiclePart> for CDDAVehiclePartIntermediate {
    fn into(self) -> CDDAVehiclePart {
        let (id, is_abstract) = match self.identifier {
            IdOrAbstract::Id(id) => (id, false),
            IdOrAbstract::Abstract(abs) => (abs, true),
        };

        CDDAVehiclePart {
            id,
            is_abstract,
            copy_from: self.copy_from,
            looks_like: self.looks_like,
            name: self.name,
        }
    }
}
