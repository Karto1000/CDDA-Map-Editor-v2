use crate::data::io::DeserializedCDDAJsonData;
use crate::data::vehicle_parts::CDDAVehiclePart;
use crate::features::map::MappedCDDAId;
use crate::features::program_data::ProgramData;
use crate::features::tileset::data::{
    AdditionalTileType, FALLBACK_TILE_MAPPING,
};
use crate::features::tileset::legacy_tileset::io::TileConfigLoader;
use crate::features::tileset::{ForeBackIds, SingleSprite, Sprite, Tilesheet};
use crate::util::{CardinalDirection, Load, Rotation};
use anyhow::{anyhow, Error};
use cdda_lib::types::{CDDAIdentifier, MeabyVec, MeabyWeighted, Weighted};
use data::{AdditionalTile, Tile};
use io::LegacyTilesheetLoader;
use log::{debug, info, warn};
use paste::paste;
use rand::distr::Distribution;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

mod data;
pub mod fallback;
pub mod io;

pub type SpriteIndex = u32;
pub type FinalIds = Option<Vec<Weighted<Rotates>>>;

#[derive(Debug, Clone)]
pub struct Rotated<T> {
    pub data: T,
    pub rotation: Rotation,
}

impl<T> Rotated<T> {
    pub fn none(data: T) -> Self {
        Self {
            data,
            rotation: Rotation::Deg0,
        }
    }

    pub fn new(data: T, rotation: Rotation) -> Self {
        Self { data, rotation }
    }
}

#[derive(Debug, Clone)]
pub enum Rotates {
    Auto(SpriteIndex),
    Pre2((SpriteIndex, SpriteIndex)),
    Pre4((SpriteIndex, SpriteIndex, SpriteIndex, SpriteIndex)),
}

impl Rotates {
    pub fn get(&self, direction: &CardinalDirection) -> &SpriteIndex {
        match self {
            Rotates::Auto(a) => a,
            Rotates::Pre2(p) => match direction {
                CardinalDirection::North => &p.0,
                CardinalDirection::East => &p.1,
                CardinalDirection::South => unreachable!(),
                CardinalDirection::West => unreachable!(),
            },
            Rotates::Pre4(p) => match direction {
                CardinalDirection::North => &p.0,
                CardinalDirection::East => &p.1,
                CardinalDirection::South => &p.2,
                CardinalDirection::West => &p.3,
            },
        }
    }
}

impl TryFrom<Vec<SpriteIndex>> for Rotates {
    type Error = Error;

    fn try_from(value: Vec<SpriteIndex>) -> Result<Self, Self::Error> {
        match (value.get(0), value.get(1), value.get(2), value.get(3)) {
            (Some(auto), None, None, None) => Ok(Rotates::Auto(auto.clone())),
            (Some(first), Some(second), None, None) => {
                Ok(Rotates::Pre2((first.clone(), second.clone())))
            },
            (Some(first), Some(second), Some(third), Some(fourth)) => {
                Ok(Rotates::Pre4((
                    first.clone(),
                    second.clone(),
                    third.clone(),
                    fourth.clone(),
                )))
            },
            (_, _, _, _) => Err(anyhow!(
                "Invalid vec supplied for rotation for sprite indices {:?}",
                value
            )),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default, Eq, PartialEq)]
pub struct TilesheetCDDAId {
    pub id: CDDAIdentifier,
    pub prefix: Option<String>,
    pub postfix: Option<String>,
}

impl Display for TilesheetCDDAId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.prefix {
            None => {},
            Some(prefix) => {
                write!(f, "{}_", prefix)?;
            },
        }

        let res = write!(f, "{}", self.id);

        match &self.postfix {
            None => res,
            Some(postfix) => {
                write!(f, "_{}", postfix)
            },
        }
    }
}

impl TilesheetCDDAId {
    pub fn simple(id: impl Into<CDDAIdentifier>) -> TilesheetCDDAId {
        TilesheetCDDAId {
            id: id.into(),
            prefix: None,
            postfix: None,
        }
    }

    pub fn full(&self) -> CDDAIdentifier {
        CDDAIdentifier(format!(
            "{}{}{}",
            self.prefix
                .clone()
                .map(|p| format!("{}_", p))
                .unwrap_or_default(),
            self.id,
            self.postfix
                .clone()
                .map(|p| format!("_{}", p))
                .unwrap_or_default(),
        ))
    }
}

