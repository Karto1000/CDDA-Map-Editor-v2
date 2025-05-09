use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::TileLayer;
use crate::tileset::current_tileset::CurrentTilesheet;
use crate::tileset::legacy_tileset::tile_config::AdditionalTileId;
use crate::tileset::legacy_tileset::{
    AdditionalTileIds, CardinalDirection, FinalIds, LegacyTilesheet, MappedCDDAIds, Rotated,
    Rotates, Rotation, SpriteIndex,
};
use crate::util::{CDDAIdentifier, MeabyVec, Weighted};
use glam::IVec3;
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub(crate) mod current_tileset;
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

pub enum TilesheetKind {
    Legacy(LegacyTilesheet),
    Current(CurrentTilesheet),
}

pub trait Tilesheet {
    fn get_sprite(&self, id: &CDDAIdentifier, json_data: &DeserializedCDDAJsonData) -> SpriteKind;
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
    Open {
        ids: ForeBackIds<FinalIds, FinalIds>,
        open: ForeBackIds<FinalIds, FinalIds>,
        rotates: bool,
        animated: bool,
    },
    Broken {
        ids: ForeBackIds<FinalIds, FinalIds>,
        broken: ForeBackIds<FinalIds, FinalIds>,
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

        let rotated = Rotated::none(MeabyAnimated::Single(ids.get_random().clone()));
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
            AdditionalTileId::Center | AdditionalTileId::Unconnected => Rotated {
                data: MeabyAnimated::Single(ids.get_random().get(&direction).clone()),
                rotation: Rotation::Deg0,
            },
            AdditionalTileId::Corner
            | AdditionalTileId::TConnection
            | AdditionalTileId::Edge
            | AdditionalTileId::EndPiece => match ids.get_random() {
                Rotates::Auto(a) => match rotates {
                    true => Rotated {
                        data: MeabyAnimated::Single(a.clone()),
                        rotation: Rotation::from(direction),
                    },
                    false => Rotated::none(MeabyAnimated::Single(a.clone())),
                },
                Rotates::Pre2(p) => match direction {
                    CardinalDirection::North => Rotated::none(MeabyAnimated::Single(p.0.clone())),
                    CardinalDirection::East => Rotated::none(MeabyAnimated::Single(p.1.clone())),
                    CardinalDirection::South => unreachable!(),
                    CardinalDirection::West => unreachable!(),
                },
                Rotates::Pre4(p) => match direction {
                    CardinalDirection::North => Rotated::none(MeabyAnimated::Single(p.0.clone())),
                    CardinalDirection::East => Rotated::none(MeabyAnimated::Single(p.1.clone())),
                    CardinalDirection::South => Rotated::none(MeabyAnimated::Single(p.2.clone())),
                    CardinalDirection::West => Rotated::none(MeabyAnimated::Single(p.3.clone())),
                },
            },
            _ => unreachable!(),
        };

