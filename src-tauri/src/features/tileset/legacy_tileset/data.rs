use crate::features::tileset::data::AdditionalTileType;
use crate::features::tileset::legacy_tileset::{Rotates, SpriteIndex};
use crate::features::tileset::{FgBgIds, SingleSprite, Sprite};
use cdda_lib::types::{CDDAIdentifier, MeabyVec, MeabyWeighted, Weighted};
use log::info;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

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

/// TODO: This is a very bad hack to make the deserialization work with MeabyVec<MeabyWeighted<u32>>
/// For example, when deserializing a array with two numbers like: `[1, 2]`, these would be deserialized as
/// `MeabyVec::Single(MeabyWeighted::Weighted({data: 1, weight: 2}))`. This is a problem because here we
/// want the data serialized as `MeabyVec::Vec(MeabyWeighted::NotWeighted(1), MeabyWeighted::NotWeighted(2))`
///
/// This is happening because there is a deserialization implementation for Weighted which makes it so
/// arrays with two fields are deserialized into a Weighted struct. To fix the above problem, the `visit_seq()` implementation
/// has been removed from the WeightedVisitor in the Deserialize implementation of this struct.
/// Now only objects with a `weighted` and `sprite` property are treated as Weighted instances
#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub(crate) struct __WeightedDeserializationFix<T> {
    pub data: T,
    pub weight: i32,
}

impl<T> Into<Weighted<T>> for __WeightedDeserializationFix<T> {
    fn into(self) -> Weighted<T> {
        Weighted::new(self.data, self.weight)
    }
}

impl<T> __WeightedDeserializationFix<T> {
    pub fn new(data: impl Into<T>, weight: i32) -> Self {
        Self {
            data: data.into(),
            weight,
        }
    }
}

impl<'de, T> Deserialize<'de> for __WeightedDeserializationFix<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Helper visitor to handle both array and object formats
        struct WeightedVisitor<T> {
            _marker: std::marker::PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for WeightedVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = __WeightedDeserializationFix<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("expected { \"weight\": i32, \"sprite\": T }")
            }

            // Handle the map format: { "weight": ..., "sprite": ... }
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut data: Option<T> = None;
                let mut weight: Option<i32> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "sprite" => {
                            data = Some(map.next_value()?);
                        },
                        "weight" => {
                            weight = Some(map.next_value()?);
                        },
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                &key,
                                &["sprite", "weight"],
                            ));
                        },
                    }
                }

                let data = data
                    .ok_or_else(|| serde::de::Error::missing_field("sprite"))?;
                let weight = weight
                    .ok_or_else(|| serde::de::Error::missing_field("weight"))?;

                Ok(__WeightedDeserializationFix { data, weight })
            }
        }

        deserializer.deserialize_any(WeightedVisitor {
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum __MeabyWeightedDeserializationFix<T> {
    Weighted(__WeightedDeserializationFix<T>),
    NotWeighted(T),
}

impl<T> Into<MeabyWeighted<T>> for __MeabyWeightedDeserializationFix<T> {
    fn into(self) -> MeabyWeighted<T> {
        match self {
            __MeabyWeightedDeserializationFix::Weighted(w) => {
                MeabyWeighted::Weighted(w.into())
            },
            __MeabyWeightedDeserializationFix::NotWeighted(nw) => {
                MeabyWeighted::NotWeighted(nw)
            },
        }
    }
}

pub(crate) fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, anyhow::Error> {
    let mut additional_tile_ids = HashMap::new();
    // Special cases for open and broken
    let mut broken: Option<SingleSprite> = None;
    let mut open: Option<SingleSprite> = None;

    for additional_tile in additional_tiles {
        match additional_tile.id {
            AdditionalTileType::Broken => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                broken = Some(SingleSprite {
                    ids: FgBgIds::new(fg, bg),
                    animated: false,
                    rotates: false,
                });
            },
            AdditionalTileType::Open => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                open = Some(SingleSprite {
                    ids: FgBgIds::new(fg, bg),
                    animated: false,
                    rotates: false,
                });
            },
            _ => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                additional_tile_ids.insert(
                    additional_tile.id.clone(),
                    SingleSprite {
                        ids: FgBgIds::new(fg, bg),
                        animated: additional_tile.animated.unwrap_or(false),
                        rotates: additional_tile.rotates.unwrap_or(true),
                    },
                );
            },
        }
    }

    let fg = to_weighted_vec(tile.fg.clone());
    let bg = to_weighted_vec(tile.bg.clone());

    Ok(Sprite::Multitile {
        fallback: SingleSprite {
            ids: FgBgIds::new(fg, bg),
            rotates: tile.rotates.unwrap_or(false),
            animated: tile.animated.unwrap_or(false),
        },
        center: additional_tile_ids.remove(&AdditionalTileType::Center),
        corner: additional_tile_ids.remove(&AdditionalTileType::Corner),
        edge: additional_tile_ids.remove(&AdditionalTileType::Edge),
        t_connection: additional_tile_ids
            .remove(&AdditionalTileType::TConnection),
        unconnected: additional_tile_ids
            .remove(&AdditionalTileType::Unconnected),
        end_piece: additional_tile_ids.remove(&AdditionalTileType::EndPiece),
        broken,
        open,
    })
}

