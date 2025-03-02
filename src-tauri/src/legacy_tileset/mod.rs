use crate::legacy_tileset::tile_config::{
    AdditionalTile, AdditionalTileId, Rotations, Spritesheet, Tile, TileConfig,
};
use crate::util::{CDDAIdentifier, MeabyVec};
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub(crate) mod handlers;
pub(crate) mod tile_config;

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

pub struct SpritesheetConfigReader {
    tileset_path: PathBuf,
}

impl SpritesheetConfigReader {
    pub fn new(tileset_path: PathBuf) -> Self {
        Self { tileset_path }
    }

    pub fn read(&self) -> serde_json::Result<SpritesheetConfig> {
        let config_path = self.tileset_path.join("tile_config.json");
        serde_json::from_str(fs::read_to_string(config_path).unwrap().as_str())
    }
}

pub type FinalIds = Option<Vec<WeightedSprite<Rotations<u32>>>>;

#[derive(Debug)]
pub struct ForeBackIds {
    pub rotates: bool,
    pub fg: FinalIds,
    pub bg: FinalIds,
}

impl ForeBackIds {
    pub fn new(fg: FinalIds, bg: FinalIds, rotates: bool) -> Self {
        Self { fg, bg, rotates }
    }
}

#[derive(Debug)]
pub enum Sprite {
    Single {
        ids: ForeBackIds,
    },
    Open {
        ids: ForeBackIds,
        open: ForeBackIds,
    },
    Broken {
        ids: ForeBackIds,
        broken: ForeBackIds,
    },
    Explosion {
        ids: ForeBackIds,
        center: ForeBackIds,
        edge: ForeBackIds,
        corner: ForeBackIds,
    },
    Multitile {
        ids: ForeBackIds,

        edge: Option<ForeBackIds>,
        corner: Option<ForeBackIds>,
        center: Option<ForeBackIds>,
        t_connection: Option<ForeBackIds>,
        end_piece: Option<ForeBackIds>,
        unconnected: Option<ForeBackIds>,
    },
}

impl Sprite {
    pub fn get_fg_id(&self) -> Option<u32> {
        match self {
            Sprite::Single { ids } => match &ids.fg {
                None => None,
                Some(v) => {
                    if v.len() == 0 {
                        return None;
                    }

                    let random_choice = v.get_random();
                    Some(random_choice.up())
                }
            },
            Sprite::Multitile { center, .. } => {
                if let Some(center) = center {
                    return match &center.fg {
                        None => None,
                        Some(v) => {
                            if v.len() == 0 {
                                return None;
                            }

                            let random_choice = v.get_random();
                            return Some(random_choice.up());
                        }
                    };
                }

                None
            }
            _ => None,
        }
    }

    pub fn get_bg_id(&self) -> Option<u32> {
        match self {
            Sprite::Single { ids } => match &ids.bg {
                None => None,
                Some(v) => {
                    if v.len() == 0 {
                        return None;
                    }

                    let random_choice = v.get_random();
                    Some(random_choice.up())
                }
            },
            _ => None,
        }
    }
}

fn to_weighted_vec<T>(
    indices: Option<MeabyVec<MeabyWeightedSprite<Rotations<T>>>>,
) -> Option<Vec<WeightedSprite<Rotations<T>>>> {
    indices.map(|fg| fg.map(|mw| mw.weighted()))
}

fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, anyhow::Error> {
    let mut additional_tile_ids = HashMap::new();

    for additional_tile in additional_tiles {
        let fg = to_weighted_vec(additional_tile.fg.clone());
        let bg = to_weighted_vec(additional_tile.bg.clone());

        additional_tile_ids.insert(
            additional_tile.id.clone(),
            ForeBackIds::new(fg, bg, additional_tile.rotates.unwrap_or(true)),
        );
    }

    let fg = to_weighted_vec::<u32>(tile.fg.clone());
    let bg = to_weighted_vec::<u32>(tile.bg.clone());

    match additional_tile_ids.remove(&AdditionalTileId::Broken) {
        None => {}
        Some(ids) => {
            return Ok(Sprite::Broken {
                ids: ForeBackIds::new(fg, bg, ids.rotates),
                broken: ids,
            })
        }
    }

    match additional_tile_ids.remove(&AdditionalTileId::Open) {
        None => {}
        Some(ids) => {
            return Ok(Sprite::Open {
                ids: ForeBackIds::new(fg, bg, ids.rotates),
                open: ids,
            })
        }
    }

    Ok(Sprite::Multitile {
        ids: ForeBackIds::new(fg, bg, tile.rotates.unwrap_or(true)),
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
                            ids: ForeBackIds::new(
                                fg.clone(),
                                bg.clone(),
                                tile.rotates.unwrap_or(true),
                            ),
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
