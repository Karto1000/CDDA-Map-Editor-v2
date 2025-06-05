use crate::features::tileset::data::FALLBACK_TILE_MAPPING;
use crate::features::tileset::legacy_tileset::data::{
    FallbackSpritesheet, TileInfo,
};
use crate::features::tileset::legacy_tileset::LegacyTilesheet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const FALLBACK_TILESHEET_CONFIG: &'static [u8] =
    include_bytes!("tile_config.json");
pub const FALLBACK_TILESHEET_IMAGE: &'static [u8] =
    include_bytes!("fallback.png");

#[derive(Debug, Serialize, Deserialize)]
pub struct FallbackTileConfig {
    pub tile_info: Vec<TileInfo>,

    #[serde(rename = "tiles-new")]
    pub spritesheets: Vec<FallbackSpritesheet>,
}

pub fn get_fallback_config() -> FallbackTileConfig {
    serde_json::from_slice(FALLBACK_TILESHEET_CONFIG).unwrap()
}

pub fn get_fallback_tilesheet() -> LegacyTilesheet {
    let mut config = get_fallback_config();
    let mut fallback_map = HashMap::new();

    let fallback_spritesheet = config
        .spritesheets
        .pop()
        .expect("Fallback spritesheet to exist");

    for ascii_group in fallback_spritesheet.ascii.into_iter() {
        for (character, offset) in FALLBACK_TILE_MAPPING {
            fallback_map.insert(
                format!("{}_{}", character, ascii_group.color),
                ascii_group.offset as u32 + offset,
            );
        }
    }

    LegacyTilesheet {
        id_map: HashMap::new(),
        fallback_map,
    }
}
