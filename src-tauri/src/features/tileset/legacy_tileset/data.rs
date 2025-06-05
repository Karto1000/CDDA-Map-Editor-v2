use crate::features::tileset::data::AdditionalTileType;
use crate::features::tileset::legacy_tileset::SpriteIndex;
use cdda_lib::types::{CDDAIdentifier, MeabyVec, MeabyWeighted};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_range_comment<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<(u32, u32), D::Error> {
    let s = String::deserialize(deserializer)?;

    let (mut left, mut right) = s
        .split_once(" to ")
        .ok_or(Error::custom("Failed to split comment at ' to '"))?;

    right = right.trim();
    left = left
        .strip_prefix("range ")
        .ok_or_else(|| Error::custom("Failed to strip 'range ' from prefix"))?
        .trim();

    let mut from = left
        .parse()
        .map_err(|e| Error::custom("Failed to parse range start"))?;

    // TODO: Special case for the first entry of the first spritesheet. This is done to fix the
    // Off by one error when rendering sprites of the first spritesheet. Probably a better way to do
    // this
    if from == 1 {
        from = 0
    }

    let to = right
        .parse()
        .map_err(|e| Error::custom("Failed to parse range end"))?;

    Ok((from, to))
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct LegacyTileConfig {
    pub tile_info: Vec<TileInfo>,

    #[serde(rename = "tiles-new")]
    pub spritesheets: Vec<Spritesheet>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(super) enum Spritesheet {
    Normal(NormalSpritesheet),
    Fallback(FallbackSpritesheet),
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct NormalSpritesheet {
    pub file: String,

    pub sprite_width: Option<u32>,
    pub sprite_height: Option<u32>,
    pub sprite_offset_x: Option<i32>,
    pub sprite_offset_y: Option<i32>,

    #[serde(deserialize_with = "deserialize_range_comment", rename = "//")]
    pub range: (u32, u32),

    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct AdditionalTile {
    pub id: AdditionalTileType,
    pub rotates: Option<bool>,
    pub animated: Option<bool>,
    pub fg: Option<MeabyVec<MeabyWeighted<MeabyVec<SpriteIndex>>>>,
    pub bg: Option<MeabyVec<MeabyWeighted<MeabyVec<SpriteIndex>>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct Tile {
    pub id: MeabyVec<CDDAIdentifier>,
    pub fg: Option<MeabyVec<MeabyWeighted<MeabyVec<SpriteIndex>>>>,
    pub bg: Option<MeabyVec<MeabyWeighted<MeabyVec<SpriteIndex>>>>,
    pub rotates: Option<bool>,
    pub animated: Option<bool>,
    pub multitile: Option<bool>,
    pub additional_tiles: Option<Vec<AdditionalTile>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct FallbackSpritesheet {
    pub file: String,

    // TODO: Idk what this is for
    pub tiles: Vec<()>,

    pub ascii: Vec<AsciiCharGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct AsciiCharGroup {
    pub offset: i32,
    pub bold: bool,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct TileInfo {
    pub pixelscale: u32,
    pub width: u32,
    pub height: u32,
    pub zlevel_height: u32,
    pub iso: bool,
    pub retract_dist_min: f32,
    pub retract_dist_max: f32,
}