pub(crate) fn to_weighted_vec(
    indices: Option<
        MeabyVec<__MeabyWeightedDeserializationFix<MeabyVec<SpriteIndex>>>,
    >,
) -> Option<Vec<Weighted<Rotates>>> {
    let mut mapped_indices = Vec::new();

    for fg_indices_outer in indices?.into_vec() {
        let (indices_vec, weight) = match fg_indices_outer {
            __MeabyWeightedDeserializationFix::NotWeighted(nw) => {
                (nw.into_vec(), 1)
            },
            __MeabyWeightedDeserializationFix::Weighted(w) => {
                (w.data.into_vec(), w.weight)
            },
        };

        match Rotates::try_from(indices_vec) {
            Ok(v) => {
                mapped_indices.push(Weighted::new(v, weight));
            },
            Err(e) => {
                // TODO: This happens when the supplied fg or bg is an empty array
                info!(
                    "{}, this is probably due to an empty array. Ignoring this entry ",
                    e
                );
                continue;
            },
        }
    }

    Some(mapped_indices)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct AdditionalTile {
    pub id: AdditionalTileType,
    pub rotates: Option<bool>,
    pub animated: Option<bool>,
    pub fg: Option<
        MeabyVec<__MeabyWeightedDeserializationFix<MeabyVec<SpriteIndex>>>,
    >,
    pub bg: Option<
        MeabyVec<__MeabyWeightedDeserializationFix<MeabyVec<SpriteIndex>>>,
    >,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct Tile {
    pub id: MeabyVec<CDDAIdentifier>,
    pub fg: Option<
        MeabyVec<__MeabyWeightedDeserializationFix<MeabyVec<SpriteIndex>>>,
    >,
    pub bg: Option<
        MeabyVec<__MeabyWeightedDeserializationFix<MeabyVec<SpriteIndex>>>,
    >,
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
    pub pixelscale: Option<u32>,
    pub width: u32,
    pub height: u32,
    pub zlevel_height: Option<u32>,
    pub iso: Option<bool>,
    pub retract_dist_min: Option<f32>,
    pub retract_dist_max: Option<f32>,
}

#[cfg(test)]
mod tests {
    use cdda_lib::types::Weighted;
    use serde_json::json;

    #[test]
    pub fn test_deserialize() {
        let data = json!(
          [
            { "weight": 8, "sprite": 1410 },
            { "weight": 8, "sprite": 1411 },
            { "weight": 8, "sprite": 1412 },
            { "weight": 8, "sprite": 1413 }
          ]
        );

        let tile: Vec<Weighted<u32>> = serde_json::from_value(data).unwrap();
        dbg!(tile);
    }
}
