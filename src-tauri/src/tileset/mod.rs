use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::TileLayer;
use crate::tileset::legacy_tileset::tile_config::AdditionalTileId;
use crate::tileset::legacy_tileset::tile_config::AdditionalTileId::{
    Center, Corner, Edge, EndPiece, TConnection, Unconnected,
};
use crate::tileset::legacy_tileset::CardinalDirection::{
    East, North, South, West,
};
use crate::tileset::legacy_tileset::{
    AdditionalTileIds, CardinalDirection, FinalIds, MappedCDDAId, Rotated,
    Rotates, Rotation, SpriteIndex, TilesheetCDDAId,
};
use cdda_lib::types::{CDDAIdentifier, MeabyVec, Weighted};
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub(crate) mod handlers;
pub(crate) mod io;
pub(crate) mod legacy_tileset;

pub type MeabyAnimated<T> = MeabyVec<T>;

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

pub trait Tilesheet {
    fn get_sprite(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> SpriteKind;
}

#[derive(Debug)]
pub enum SpriteKind<'a> {
    Exists(&'a Sprite),
    Fallback(SpriteIndex),
}

#[derive(Debug)]
pub struct MultitileSprite {
    pub ids: ForeBackIds<AdditionalTileIds, FinalIds>,
    pub animated: bool,
    pub rotates: bool,
}

#[derive(Debug)]
pub enum Sprite {
    Single {
        ids: ForeBackIds<FinalIds, FinalIds>,
        rotates: bool,
        animated: bool,
    },
    Multitile {
        ids: ForeBackIds<FinalIds, FinalIds>,

        animated: bool,
        rotates: bool,

        edge: Option<MultitileSprite>,
        corner: Option<MultitileSprite>,
        center: Option<MultitileSprite>,
        t_connection: Option<MultitileSprite>,
        end_piece: Option<MultitileSprite>,
        unconnected: Option<MultitileSprite>,
        broken: ForeBackIds<FinalIds, FinalIds>,
        open: ForeBackIds<FinalIds, FinalIds>,
    },
}

impl Sprite {
    pub fn is_animated(&self) -> bool {
        match self {
            Sprite::Single { animated, .. } => animated.clone(),
            Sprite::Multitile { animated, .. } => animated.clone(),
        }
    }

    fn get_random_animated_sprite(
        ids: &Vec<WeightedSprite<SpriteIndex>>,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        if ids.len() == 0 {
            return None;
        }

        Some(Rotated::none(MeabyVec::Vec(
            ids.to_vec().into_iter().map(|v| v.sprite).collect(),
        )))
    }

    fn get_random_sprite(
        ids: &Vec<WeightedSprite<SpriteIndex>>,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        if ids.len() == 0 {
            return None;
        }

        let rotated =
            Rotated::none(MeabyAnimated::Single(ids.get_random().clone()));
        Some(rotated)
    }

    fn get_random_additional_tile_sprite(
        direction: CardinalDirection,
        tile_id: AdditionalTileId,
        rotates: bool,
        ids: &Vec<WeightedSprite<Rotates>>,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        if ids.len() == 0 {
            return None;
        }

        let rotated = match tile_id {
            Center | Unconnected => Rotated {
                data: MeabyAnimated::Single(
                    ids.get_random().get(&direction).clone(),
                ),
                rotation: Rotation::Deg0,
            },
            Corner | TConnection | Edge | EndPiece => match ids.get_random() {
                Rotates::Auto(a) => match rotates {
                    true => Rotated {
                        data: MeabyAnimated::Single(a.clone()),
                        rotation: Rotation::from(direction),
                    },
                    false => Rotated::none(MeabyAnimated::Single(a.clone())),
                },
                Rotates::Pre2(p) => match direction {
                    North => Rotated::none(MeabyAnimated::Single(p.0.clone())),
                    East => Rotated::none(MeabyAnimated::Single(p.1.clone())),
                    South => unreachable!(),
                    West => unreachable!(),
                },
                Rotates::Pre4(p) => match direction {
                    North => Rotated::none(MeabyAnimated::Single(p.0.clone())),
                    East => Rotated::none(MeabyAnimated::Single(p.1.clone())),
                    South => Rotated::none(MeabyAnimated::Single(p.2.clone())),
                    West => Rotated::none(MeabyAnimated::Single(p.3.clone())),
                },
            },
            _ => unreachable!(),
        };

        Some(rotated)
    }

