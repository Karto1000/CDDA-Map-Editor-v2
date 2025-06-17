use std::collections::HashMap;
use glam::UVec2;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use crate::features::program_data::{MapDataCollection, ZLevel};

mod data;
pub mod handler;
mod io;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapEditor {
    pub maps: HashMap<ZLevel, MapDataCollection>,
    pub size: UVec2,
}

#[derive(Debug, Clone)]
pub struct MapSize(UVec2);

impl<'de> Deserialize<'de> for MapSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let uvec2 = UVec2::deserialize(deserializer)?;
        let map_size =
            MapSize::new(uvec2.x, uvec2.y).map_err(serde::de::Error::custom)?;
        Ok(map_size)
    }
}

#[derive(Debug, Serialize, Error)]
pub enum MapSizeError {
    #[error(
        "Invalid map size: {0}. Map sizes over 24 tiles must be a multiple of 24"
    )]
    InvalidMultiple(u32),
}

impl MapSize {
    pub fn new(width: u32, height: u32) -> Result<Self, MapSizeError> {
        if width < 24 || height < 24 {
            return Ok(Self(UVec2::new(width, height)));
        }

        if width % 24 != 0 {
            return Err(MapSizeError::InvalidMultiple(width));
        }

        if height % 24 != 0 {
            return Err(MapSizeError::InvalidMultiple(height));
        }

        Ok(Self(UVec2::new(width, height)))
    }

    pub fn value(&self) -> UVec2 {
        self.0
    }
}
