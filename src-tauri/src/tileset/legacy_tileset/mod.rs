use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::terrain::CDDATerrain;
use crate::tileset::io::{TilesheetConfigLoader, TilesheetLoader};
use crate::tileset::legacy_tileset::tile_config::{
    AdditionalTile, AdditionalTileId, LegacyTileConfig, Spritesheet, Tile,
};
use crate::tileset::{
    ForeBackIds, MeabyWeightedSprite, MultitileSprite, Sprite, SpriteKind, Tilesheet,
    WeightedSprite, FALLBACK_TILE_MAPPING, FALLBACK_TILE_ROW_SIZE,
};
use crate::util::{CDDAIdentifier, Load, MeabyVec};
use anyhow::{anyhow, Error};
use log::info;
use rand::distr::Distribution;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub(crate) mod tile_config;

pub type SpriteIndex = u32;
pub type FinalIds = Option<Vec<WeightedSprite<SpriteIndex>>>;
pub type AdditionalTileIds = Option<Vec<WeightedSprite<Rotates>>>;

#[derive(Debug, Clone)]
pub enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    pub fn deg(self) -> i32 {
        match self {
            Rotation::Deg0 => 0,
            Rotation::Deg90 => 90,
            Rotation::Deg180 => 180,
            Rotation::Deg270 => 270,
        }
    }
}