    fn edit_connection_groups(
        flags: &Vec<String>,
        connection: &mut HashSet<CDDAIdentifier>,
    ) {
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
        this_id: &TilesheetCDDAId,
        layer: &TileLayer,
        json_data: &DeserializedCDDAJsonData,
        adjacent_sprites: &AdjacentSprites,
    ) -> (bool, bool, bool, bool) {
        let mut this_connects_to = json_data
            .get_connects_to(this_id.id.clone(), layer)
            .unwrap_or_default();

        let this_flags = json_data
            .get_flags(this_id.id.clone(), layer)
            .unwrap_or_default();

        let (mut top_connect_groups, top_flags) =
            match adjacent_sprites.top.clone() {
                None => (HashSet::new(), Vec::new()),
                Some(top) => (
                    json_data
                        .get_connect_groups(top.clone(), layer)
                        .unwrap_or_default(),
                    json_data.get_flags(top, layer).unwrap_or_default(),
                ),
            };

        let (mut right_connect_groups, right_flags) =
            match adjacent_sprites.right.clone() {
                None => (HashSet::new(), Vec::new()),
                Some(right) => (
                    json_data
                        .get_connect_groups(right.clone(), layer)
                        .unwrap_or_default(),
                    json_data.get_flags(right, layer).unwrap_or_default(),
                ),
            };

        let (mut bottom_connect_groups, bottom_flags) =
            match adjacent_sprites.bottom.clone() {
                None => (HashSet::new(), Vec::new()),
                Some(bottom) => (
                    json_data
                        .get_connect_groups(bottom.clone(), layer)
                        .unwrap_or_default(),
                    json_data.get_flags(bottom, layer).unwrap_or_default(),
                ),
            };

        let (mut left_connect_groups, left_flags) =
            match adjacent_sprites.left.clone() {
                None => (HashSet::new(), Vec::new()),
                Some(left) => (
                    json_data
                        .get_connect_groups(left.clone(), layer)
                        .unwrap_or_default(),
                    json_data.get_flags(left, layer).unwrap_or_default(),
                ),
            };

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
            || this_id.id
                == adjacent_sprites
                    .top
                    .clone()
                    .unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_right = this_connects_to
            .intersection(&right_connect_groups)
            .next()
            .is_some()
            || this_id.id
                == adjacent_sprites
                    .right
                    .clone()
                    .unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_bottom = this_connects_to
            .intersection(&bottom_connect_groups)
            .next()
            .is_some()
            || this_id.id
                == adjacent_sprites
                    .bottom
                    .clone()
                    .unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_left = this_connects_to
            .intersection(&left_connect_groups)
            .next()
            .is_some()
            || this_id.id
                == adjacent_sprites
                    .left
                    .clone()
                    .unwrap_or(CDDAIdentifier("".to_string()));

        (
            can_connect_top,
            can_connect_right,
            can_connect_bottom,
            can_connect_left,
        )
    }

    fn get_sprite_from_multitile_sprite(
        direction: &CardinalDirection,
        add_id: &AdditionalTileId,
        all_ids: &ForeBackIds<FinalIds, FinalIds>,
        sprite: Option<&MultitileSprite>,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        match sprite {
            None => match &all_ids.fg {
                None => None,
                Some(bg) => Self::get_random_sprite(bg),
            },
            Some(sprite) => match &sprite.ids.fg {
                None => None,
                Some(fg) => Self::get_random_additional_tile_sprite(
                    direction.clone(),
                    add_id.clone(),
                    sprite.rotates,
                    fg,
                ),
            },
        }
    }

