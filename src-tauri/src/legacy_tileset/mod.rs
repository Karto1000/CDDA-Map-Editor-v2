use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::terrain::CDDATerrain;
use crate::cdda_data::{ConnectGroup, TileLayer};
use crate::legacy_tileset::tile_config::{
    AdditionalTile, AdditionalTileId, FallbackSpritesheet, Spritesheet, Tile, TileConfig,
};
use crate::util::{CDDAIdentifier, MeabyVec};
use log::info;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};

pub(crate) mod handlers;
mod io;
pub(crate) mod tile_config;

pub type SpriteIndex = u32;
pub type FinalIds = Option<Vec<WeightedSprite<SpriteIndex>>>;
pub type AdditionalTileIds = Option<Vec<WeightedSprite<Vec<SpriteIndex>>>>;

const FALLBACK_TILE_ROW_SIZE: usize = 16;
const FALLBACK_TILE_WIDTH: usize = 32;
const FALLBACK_TILE_MAPPING: &'static [(&'static str, u32)] = &[
    // Ignore some textures at the start and end of each color
    ("!", 33),
    ("#", 35),
    ("$", 36),
    ("%", 37),
    ("&", 38),
    ("(", 40),
    (")", 41),
    ("*", 42),
    ("+", 43),
    ("0", 48),
    ("1", 49),
    ("2", 50),
    ("3", 51),
    ("4", 52),
    ("5", 53),
    ("6", 54),
    ("7", 55),
    ("8", 56),
    ("9", 57),
    (":", 58),
    (";", 59),
    ("<", 60),
    ("=", 61),
    ("?", 62),
    ("@", 63),
    ("A", 64),
    ("B", 65),
    ("C", 66),
    ("D", 67),
    ("E", 68),
    ("F", 69),
    ("G", 70),
    ("H", 71),
    ("I", 72),
    ("J", 73),
    ("K", 74),
    ("L", 75),
    ("M", 76),
    ("N", 77),
    ("O", 78),
    ("P", 79),
    ("Q", 80),
    ("R", 81),
    ("S", 82),
    ("T", 83),
    ("U", 84),
    ("V", 85),
    ("W", 86),
    ("X", 87),
    ("Y", 88),
    ("Z", 89),
    ("[", 90),
    (r"\", 91),
    ("]", 92),
    ("^", 93),
    ("_", 94),
    ("`", 95),
    ("{", 122),
    ("}", 124),
    ("|", 178),
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CardinalDirection {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SpriteLayer {
    Bg = 0,
    Fg = 1,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MappedSprite {
    pub terrain: Option<CDDAIdentifier>,
    pub furniture: Option<CDDAIdentifier>,
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
    pub bg: FinalIds,
}

impl<T> ForeBackIds<T> {
    pub fn new(fg: T, bg: FinalIds) -> Self {
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

    fn get_random_animated_sprite(
        ids: &Vec<WeightedSprite<SpriteIndex>>,
    ) -> Option<MeabyVec<SpriteIndex>> {
        if ids.len() == 0 {
            return None;
        }

        Some(MeabyVec::Vec(
            ids.to_vec().into_iter().map(|v| v.sprite).collect(),
        ))
    }

    fn get_random_sprite(ids: &Vec<WeightedSprite<SpriteIndex>>) -> Option<MeabyVec<SpriteIndex>> {
        if ids.len() == 0 {
            return None;
        }

        Some(MeabyVec::Single(ids.get_random().clone()))
    }

    fn get_random_additional_tile_sprite(
        direction: CardinalDirection,
        ids: &Vec<WeightedSprite<Vec<SpriteIndex>>>,
    ) -> Option<MeabyVec<SpriteIndex>> {
        if ids.len() == 0 {
            return None;
        }

        Some(MeabyVec::Single(
            ids.get_random()
                .get(direction.clone() as usize)
                .expect(format!("t_connection to have a {:?} sprite", direction).as_str())
                .clone(),
        ))
    }

    fn edit_connection_groups(flags: &Vec<String>, connection: &mut HashSet<CDDAIdentifier>) {
        // "WALL is implied by the flags WALL and CONNECT_WITH_WALL"
        // TODO: I assume that the flag WIRED_WALL also implies this although this is
        // not mentioned anywhere
        if flags.contains(&"WALL".to_string())
            || flags.contains(&"CONNECT_WITH_WALL".to_string())
            || flags.contains(&"WIRED_WALL".to_string())
        {
            connection.insert(CDDAIdentifier("WALL".to_string()));
        }

        // "INDOORFLOOR is implied by the flag INDOORS"
        if flags.contains(&"INDOORS".to_string()) {
            connection.insert(CDDAIdentifier("INDOORFLOOR".to_string()));
        }
    }

    fn get_matching_list(
        this_id: &CDDAIdentifier,
        layer: &TileLayer,
        json_data: &DeserializedCDDAJsonData,
        top: Option<CDDAIdentifier>,
        right: Option<CDDAIdentifier>,
        bottom: Option<CDDAIdentifier>,
        left: Option<CDDAIdentifier>,
    ) -> (bool, bool, bool, bool) {
        let mut this_connects_to = json_data.get_connects_to(Some(this_id.clone()), layer);
        let mut top_connect_groups = json_data.get_connect_groups(top.clone(), layer);
        let mut right_connect_groups = json_data.get_connect_groups(right.clone(), layer);
        let mut bottom_connect_groups = json_data.get_connect_groups(bottom.clone(), layer);
        let mut left_connect_groups = json_data.get_connect_groups(left.clone(), layer);

        let this_flags = json_data.get_flags(Some(this_id.clone()), layer);
        let top_flags = json_data.get_flags(top.clone(), layer);
        let right_flags = json_data.get_flags(right.clone(), layer);
        let bottom_flags = json_data.get_flags(bottom.clone(), layer);
        let left_flags = json_data.get_flags(left.clone(), layer);

        Self::edit_connection_groups(&this_flags, &mut this_connects_to);
        Self::edit_connection_groups(&top_flags, &mut top_connect_groups);
        Self::edit_connection_groups(&right_flags, &mut right_connect_groups);
        Self::edit_connection_groups(&bottom_flags, &mut bottom_connect_groups);
        Self::edit_connection_groups(&left_flags, &mut left_connect_groups);

        let can_connect_top = this_connects_to
            .intersection(&top_connect_groups)
            .next()
            // We have the second check here since the tile can also connect to itself
            // TODO: I think there's a no self connect flag to toggle this behaviour
            // although im not sure
            .is_some()
            || this_id == &top.unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_right = this_connects_to
            .intersection(&right_connect_groups)
            .next()
            .is_some()
            || this_id == &right.unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_bottom = this_connects_to
            .intersection(&bottom_connect_groups)
            .next()
            .is_some()
            || this_id == &bottom.unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_left = this_connects_to
            .intersection(&left_connect_groups)
            .next()
            .is_some()
            || this_id == &left.unwrap_or(CDDAIdentifier("".to_string()));

        (
            can_connect_top,
            can_connect_right,
            can_connect_bottom,
            can_connect_left,
        )
    }

    pub fn get_fg_id(
        &self,
        this_id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
        layer: &TileLayer,
        top: Option<CDDAIdentifier>,
        right: Option<CDDAIdentifier>,
        bottom: Option<CDDAIdentifier>,
        left: Option<CDDAIdentifier>,
    ) -> Option<MeabyVec<SpriteIndex>> {
        match self {
            Sprite::Single { ids, animated, .. } => match *animated {
                true => match &ids.fg {
                    None => None,
                    Some(fg) => Self::get_random_animated_sprite(fg),
                },
                false => match &ids.fg {
                    None => None,
                    Some(fg) => Self::get_random_sprite(fg),
                },
            },
            Sprite::Multitile {
                animated,
                center,
                corner,
                t_connection,
                edge,
                unconnected,
                end_piece,
                ..
            } => match *animated {
                true => todo!(),
                false => {
                    let matching_list = Self::get_matching_list(
                        this_id, layer, json_data, top, right, bottom, left,
                    );

                    match matching_list {
                        (true, true, true, true) => match center {
                            None => None,
                            Some(center) => match &center.ids.fg {
                                None => None,
                                // TODO: Kind of weird but since the first elements index is 0 and
                                // the CardinalDirection North is mapped to 0, we can use North here
                                // instead of copying the contents of the function into this match arm
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    fg,
                                ),
                            },
                        },
                        (true, true, true, false) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::East,
                                    fg,
                                ),
                            },
                        },
                        (true, true, false, true) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::South,
                                    fg,
                                ),
                            },
                        },
                        (true, false, true, true) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::West,
                                    fg,
                                ),
                            },
                        },
                        (false, true, true, true) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    fg,
                                ),
                            },
                        },
                        (true, true, false, false) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::East,
                                    fg,
                                ),
                            },
                        },
                        (true, false, false, true) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::South,
                                    fg,
                                ),
                            },
                        },
                        (false, true, true, false) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    fg,
                                ),
                            },
                        },
                        (false, false, true, true) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::West,
                                    fg,
                                ),
                            },
                        },
                        (true, false, false, false) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::South,
                                    fg,
                                ),
                            },
                        },
                        (false, true, false, false) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::East,
                                    fg,
                                ),
                            },
                        },
                        (false, false, true, false) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    fg,
                                ),
                            },
                        },
                        (false, false, false, true) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::West,
                                    fg,
                                ),
                            },
                        },
                        (false, true, false, true) => match edge {
                            None => None,
                            Some(edge) => match &edge.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    // East-West
                                    CardinalDirection::East,
                                    fg,
                                ),
                            },
                        },
                        (true, false, true, false) => match edge {
                            None => None,
                            Some(edge) => match &edge.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    // North-South
                                    CardinalDirection::North,
                                    fg,
                                ),
                            },
                        },
                        (false, false, false, false) => match unconnected {
                            None => None,
                            Some(unconnected) => match &unconnected.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    // First
                                    CardinalDirection::North,
                                    fg,
                                ),
                            },
                        },
                    }
                }
            },
            Sprite::Open { .. } => todo!(),
            Sprite::Broken { .. } => todo!(),
        }
    }

    pub fn get_bg_id(
        &self,
        this_id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
        layer: &TileLayer,
        top: Option<CDDAIdentifier>,
        right: Option<CDDAIdentifier>,
        bottom: Option<CDDAIdentifier>,
        left: Option<CDDAIdentifier>,
    ) -> Option<MeabyVec<SpriteIndex>> {
        match self {
            Sprite::Single { ids, animated, .. } => match *animated {
                true => match &ids.bg {
                    None => None,
                    Some(bg) => Self::get_random_animated_sprite(bg),
                },
                false => match &ids.bg {
                    None => None,
                    Some(bg) => Self::get_random_sprite(bg),
                },
            },
            Sprite::Multitile {
                animated,
                center,
                corner,
                t_connection,
                edge,
                unconnected,
                end_piece,
                ..
            } => match *animated {
                true => todo!(),
                false => {
                    let matching_list = Self::get_matching_list(
                        this_id, layer, json_data, top, right, bottom, left,
                    );

                    match matching_list {
                        (true, true, true, true) => match center {
                            None => None,
                            Some(center) => match &center.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (true, true, true, false)
                        | (true, true, false, true)
                        | (true, false, true, true)
                        | (false, true, true, true) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (true, true, false, false)
                        | (true, false, false, true)
                        | (false, true, true, false)
                        | (false, false, true, true) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (true, false, false, false)
                        | (false, true, false, false)
                        | (false, false, true, false)
                        | (false, false, false, true) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (false, true, false, true) | (true, false, true, false) => match edge {
                            None => None,
                            Some(edge) => match &edge.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (false, false, false, false) => match unconnected {
                            None => None,
                            Some(unconnected) => match &unconnected.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                    }
                }
            },
            Sprite::Open { .. } => todo!(),
            Sprite::Broken { .. } => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct FallbackSprite {
    symbol: char,
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
                let bg = to_weighted_vec_additional_exception(additional_tile.bg.clone());

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

pub fn get_tilesheet_from_config(config: TileConfig) -> Tilesheet {
    let mut id_map = HashMap::new();
    let mut fallback_map = HashMap::new();

    let mut normal_spritesheets = vec![];
    let mut fallback_spritesheet = None;

    for spritesheet in config.spritesheets.iter() {
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
                        get_multitile_sprite_from_additional_tiles(tile, additional_tiles).unwrap(),
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

    Tilesheet {
        id_map,
        fallback_map,
    }
}

#[derive(Debug)]
pub enum SpriteKind<'a> {
    Exists(&'a Sprite),
    Fallback(SpriteIndex),
}

pub struct Tilesheet {
    pub id_map: HashMap<CDDAIdentifier, Sprite>,
    pub fallback_map: HashMap<String, SpriteIndex>,
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
        json_data: &DeserializedCDDAJsonData,
    ) -> SpriteKind {
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