        Some(rotated)
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
        adjacent_sprites: &AdjacentSprites,
    ) -> (bool, bool, bool, bool) {
        let mut this_connects_to = json_data.get_connects_to(Some(this_id.clone()), layer);
        let mut top_connect_groups =
            json_data.get_connect_groups(adjacent_sprites.top.clone(), layer);
        let mut right_connect_groups =
            json_data.get_connect_groups(adjacent_sprites.right.clone(), layer);
        let mut bottom_connect_groups =
            json_data.get_connect_groups(adjacent_sprites.bottom.clone(), layer);
        let mut left_connect_groups =
            json_data.get_connect_groups(adjacent_sprites.left.clone(), layer);

        let this_flags = json_data.get_flags(Some(this_id.clone()), layer);
        let top_flags = json_data.get_flags(adjacent_sprites.top.clone(), layer);
        let right_flags = json_data.get_flags(adjacent_sprites.right.clone(), layer);
        let bottom_flags = json_data.get_flags(adjacent_sprites.bottom.clone(), layer);
        let left_flags = json_data.get_flags(adjacent_sprites.left.clone(), layer);

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
            || this_id
            == &adjacent_sprites
            .top
            .clone()
            .unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_right = this_connects_to
            .intersection(&right_connect_groups)
            .next()
            .is_some()
            || this_id
            == &adjacent_sprites
            .right
            .clone()
            .unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_bottom = this_connects_to
            .intersection(&bottom_connect_groups)
            .next()
            .is_some()
            || this_id
            == &adjacent_sprites
            .bottom
            .clone()
            .unwrap_or(CDDAIdentifier("".to_string()));

        let can_connect_left = this_connects_to
            .intersection(&left_connect_groups)
            .next()
            .is_some()
            || this_id
            == &adjacent_sprites
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

    pub fn get_fg_id(
        &self,
        this_id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
        layer: &TileLayer,
        adjacent_sprites: &AdjacentSprites,
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
                    let matching_list =
                        Self::get_matching_list(this_id, layer, json_data, adjacent_sprites);

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
                                    AdditionalTileId::Center,
                                    center.rotates,
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
                                    AdditionalTileId::TConnection,
                                    t_connection.rotates,
                                    fg,
                                ),
                            },
                        },
                        (true, true, false, true) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    AdditionalTileId::TConnection,
                                    t_connection.rotates,
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
                                    AdditionalTileId::TConnection,
                                    t_connection.rotates,
                                    fg,
                                ),
                            },
                        },
                        (false, true, true, true) => match t_connection {
                            None => None,
                            Some(t_connection) => match &t_connection.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::South,
                                    AdditionalTileId::TConnection,
                                    t_connection.rotates,
                                    fg,
                                ),
                            },
                        },
                        (true, true, false, false) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    AdditionalTileId::Corner,
                                    corner.rotates,
                                    fg,
                                ),
                            },
                        },
                        (true, false, false, true) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::West,
                                    AdditionalTileId::Corner,
                                    corner.rotates,
                                    fg,
                                ),
                            },
                        },
                        (false, true, true, false) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::East,
                                    AdditionalTileId::Corner,
                                    corner.rotates,
                                    fg,
                                ),
                            },
                        },
                        (false, false, true, true) => match corner {
                            None => None,
                            Some(corner) => match &corner.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::South,
                                    AdditionalTileId::Corner,
                                    corner.rotates,
                                    fg,
                                ),
                            },
                        },
                        (true, false, false, false) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::North,
                                    AdditionalTileId::EndPiece,
                                    end_piece.rotates,
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
                                    AdditionalTileId::EndPiece,
                                    end_piece.rotates,
                                    fg,
                                ),
                            },
                        },
                        (false, false, true, false) => match end_piece {
                            None => None,
                            Some(end_piece) => match &end_piece.ids.fg {
                                None => None,
                                Some(fg) => Self::get_random_additional_tile_sprite(
                                    CardinalDirection::South,
                                    AdditionalTileId::EndPiece,
                                    end_piece.rotates,
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
                                    AdditionalTileId::EndPiece,
                                    end_piece.rotates,
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
                                    AdditionalTileId::Edge,
                                    edge.rotates,
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
                                    AdditionalTileId::Edge,
                                    edge.rotates,
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
                                    AdditionalTileId::Unconnected,
                                    unconnected.rotates,
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
        adjacent_sprites: &AdjacentSprites,
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
                    let matching_list =
                        Self::get_matching_list(this_id, layer, json_data, adjacent_sprites);

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

pub fn get_id_from_mapped_sprites(
    mapped_cdda_ids: &HashMap<IVec3, MappedCDDAIds>,
    cords: &IVec3,
    layer: &TileLayer,
) -> Option<CDDAIdentifier> {
    mapped_cdda_ids
        .get(cords)
        .map(|v| match layer {
            TileLayer::Terrain => v.terrain.clone(),
            TileLayer::Furniture => v.furniture.clone(),
            TileLayer::Monster => v.monster.clone(),
            TileLayer::Field => v.field.clone(),
        })
        .flatten()
}

pub fn get_adjacent_sprites(
    mapped_cdda_ids: &HashMap<IVec3, MappedCDDAIds>,
    coordinates: IVec3,
    layer: &TileLayer,
) -> AdjacentSprites {
    let top_cords = coordinates + IVec3::new(0, 1, 0);
    let top = get_id_from_mapped_sprites(&mapped_cdda_ids, &top_cords, &layer);

    let right_cords = coordinates + IVec3::new(1, 0, 0);
    let right = get_id_from_mapped_sprites(&mapped_cdda_ids, &right_cords, &layer);

    let bottom = match coordinates.y > 0 {
        true => {
            let bottom_cords = coordinates - IVec3::new(0, 1, 0);
            get_id_from_mapped_sprites(&mapped_cdda_ids, &bottom_cords, &layer)
        }
        false => None,
    };

    let left = match coordinates.x > 0 {
        true => {
            let left_cords = coordinates - IVec3::new(1, 0, 0);
            get_id_from_mapped_sprites(&mapped_cdda_ids, &left_cords, &layer)
        }
        false => None,
    };

    AdjacentSprites {
        top,
        right,
        bottom,
        left,
    }
}

#[derive(Debug)]
pub struct AdjacentSprites {
    pub top: Option<CDDAIdentifier>,
    pub right: Option<CDDAIdentifier>,
    pub bottom: Option<CDDAIdentifier>,
    pub left: Option<CDDAIdentifier>,
}
