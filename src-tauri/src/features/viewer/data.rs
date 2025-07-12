use crate::features::program_data::ProjectType;
use glam::UVec2;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct CreateMapData {
    name: String,
    size: UVec2,
    ty: ProjectType,
}