    pub fn get_fg_id(
        &self,
        mapped_id: &MappedCDDAId,
        layer: &TileLayer,
        adjacent_sprites: &AdjacentSprites,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Rotated<MeabyVec<SpriteIndex>>> {
        match self {
            Sprite::Single {
                ids,
                animated,
                rotates,
            } => match *animated {
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
                ids,
                animated,
                center,
                corner,
                t_connection,
                edge,
                unconnected,
                end_piece,
                broken,
                open,
                rotates,
            } => match *animated {
                true => todo!(),
                false => {
                    if mapped_id.is_broken {
                        return match &broken.fg {
                            None => match &ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_sprite(fg),
                            },
                            Some(fg) => Self::get_random_sprite(fg),
                        };
                    }

                    if mapped_id.is_open {
                        return match &open.fg {
                            None => match &ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_sprite(fg),
                            },
                            Some(fg) => Self::get_random_sprite(fg),
                        };
                    }

                    let matching_list = Self::get_matching_list(
                        &mapped_id.tilesheet_id,
                        layer,
                        json_data,
                        adjacent_sprites,
                    );

                    match matching_list {
                        (true, true, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &North,
                                &Center,
                                ids,
                                center.as_ref(),
                            )
                        },
                        (true, true, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &East,
                                &TConnection,
                                ids,
                                t_connection.as_ref(),
                            )
                        },
                        (true, true, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &North,
                                &TConnection,
                                ids,
                                t_connection.as_ref(),
                            )
                        },
                        (true, false, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &West,
                                &TConnection,
                                ids,
                                t_connection.as_ref(),
                            )
                        },
                        (false, true, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &South,
                                &TConnection,
                                ids,
                                t_connection.as_ref(),
                            )
                        },
                        (true, true, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &North,
                                &Corner,
                                ids,
                                corner.as_ref(),
                            )
                        },
                        (true, false, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &West,
                                &Corner,
                                ids,
                                corner.as_ref(),
                            )
                        },
                        (false, true, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &East,
                                &Corner,
                                ids,
                                corner.as_ref(),
                            )
                        },
                        (false, false, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &South,
                                &Corner,
                                ids,
                                corner.as_ref(),
                            )
                        },
                        (true, false, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &North,
                                &EndPiece,
                                ids,
                                end_piece.as_ref(),
                            )
                        },
                        (false, true, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &East,
                                &EndPiece,
                                ids,
                                end_piece.as_ref(),
                            )
                        },
                        (false, false, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &South,
                                &EndPiece,
                                ids,
                                end_piece.as_ref(),
                            )
                        },
                        (false, false, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &West,
                                &EndPiece,
                                ids,
                                end_piece.as_ref(),
                            )
                        },
                        (false, true, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                &East,
                                &Edge,
                                ids,
                                edge.as_ref(),
                            )
                        },
                        (true, false, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &North,
                                &Edge,
                                ids,
                                edge.as_ref(),
                            )
                        },
                        (false, false, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                &North,
                                &Unconnected,
                                ids,
                                unconnected.as_ref(),
                            )
                        },
                    }
                },
            },
        }
    }

    pub fn get_bg_id(
        &self,
        mapped_id: &MappedCDDAId,
        layer: &TileLayer,
        adjacent_sprites: &AdjacentSprites,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Rotated<MeabyVec<SpriteIndex>>> {
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
                ids,
                animated,
                center,
                corner,
                t_connection,
                edge,
                unconnected,
                end_piece,
                broken,
                open,
                rotates,
            } => match *animated {
                true => todo!(),
                false => {
                    if mapped_id.is_broken {
                        return match &broken.bg {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(bg) => Self::get_random_sprite(bg),
                        };
                    }

                    if mapped_id.is_open {
                        return match &open.bg {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(bg) => Self::get_random_sprite(bg),
                        };
                    }

                    let matching_list = Self::get_matching_list(
                        &mapped_id.tilesheet_id,
                        layer,
                        json_data,
                        adjacent_sprites,
                    );

                    match matching_list {
                        (true, true, true, true) => match center {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(center) => match &center.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (true, true, true, false)
                        | (true, true, false, true)
                        | (true, false, true, true)
                        | (false, true, true, true) => match t_connection {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(t_connection) => match &t_connection.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (true, true, false, false)
                        | (true, false, false, true)
                        | (false, true, true, false)
                        | (false, false, true, true) => match corner {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(corner) => match &corner.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (true, false, false, false)
                        | (false, true, false, false)
                        | (false, false, true, false)
                        | (false, false, false, true) => match end_piece {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(end_piece) => match &end_piece.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (false, true, false, true)
                        | (true, false, true, false) => match edge {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(edge) => match &edge.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                        (false, false, false, false) => match unconnected {
                            None => match &ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                            Some(unconnected) => match &unconnected.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(bg),
                            },
                        },
                    }
                },
            },
        }
    }
}

#[derive(Debug)]
pub struct ForeBackIds<FG, BG> {
    pub fg: FG,
    pub bg: BG,
}

impl<FG, BG> ForeBackIds<FG, BG> {
    pub fn new(fg: FG, bg: BG) -> Self {
        Self { fg, bg }
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
        //let mut rng = RANDOM.write().unwrap();

        let chosen_index = weighted_index.sample(&mut rng);

        &self.get(chosen_index).unwrap().sprite
    }
}

impl<T> GetRandom<T> for Vec<Weighted<T>> {
    fn get_random(&self) -> &T {
        let mut weights = vec![];
        self.iter().for_each(|v| weights.push(v.weight));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");

        let mut rng = rng();
        //let mut rng = RANDOM.write().unwrap();

        let chosen_index = weighted_index.sample(&mut rng);

        &self.get(chosen_index).unwrap().data
    }
}

impl<T> GetRandom<T> for IndexMap<T, i32> {
    fn get_random(&self) -> &T {
        let mut weights = vec![];

        let mut vec = self.iter().collect::<Vec<(&T, &i32)>>();
        vec.iter().for_each(|(_, w)| weights.push(**w));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");

        let mut rng = rng();
        //let mut rng = RANDOM.write().unwrap();

        let chosen_index = weighted_index.sample(&mut rng);
        let item = vec.remove(chosen_index);

        &item.0
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SpriteLayer {
    Bg = 0,
    Fg = 1,
}

#[derive(Debug)]
pub struct AdjacentSprites {
    pub top: Option<CDDAIdentifier>,
    pub right: Option<CDDAIdentifier>,
    pub bottom: Option<CDDAIdentifier>,
    pub left: Option<CDDAIdentifier>,
}