fn to_weighted_vec(
    indices: Option<MeabyVec<MeabyWeighted<MeabyVec<SpriteIndex>>>>,
) -> Option<Vec<Weighted<Rotates>>> {
    let mut mapped_indices = Vec::new();

    for fg_indices_outer in indices?.into_vec() {
        let (indices_vec, weight) = match fg_indices_outer {
            MeabyWeighted::NotWeighted(nw) => (nw.into_vec(), 1),
            MeabyWeighted::Weighted(w) => (w.data.into_vec(), w.weight),
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

fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, Error> {
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
                    ids: ForeBackIds::new(fg, bg),
                    animated: false,
                    rotates: false,
                });
            },
            AdditionalTileType::Open => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                open = Some(SingleSprite {
                    ids: ForeBackIds::new(fg, bg),
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
                        ids: ForeBackIds::new(fg, bg),
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
            ids: ForeBackIds::new(fg, bg),
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

pub struct LegacyTilesheet {
    id_map: HashMap<CDDAIdentifier, Sprite>,
    fallback_map: HashMap<String, SpriteIndex>,
}

impl Tilesheet for LegacyTilesheet {
    fn get_fallback(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> SpriteIndex {
        match json_data.terrain.get(&id.tilesheet_id.id) {
            None => {},
            Some(t) => {
                // TODO: _LIGHT and _DARK should be handled, but right now i don't fully understand how they work

                let color = t
                    .color
                    .clone()
                    .unwrap_or(MeabyVec::Single("WHITE".to_string()))
                    .into_single()
                    .unwrap_or("WHITE".to_string())
                    .to_uppercase()
                    .replace("LIGHT_", "")
                    .replace("DARK_", "");

                let fallback_id =
                    format!("{}_{}", t.symbol.unwrap_or('?'), color);

                match self.fallback_map.get(&fallback_id).clone() {
                    None => {
                        info!("No fallback for {} found", fallback_id);
                    },
                    Some(_) => {},
                }

                return self
                    .fallback_map
                    .get(&fallback_id)
                    .unwrap_or(&FALLBACK_TILE_MAPPING.first().unwrap().1)
                    .clone();
            },
        }

        match json_data.furniture.get(&id.tilesheet_id.id) {
            None => {},
            Some(t) => {
                // TODO: _LIGHT and _DARK should be handled, but right now i don't fully understand how they work

                let color = t
                    .color
                    .clone()
                    .unwrap_or(MeabyVec::Single("WHITE".to_string()))
                    .into_single()
                    .unwrap_or("WHITE".to_string())
                    .to_uppercase()
                    .replace("LIGHT_", "")
                    .replace("DARK_", "");

                let fallback_id =
                    format!("{}_{}", t.symbol.unwrap_or('?'), color);

                match self.fallback_map.get(&fallback_id).clone() {
                    None => {
                        info!("No fallback for {} found", fallback_id);
                    },
                    Some(_) => {},
                }

                return self
                    .fallback_map
                    .get(&fallback_id)
                    .unwrap_or(&FALLBACK_TILE_MAPPING.first().unwrap().1)
                    .clone();
            },
        }

        FALLBACK_TILE_MAPPING.first().unwrap().1
    }
    fn get_sprite(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<&Sprite> {
        match self.id_map.get(&id.tilesheet_id.full()) {
            None => {
                debug!(
                    "Could not find {} in tilesheet ids, trying to use looks_like property",
                    id.tilesheet_id.full(),
                );

                let sliced_postfix = id.slice_right();
                debug!(
                    "Slicing postfix and trying to get sprite again, new id {}",
                    &sliced_postfix.tilesheet_id
                );

                match sliced_postfix.tilesheet_id.postfix {
                    None => {
                        // We want to get the sprites one more time after the entire postfix has been sliced
                        if id.tilesheet_id.postfix.is_some() {
                            return self.get_sprite(&sliced_postfix, json_data);
                        }
                    },
                    Some(_) => {
                        return self.get_sprite(&sliced_postfix, json_data);
                    },
                }

                self.get_looks_like_sprite(
                    &sliced_postfix.tilesheet_id.id,
                    &json_data,
                )
            },
            Some(s) => {
                debug!("Found sprite with id {}", id.tilesheet_id.full());
                Some(s)
            },
        }
    }
}

impl LegacyTilesheet {
    fn get_looks_like_sprite(
        &self,
        id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<&Sprite> {
        // Id of a similar item that this item looks like. The tileset loader will try to load the
        // tile for that item if this item doesn't have a tile. Looks_like entries are implicitly
        // chained, so if 'throne' has looks_like 'big_chair' and 'big_chair' has looks_like 'chair',
        // a throne will be displayed using the chair tile if tiles for throne and big_chair do not exist.
        // If a tileset can't find a tile for any item in the looks_like chain, it will default to the ascii symbol.

        macro_rules! get_looks_like_sprite {
            (
                $path: ident.$name: ident
            ) => {
                // The tiles with this property do not have a corresponding entry in the tilesheet which
                // means that we have to check this here dynamically
                match $path.$name.get(&id) {
                    None => {},
                    Some(s) => {
                        return match &s.looks_like {
                            None => None,
                            Some(ident) => {
                                // Stop stackoverflow when object "looks_like" itself
                                if ident == id {
                                    return self.id_map.get(ident);
                                }

                                // Check for a reference chain where an entry "a" looks like an entry "b" property
                                // and the entry "b" looks like the entry "a"

                                // TODO: Meaby try and detect every chain with any number of looks_like
                                // entries chained together
                                match $path.$name.get(&ident) {
                                    None => {},
                                    Some(v) => {
                                        if v.looks_like == Some(id.clone()) {
                                            return self.id_map.get(ident);
                                        }
                                    },
                                }

                                // "Looks like entries are implicitly chained"
                                match self.id_map.get(ident) {
                                    None => {
                                        self.get_looks_like_sprite(ident, json_data)
                                    },
                                    Some(s) => Some(s),
                                }
                            },
                        };
                    },
                };
            };
        }

        get_looks_like_sprite!(json_data.terrain);
        get_looks_like_sprite!(json_data.furniture);
        get_looks_like_sprite!(json_data.vehicle_parts);

        None
    }
}

pub async fn load_tilesheet(
    editor_data: &ProgramData,
) -> Result<Option<LegacyTilesheet>, Error> {
    let tileset = match &editor_data.config.selected_tileset {
        None => return Ok(None),
        Some(t) => t.clone(),
    };

    let cdda_path = match &editor_data.config.cdda_path {
        None => return Ok(None),
        Some(p) => p.clone(),
    };

    let config_path = cdda_path
        .join("gfx")
        .join(&tileset)
        .join("tile_config.json");

    let mut tile_config_loader = TileConfigLoader::new(config_path);
    let config = tile_config_loader.load().await?;

    let mut tilesheet_loader = LegacyTilesheetLoader::new(config);
    let tilesheet = tilesheet_loader.load().await?;

    Ok(Some(tilesheet))
}
