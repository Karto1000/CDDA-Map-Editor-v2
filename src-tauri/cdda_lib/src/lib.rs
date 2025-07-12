use comfy_bounded_ints::types::Bound_u32;
use glam::UVec2;
use serde_derive::{Deserialize, Serialize};
use std::ops::{Add, Deref};
use std::string::ToString;

pub mod serde_vec;
pub mod types;

pub const TERRAIN_PREFIX: &'static str = "t_";
pub const FURNITURE_PREFIX: &'static str = "f_";
pub const REGION_SETTING_PREFIX: &'static str = "t_region";
pub const NULL_TERRAIN: &'static str = "t_null";
pub const NULL_FURNITURE: &'static str = "f_null";
pub const NULL_NESTED: &'static str = "null";
pub const NULL_FIELD: &'static str = "fd_null";
pub const NULL_TRAP: &'static str = "tr_null";
pub const DEFAULT_REGION_SETTING_ENTRY: &'static str = "default";
pub const MAX_MAPGEN_WIDTH: u32 = 24;
pub const MIN_MAPGEN_WIDTH: u32 = 0;
pub const MAX_MAPGEN_HEIGHT: u32 = 24;
pub const MIN_MAPGEN_HEIGHT: u32 = 0;
pub const DEFAULT_CELL_CHARACTER: char = ' ';
pub const DEFAULT_EMPTY_CHAR_ROW: &'static str = "                        ";
pub const DEFAULT_MAP_ROWS: [&'static str; 24] = [DEFAULT_EMPTY_CHAR_ROW; 24];

/// Overmap coordinates are coordinates that are used to compose multiple mapgens into one overmap special
pub type OvermapCoordinates = UVec2;

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash, Default,
)]
pub struct MapgenCellCoordinates(
    #[serde(
        serialize_with = "crate::serde_vec::serialize_uvec2_as_key",
        deserialize_with = "crate::serde_vec::deserialize_uvec2_as_key"
    )]
    UVec2,
);

impl Add for MapgenCellCoordinates {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Deref for MapgenCellCoordinates {
    type Target = UVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<UVec2> for MapgenCellCoordinates {
    fn from(value: UVec2) -> Self {
        let x = Bound_u32::<MIN_MAPGEN_WIDTH, MAX_MAPGEN_WIDTH>::new(value.x);
        let y = Bound_u32::<MIN_MAPGEN_HEIGHT, MAX_MAPGEN_HEIGHT>::new(value.y);
        Self(UVec2::new(x.get(), y.get()))
    }
}

impl MapgenCellCoordinates {
    pub fn new(
        width: Bound_u32<MIN_MAPGEN_WIDTH, MAX_MAPGEN_HEIGHT>,
        height: Bound_u32<MIN_MAPGEN_WIDTH, MAX_MAPGEN_HEIGHT>,
    ) -> Self {
        Self(UVec2::new(width.get(), height.get()))
    }

    pub fn uvec(self) -> UVec2 {
        self.0
    }
}
