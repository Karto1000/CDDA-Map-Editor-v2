use crate::tileset::io::TileConfigLoader;
use crate::tileset::legacy_tileset::SpriteIndex;
use crate::tileset::MeabyWeightedSprite;
use crate::util::Load;
use cdda_lib::types::{CDDAIdentifier, MeabyVec};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use std::fs::File;
use std::io::BufReader;

pub fn deserialize_range_comment<'de, D: Deserializer<'de>>(
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

impl Load<LegacyTileConfig> for TileConfigLoader {
    async fn load(&mut self) -> Result<LegacyTileConfig, anyhow::Error> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| anyhow::anyhow!(e))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LegacyTileConfig {
    pub tile_info: Vec<TileInfo>,

    #[serde(rename = "tiles-new")]
    pub spritesheets: Vec<Spritesheet>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Spritesheet {
    Normal(NormalSpritesheet),
    Fallback(FallbackSpritesheet),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NormalSpritesheet {
    pub file: String,

    pub sprite_width: Option<u32>,
    pub sprite_height: Option<u32>,
    pub sprite_offset_x: Option<i32>,
    pub sprite_offset_y: Option<i32>,

    #[serde(deserialize_with = "deserialize_range_comment", rename = "//")]
    pub range: (u32, u32),

    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Hash, Eq, PartialEq)]
pub enum AdditionalTileId {
    #[serde(rename = "center")]
    Center,

    #[serde(rename = "corner")]
    Corner,

    #[serde(rename = "t_connection")]
    TConnection,

    #[serde(rename = "edge")]
    Edge,

    #[serde(rename = "end_piece")]
    EndPiece,

    #[serde(rename = "unconnected")]
    Unconnected,

    #[serde(rename = "broken")]
    Broken,

    #[serde(rename = "open")]
    Open,

    // ???
    // BrownLikeBears -> tile_config.json -> Line 5688
    #[serde(rename = "h")]
    H,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdditionalTile {
    pub id: AdditionalTileId,
    pub rotates: Option<bool>,
    pub animated: Option<bool>,
    pub fg: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
    pub bg: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tile {
    pub id: MeabyVec<CDDAIdentifier>,
    pub fg: Option<MeabyVec<MeabyWeightedSprite<SpriteIndex>>>,
    pub bg: Option<MeabyVec<MeabyWeightedSprite<SpriteIndex>>>,
    pub rotates: Option<bool>,
    pub animated: Option<bool>,
    pub multitile: Option<bool>,
    pub additional_tiles: Option<Vec<AdditionalTile>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FallbackSpritesheet {
    pub file: String,

    // TODO: Idk what this is for
    pub tiles: Vec<()>,

    pub ascii: Vec<AsciiCharGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AsciiCharGroup {
    pub offset: i32,
    pub bold: bool,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileInfo {
    pub pixelscale: u32,
    pub width: u32,
    pub height: u32,
    pub zlevel_height: u32,
    pub iso: bool,
    pub retract_dist_min: f32,
    pub retract_dist_max: f32,
}