impl From<CardinalDirection> for Rotation {
    fn from(value: CardinalDirection) -> Self {
        match value {
            CardinalDirection::North => Self::Deg0,
            CardinalDirection::East => Self::Deg90,
            CardinalDirection::South => Self::Deg180,
            CardinalDirection::West => Self::Deg270,
        }
    }
}

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
            }
            (Some(first), Some(second), Some(third), Some(fourth)) => Ok(Rotates::Pre4((
                first.clone(),
                second.clone(),
                third.clone(),
                fourth.clone(),
            ))),
            (_, _, _, _) => Err(anyhow!("Invalid vec supplied for rotation")),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CardinalDirection {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MappedSprite {
    pub terrain: Option<CDDAIdentifier>,
    pub furniture: Option<CDDAIdentifier>,
    pub trap: Option<CDDAIdentifier>,
    pub monster: Option<CDDAIdentifier>,
}

fn to_weighted_vec(
    indices: Option<MeabyVec<MeabyWeightedSprite<SpriteIndex>>>,
) -> Option<Vec<WeightedSprite<SpriteIndex>>> {
    indices.map(|fg| fg.map(|mw| mw.weighted()))
}

fn to_weighted_vec_additional_exception(
    indices: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
) -> Option<Vec<WeightedSprite<SpriteIndex>>> {
    indices.map(|ids| {
        ids.map(|mw| {
            let weighted = mw.weighted();
            let single = match weighted.sprite.into_single() {
                None => return None,
                Some(v) => v,
            };

            Some(WeightedSprite::new(single, weighted.weight))
        })
        .into_iter()
        .filter_map(|v| {
            if v.is_none() {
                return None;
            }
            return Some(v.unwrap());
        })
        .collect()
    })
}

fn to_weighted_vec_additional(
    indices: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
) -> Option<Vec<WeightedSprite<Rotates>>> {
    indices.map(|fg| {
        fg.map(|mw| {
            let weighted = mw.weighted();
            let rotation = Rotates::try_from(weighted.sprite.into_vec()).unwrap();
            WeightedSprite::new(rotation, weighted.weight)
        })
    })
}

fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, Error> {
    let mut additional_tile_ids = HashMap::new();
    // Special cases for open and broken
    let mut broken: Option<(&AdditionalTile, ForeBackIds<FinalIds, FinalIds>)> = None;
    let mut open: Option<(&AdditionalTile, ForeBackIds<FinalIds, FinalIds>)> = None;

    for additional_tile in additional_tiles {
        match additional_tile.id {
            AdditionalTileId::Broken => {
                let fg = to_weighted_vec_additional_exception(additional_tile.fg.clone());
                let bg = to_weighted_vec_additional_exception(additional_tile.bg.clone());

                broken = Some((&additional_tile, ForeBackIds::new(fg, bg)))
            }
            AdditionalTileId::Open => {
                let fg = to_weighted_vec_additional_exception(additional_tile.fg.clone());
                let bg = to_weighted_vec_additional_exception(additional_tile.bg.clone());

                open = Some((&additional_tile, ForeBackIds::new(fg, bg)))
            }
            _ => {
                let fg = to_weighted_vec_additional(additional_tile.fg.clone());
                let bg = to_weighted_vec_additional_exception(additional_tile.bg.clone());

                additional_tile_ids.insert(
                    additional_tile.id.clone(),
                    MultitileSprite {
                        ids: ForeBackIds::new(fg, bg),
                        animated: additional_tile.animated.unwrap_or(false),
                        rotates: additional_tile.rotates.unwrap_or(true),
                    },
                );
            }
        }
    }

    let fg = to_weighted_vec(tile.fg.clone());
    let bg = to_weighted_vec(tile.bg.clone());

    match broken {
        None => {}
        Some((tile, ids)) => {
            return Ok(Sprite::Broken {
                ids: ForeBackIds::new(fg, bg),
                animated: tile.animated.unwrap_or(false),
                broken: ids,
                rotates: tile.rotates.unwrap_or(false),
            })
        }
    }

    match open {
        None => {}
        Some((tile, ids)) => {
            return Ok(Sprite::Open {
                ids: ForeBackIds::new(fg, bg),
                animated: tile.animated.unwrap_or(false),
                rotates: tile.rotates.unwrap_or(false),
                open: ids,
            })
        }
    }

    Ok(Sprite::Multitile {
        ids: ForeBackIds::new(fg, bg),
        rotates: tile.rotates.unwrap_or(false),
        animated: tile.animated.unwrap_or(false),
        center: additional_tile_ids.remove(&AdditionalTileId::Center),
        corner: additional_tile_ids.remove(&AdditionalTileId::Corner),
        edge: additional_tile_ids.remove(&AdditionalTileId::Edge),
        t_connection: additional_tile_ids.remove(&AdditionalTileId::TConnection),
        unconnected: additional_tile_ids.remove(&AdditionalTileId::Unconnected),
        end_piece: additional_tile_ids.remove(&AdditionalTileId::EndPiece),
    })
}

pub struct LegacyTilesheet {
    pub id_map: HashMap<CDDAIdentifier, Sprite>,
    pub fallback_map: HashMap<String, SpriteIndex>,
}

impl Tilesheet for LegacyTilesheet {
    fn get_sprite(&self, id: &CDDAIdentifier, json_data: &DeserializedCDDAJsonData) -> SpriteKind {
        match self.id_map.get(&id) {
            None => {
                info!(
                    "Could not find {} in tilesheet ids, trying to use looks_like property",
                    id
                );

                match self.get_looks_like_sprite(&id, &json_data.terrain, &json_data.furniture) {
                    None => {
                        match json_data.terrain.get(id) {
                            None => {}
                            Some(t) => {
                                return SpriteKind::Fallback(
                                    self.fallback_map
                                        .get(&format!(
                                            "{}_{}",
                                            t.symbol.unwrap_or('?'),
                                            t.color
                                                .clone()
                                                .unwrap_or(MeabyVec::Single("WHITE".to_string()))
                                                .into_single()
                                                .unwrap_or("WHITE".to_string())
                                        ))
                                        .unwrap_or(&FALLBACK_TILE_MAPPING.first().unwrap().1)
                                        .clone(),
                                )
                            }
                        }

                        match json_data.furniture.get(id) {
                            None => {}
                            Some(f) => {
                                return SpriteKind::Fallback(
                                    self.fallback_map
                                        .get(&format!(
                                            "{}_{}",
                                            f.symbol.unwrap_or('?'),
                                            f.color
                                                .clone()
                                                .unwrap_or(MeabyVec::Single("WHITE".to_string()))
                                                .into_single()
                                                .unwrap_or("WHITE".to_string())
                                        ))
                                        .unwrap_or(&FALLBACK_TILE_MAPPING.first().unwrap().1)
                                        .clone(),
                                )
                            }
                        };

                        SpriteKind::Fallback(FALLBACK_TILE_MAPPING.first().unwrap().1)
                    }
                    Some(s) => SpriteKind::Exists(s),
                }
            }
            Some(s) => SpriteKind::Exists(s),
        }
    }
}

impl LegacyTilesheet {
    fn get_looks_like_sprite(
        &self,
        id: &CDDAIdentifier,
        terrain: &HashMap<CDDAIdentifier, CDDATerrain>,
        furniture: &HashMap<CDDAIdentifier, CDDAFurniture>,
    ) -> Option<&Sprite> {
        // id of a similar item that this item looks like. The tileset loader will try to load the
        // tile for that item if this item doesn't have a tile. looks_like entries are implicitly
        // chained, so if 'throne' has looks_like 'big_chair' and 'big_chair' has looks_like 'chair',
        // a throne will be displayed using the chair tile if tiles for throne and big_chair do not exist.
        // If a tileset can't find a tile for any item in the looks_like chain, it will default to the ascii symbol.

        // The tiles with this property do not have a corresponding entry in the tilesheet which
        // means that we have to check this here dynamically
        match terrain.get(&id) {
            None => {}
            Some(s) => {
                return match &s.looks_like {
                    None => None,
                    Some(ident) => {
                        // "looks_like entries are implicitly chained"
                        match self.id_map.get(ident) {
                            None => self.get_looks_like_sprite(ident, terrain, furniture),
                            Some(s) => Some(s),
                        }
                    }
                };
            }
        };

        // Do again with furniture
        match furniture.get(&id) {
            None => None,
            Some(s) => match &s.looks_like {
                None => None,
                Some(ident) => match self.id_map.get(ident) {
                    None => self.get_looks_like_sprite(ident, terrain, furniture),
                    Some(s) => Some(s),
                },
            },
        }
    }
}

impl Load<LegacyTilesheet> for TilesheetLoader<LegacyTileConfig> {
    fn load(&self) -> Result<LegacyTilesheet, Error> {
        let mut id_map = HashMap::new();
        let mut fallback_map = HashMap::new();

        let mut normal_spritesheets = vec![];
        let mut fallback_spritesheet = None;

        for spritesheet in self.config.spritesheets.iter() {
            match spritesheet {
                Spritesheet::Normal(n) => normal_spritesheets.push(n),
                Spritesheet::Fallback(f) => fallback_spritesheet = Some(f),
            }
        }

        for spritesheet in normal_spritesheets {
            for tile in spritesheet.tiles.iter() {
                let is_multitile =
                    tile.multitile.unwrap_or_else(|| false) && tile.additional_tiles.is_some();

                if !is_multitile {
                    let fg = to_weighted_vec(tile.fg.clone());
                    let bg = to_weighted_vec(tile.bg.clone());

                    tile.id.for_each(|id| {
                        id_map.insert(
                            id.clone(),
                            Sprite::Single {
                                ids: ForeBackIds::new(fg.clone(), bg.clone()),
                                animated: tile.animated.unwrap_or(false),
                                rotates: tile.rotates.unwrap_or(false),
                            },
                        );
                    });
                }

                if is_multitile {
                    let additional_tiles = match &tile.additional_tiles {
                        None => unreachable!(),
                        Some(t) => t,
                    };

                    tile.id.for_each(|id| {
                        id_map.insert(
                            id.clone(),
                            get_multitile_sprite_from_additional_tiles(tile, additional_tiles)
                                .unwrap(),
                        );
                    });
                }
            }
        }

        let fallback_spritesheet = fallback_spritesheet.expect("Fallback spritesheet to exist");

        for ascii_group in fallback_spritesheet.ascii.iter() {
            for (character, offset) in FALLBACK_TILE_MAPPING {
                fallback_map.insert(
                    format!("{}_{}", character, ascii_group.color),
                    (offset / FALLBACK_TILE_ROW_SIZE as u32) + offset,
                );
            }
        }

        Ok(LegacyTilesheet {
            id_map,
            fallback_map,
        })
    }
}

impl Load<LegacyTileConfig> for TilesheetConfigLoader {
    fn load(&self) -> Result<LegacyTileConfig, Error> {
        let config_path = self.tileset_path.join("tile_config.json");
        let reader = BufReader::new(File::open(config_path)?);
        Ok(serde_json::from_reader(reader)?)
    }
}
