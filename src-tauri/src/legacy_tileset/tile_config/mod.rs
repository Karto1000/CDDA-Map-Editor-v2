pub(crate) mod reader;

use crate::legacy_tileset::{MeabyWeightedSprite, TileInfo};
use crate::util::{CDDAIdentifier, MeabyVec};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

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

    let from = u32::from_str_radix(left, 36)
        .map_err(|_| Error::custom(format!("Failed to parse {} as u32", left)))?;

    let to = u32::from_str_radix(right, 36)
        .map_err(|_| Error::custom(format!("Failed to parse {} as u32", left)))?;

    Ok((from, to))
}

// https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/TILESET.md#rotations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rotations<T> {
    None(T),
    Horizontal((T, T)),
    Full((T, T, T, T)),
}

impl<T: Clone> Rotations<T> {
    pub fn up(&self) -> T {
        match self {
            Rotations::None(d) => d.clone(),
            Rotations::Horizontal((l, r)) => todo!("Don't know what to do here"),
            Rotations::Full((u, _, _, _)) => u.clone(),
        }
    }

    pub fn right(&self) -> T {
        match self {
            Rotations::None(d) => d.clone(),
            Rotations::Horizontal((_, r)) => r.clone(),
            Rotations::Full((_, r, _, _)) => r.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TileConfig {
    pub tile_info: Vec<TileInfo>,

    #[serde(rename = "tiles-new")]
    pub spritesheets: Vec<Spritesheet>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Spritesheet {
    Normal(NormalSpritesheet),
    Fallback(FallbackSpritesheet),
}

#[derive(Debug, Deserialize)]
pub struct NormalSpritesheet {
    pub file: String,

    #[serde(deserialize_with = "deserialize_range_comment", rename = "//")]
    pub range: (u32, u32),

    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Deserialize, Hash, Eq, PartialEq)]
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdditionalTile {
    pub id: AdditionalTileId,
    pub rotates: Option<bool>,
    pub fg: Option<MeabyVec<MeabyWeightedSprite<Rotations<u32>>>>,
    pub bg: Option<MeabyVec<MeabyWeightedSprite<Rotations<u32>>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tile {
    pub id: MeabyVec<CDDAIdentifier>,
    pub fg: Option<MeabyVec<MeabyWeightedSprite<Rotations<u32>>>>,
    pub bg: Option<MeabyVec<MeabyWeightedSprite<Rotations<u32>>>>,
    pub rotates: Option<bool>,
    pub multitile: Option<bool>,
    pub additional_tiles: Option<Vec<AdditionalTile>>,
}

#[derive(Debug, Deserialize)]
pub struct FallbackSpritesheet {
    pub file: String,

    // TODO: Idk what this is for
    pub tiles: Vec<()>,

    pub ascii: Vec<AsciiTile>,
}

#[derive(Debug, Deserialize)]
pub struct AsciiTile {
    pub offset: i32,
    pub bold: bool,
    pub color: String,
}
