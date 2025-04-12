use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::terrain::CDDATerrain;
use crate::legacy_tileset::tile_config::{
    AdditionalTile, AdditionalTileId, Spritesheet, Tile, TileConfig,
};
use crate::util::{CDDAIdentifier, MeabyVec};
use log::info;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub(crate) mod handlers;
mod io;
pub(crate) mod tile_config;

pub type SpriteIndex = u32;
pub type FinalIds = Option<Vec<WeightedSprite<SpriteIndex>>>;
pub type AdditionalTileIds = Option<Vec<WeightedSprite<Vec<SpriteIndex>>>>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SpriteLayer {
    Bg = 0,
    Fg = 1,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileNew {
    file: String,
    #[serde(rename = "//")]
    range: Option<String>,

    sprite_width: Option<u32>,
    sprite_height: Option<u32>,
    sprite_offset_x: Option<i32>,
    sprite_offset_y: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileInfo {
    pixelscale: u32,
    width: u32,
    height: u32,
    zlevel_height: u32,
    iso: bool,
    retract_dist_min: f32,
    retract_dist_max: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpritesheetConfig {
    #[serde(rename = "tiles-new")]
    tiles_new: Vec<TileNew>,

    tile_info: Vec<TileInfo>,
}

#[derive(Debug)]
pub struct ForeBackIds<T> {
    pub fg: T,
    pub bg: T,
}

impl<T> ForeBackIds<T> {
    pub fn new(fg: T, bg: T) -> Self {
        Self { fg, bg }
    }
}

#[derive(Debug)]
pub struct MultitileSprite {
    pub ids: ForeBackIds<AdditionalTileIds>,
    pub animated: bool,
    pub rotates: bool,
}

#[derive(Debug)]
pub enum Sprite {
    Single {
        ids: ForeBackIds<FinalIds>,
        rotates: bool,
        animated: bool,
    },
    Open {
        ids: ForeBackIds<FinalIds>,
        open: ForeBackIds<FinalIds>,
        rotates: bool,
        animated: bool,
    },
    Broken {
        ids: ForeBackIds<FinalIds>,
        broken: ForeBackIds<FinalIds>,
        rotates: bool,
        animated: bool,
    },
    Multitile {
        ids: ForeBackIds<FinalIds>,

        animated: bool,
        rotates: bool,

        edge: Option<MultitileSprite>,
        corner: Option<MultitileSprite>,
        center: Option<MultitileSprite>,
        t_connection: Option<MultitileSprite>,
        end_piece: Option<MultitileSprite>,
        unconnected: Option<MultitileSprite>,
    },
}

impl Sprite {
    pub fn is_animated(&self) -> bool {
        match self {
            Sprite::Single { animated, .. } => animated.clone(),
            Sprite::Open { animated, .. } => animated.clone(),
            Sprite::Broken { animated, .. } => animated.clone(),
            Sprite::Multitile { animated, .. } => animated.clone(),
        }
    }

    pub fn get_fg_id(&self) -> Option<MeabyVec<SpriteIndex>> {
        match self {
            Sprite::Single { ids, animated, .. } => match *animated {
                true => match &ids.fg {
                    None => None,
                    Some(fg) => {
                        if fg.len() == 0 {
                            return None;
                        }

                        Some(MeabyVec::Vec(
                            fg.to_vec().into_iter().map(|v| v.sprite).collect(),
                        ))
                    }
                },
                false => match &ids.fg {
                    None => None,
                    Some(fg) => {
                        if fg.len() == 0 {
                            return None;
                        }

                        Some(MeabyVec::Single(fg.get_random().clone()))
                    }
                },
            },
            Sprite::Multitile { ids, animated, .. } => match *animated {
                true => None,
                false => match &ids.fg {
                    None => None,
                    Some(fg) => {
                        if fg.len() == 0 {
                            return None;
                        }

                        Some(MeabyVec::Single(fg.get_random().clone()))
                    }
                },
            },
            Sprite::Open { .. } => todo!(),
            Sprite::Broken { .. } => todo!(),
        }
    }

    pub fn get_bg_id(&self) -> Option<MeabyVec<SpriteIndex>> {
        match self {
            Sprite::Single { ids, animated, .. } => match *animated {
                true => match &ids.bg {
                    None => None,
                    Some(bg) => {
                        if bg.len() == 0 {
                            return None;
                        }

                        Some(MeabyVec::Vec(
                            bg.to_vec().into_iter().map(|v| v.sprite).collect(),
                        ))
                    }
                },
                false => match &ids.bg {
                    None => None,
                    Some(bg) => {
                        if bg.len() == 0 {
                            return None;
                        }

                        Some(MeabyVec::Single(bg.get_random().clone()))
                    }
                },
            },
            Sprite::Multitile { ids, animated, .. } => match *animated {
                true => None,
                false => match &ids.bg {
                    None => None,
                    Some(bg) => {
                        if bg.len() == 0 {
                            return None;
                        }

                        Some(MeabyVec::Single(bg.get_random().clone()))
                    }
                },
            },
            Sprite::Open { .. } => todo!(),
            Sprite::Broken { .. } => todo!(),
        }
    }
}

fn to_weighted_vec(
    indices: Option<MeabyVec<MeabyWeightedSprite<SpriteIndex>>>,
) -> Option<Vec<WeightedSprite<SpriteIndex>>> {
    indices.map(|fg| fg.map(|mw| mw.weighted()))
}

fn to_weighted_vec_additional_exception(
    indices: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
) -> Option<Vec<WeightedSprite<SpriteIndex>>> {
    indices.map(|fg| {
        fg.map(|mw| {
            let weighted = mw.weighted();
            let single = weighted.sprite.into_single().unwrap();

            WeightedSprite::new(single, weighted.weight)
        })
    })
}

fn to_weighted_vec_additional(
    indices: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
) -> Option<Vec<WeightedSprite<Vec<SpriteIndex>>>> {
    indices.map(|fg| {
        fg.map(|mw| {
            let weighted = mw.weighted();
            let single = weighted.sprite.into_vec();

            WeightedSprite::new(single, weighted.weight)
        })
    })
}

fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, anyhow::Error> {
    let mut additional_tile_ids = HashMap::new();
    // Special cases for open and broken
    let mut broken: Option<(&AdditionalTile, ForeBackIds<FinalIds>)> = None;
    let mut open: Option<(&AdditionalTile, ForeBackIds<FinalIds>)> = None;

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
                let bg = to_weighted_vec_additional(additional_tile.bg.clone());

                additional_tile_ids.insert(
                    additional_tile.id.clone(),
                    MultitileSprite {
                        ids: ForeBackIds::new(fg, bg),
                        animated: additional_tile.animated.unwrap_or(false),
                        rotates: additional_tile.rotates.unwrap_or(false),
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

pub fn get_id_map_from_config(config: TileConfig) -> HashMap<CDDAIdentifier, Sprite> {
    let mut id_map = HashMap::new();

    let mut normal_spritesheets = vec![];
    for spritesheet in config.spritesheets.iter() {
        match spritesheet {
            Spritesheet::Normal(n) => normal_spritesheets.push(n),
            Spritesheet::Fallback(_) => {}
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
                        get_multitile_sprite_from_additional_tiles(tile, additional_tiles).unwrap(),
                    );
                });
            }
        }
    }

    id_map
}

pub struct Tilesheet {
    pub id_map: HashMap<CDDAIdentifier, Sprite>,
}

impl Tilesheet {
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

    pub fn get_sprite(
        &self,
        id: &CDDAIdentifier,
        terrain: &HashMap<CDDAIdentifier, CDDATerrain>,
        furniture: &HashMap<CDDAIdentifier, CDDAFurniture>,
    ) -> &Sprite {
        match self.id_map.get(&id) {
            None => {
                info!(
                    "Could not find {} in tilesheet ids, trying to use looks_like property",
                    id
                );

                self.get_looks_like_sprite(&id, terrain, furniture)
                    .unwrap_or_else(|| {
                        // If a tileset can't find a tile for any item in the looks_like chain,
                        // it will default to the ascii symbol.
                        todo!()
                    })
            }
            Some(s) => s,
        }
    }
}

pub trait GetRandom<T> {
    fn get_random(&self) -> &T;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeightedSprite<T> {
    pub sprite: T,
    pub weight: i32,
}

impl<T> WeightedSprite<T> {
    pub fn new(sprite: T, weight: i32) -> Self {
        Self { sprite, weight }
    }
}

impl<T> GetRandom<T> for Vec<WeightedSprite<T>> {
    fn get_random(&self) -> &T {
        let mut weights = vec![];
        self.iter().for_each(|v| weights.push(v.weight));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");
        let mut rng = rng();

        let chosen_index = weighted_index.sample(&mut rng);

        &self.get(chosen_index).unwrap().sprite
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MeabyWeightedSprite<T> {
    NotWeighted(T),
    Weighted(WeightedSprite<T>),
}

impl<T> MeabyWeightedSprite<T> {
    pub fn map<F, R>(self, fun: F) -> R
    where
        F: Fn(T) -> R,
    {
        match self {
            MeabyWeightedSprite::NotWeighted(nw) => fun(nw),
            MeabyWeightedSprite::Weighted(w) => fun(w.sprite),
        }
    }

    pub fn data(self) -> T {
        match self {
            MeabyWeightedSprite::NotWeighted(nw) => nw,
            MeabyWeightedSprite::Weighted(w) => w.sprite,
        }
    }

    pub fn weighted(self) -> WeightedSprite<T> {
        match self {
            MeabyWeightedSprite::NotWeighted(d) => WeightedSprite {
                sprite: d,
                weight: 1,
            },
            MeabyWeightedSprite::Weighted(w) => w,
        }
    }
}
