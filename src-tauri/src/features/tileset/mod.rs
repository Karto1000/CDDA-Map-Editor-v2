mod data;
pub mod handlers;
pub mod legacy_tileset;

use crate::data::io::DeserializedCDDAJsonData;
use crate::data::TileLayer;
use crate::features::map::MappedCDDAId;
use crate::features::program_data::AdjacentSprites;
use crate::features::tileset::data::AdditionalTileType;
use crate::features::tileset::data::AdditionalTileType::{
    Center, Corner, Edge, EndPiece, TConnection, Unconnected,
};
use crate::features::tileset::legacy_tileset::{
    FinalIds, Rotated, Rotates, SpriteIndex, TilesheetCDDAId,
};
use crate::util::CardinalDirection::{East, North, South, West};
use crate::util::{CardinalDirection, GetRandom, Rotation};
use cdda_lib::types::{CDDAIdentifier, MeabyVec, Weighted};
use data::MeabyAnimated;
use rand::distr::Distribution;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub(super) trait Tilesheet {
    fn get_fallback(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> SpriteIndex;

    fn get_sprite(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<&Sprite>;
}

#[derive(Debug)]
pub(super) struct SingleSprite {
    ids: ForeBackIds<FinalIds, FinalIds>,
    rotates: bool,
    animated: bool,
}

#[derive(Debug)]
pub(super) enum Sprite {
    Single(SingleSprite),
    Multitile {
        fallback: SingleSprite,
        edge: Option<SingleSprite>,
        corner: Option<SingleSprite>,
        center: Option<SingleSprite>,
        t_connection: Option<SingleSprite>,
        end_piece: Option<SingleSprite>,
        unconnected: Option<SingleSprite>,
        broken: Option<SingleSprite>,
        open: Option<SingleSprite>,
    },
}

impl Sprite {
    pub fn is_animated(&self) -> bool {
        match self {
            Sprite::Single(single) => single.animated.clone(),
            Sprite::Multitile { fallback, .. } => fallback.animated.clone(),
        }
    }

    fn get_random_animated_sprite(
        mapped_id: &MappedCDDAId,
        tilesheet_ids: &Vec<Weighted<Rotates>>,
        rotates: bool,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        if tilesheet_ids.len() == 0 {
            return None;
        }

        let mut indices = Vec::new();

        for rotates_id in tilesheet_ids.to_vec() {
            let (index, _) = Self::get_sprite_index_from_rotates(
                mapped_id,
                rotates_id.data,
                rotates,
            );

            indices.push(index);
        }

        match rotates {
            true => Some(Rotated {
                rotation: mapped_id.rotation.clone(),
                data: MeabyVec::Vec(indices),
            }),
            false => Some(Rotated::none(MeabyAnimated::Vec(indices))),
        }
    }

    fn get_sprite_index_from_rotates(
        mapped_id: &MappedCDDAId,
        rotates: Rotates,
        does_rotates: bool,
    ) -> (SpriteIndex, Rotation) {
        match rotates {
            Rotates::Auto(i) => match does_rotates {
                false => (i, Rotation::Deg0),
                true => (i, mapped_id.rotation.clone()),
            },
            Rotates::Pre2((a, b)) => {
                let chosen_index = match mapped_id.rotation {
                    // TODO: I don't know if these are actually the same or if this is different
                    Rotation::Deg0 | Rotation::Deg180 => a,
                    Rotation::Deg90 | Rotation::Deg270 => b,
                };

                (chosen_index, Rotation::Deg0)
            },
            Rotates::Pre4((a, b, c, d)) => {
                let chosen_index = match mapped_id.rotation {
                    Rotation::Deg0 => a,
                    Rotation::Deg90 => b,
                    Rotation::Deg180 => c,
                    Rotation::Deg270 => d,
                };

                (chosen_index, Rotation::Deg0)
            },
        }
    }

    fn get_random_sprite(
        mapped_id: &MappedCDDAId,
        tilesheet_ids: &Vec<Weighted<Rotates>>,
        rotates: bool,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        if tilesheet_ids.len() == 0 {
            return None;
        }

        let random_id = tilesheet_ids.get_random().clone();
        let (random_index, rotation) = Self::get_sprite_index_from_rotates(
            mapped_id,
            random_id.clone(),
            rotates,
        );

        Some(Rotated {
            rotation,
            data: MeabyAnimated::Single(random_index),
        })
    }

    fn get_random_additional_tile_sprite(
        mapped_id: &MappedCDDAId,
        tilesheet_ids: &Vec<Weighted<Rotates>>,
        additional_ids: &Vec<Weighted<Rotates>>,
        direction: CardinalDirection,
        additional_tile_type: AdditionalTileType,
        does_rotate: bool,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        if additional_ids.len() == 0 {
            return None;
        }

        let rotated = match additional_tile_type {
            Center | Unconnected => {
                let random_id = MeabyAnimated::Single(
                    additional_ids.get_random().get(&direction).clone(),
                );

                match does_rotate {
                    true => Rotated {
                        data: random_id,
                        rotation: mapped_id.rotation.clone(),
                    },
                    false => Rotated::none(random_id),
                }
            },
            Corner | TConnection | Edge | EndPiece => match additional_ids
                .get_random()
            {
                Rotates::Auto(a) => match does_rotate {
                    true => Rotated {
                        data: MeabyAnimated::Single(a.clone()),
                        rotation: Rotation::from(direction)
                            + mapped_id.rotation.clone(),
                    },
                    false => Rotated::none(MeabyAnimated::Single(a.clone())),
                },
                Rotates::Pre2(p) => match does_rotate {
                    true => match direction {
                        North => Rotated::new(
                            MeabyAnimated::Single(p.0.clone()),
                            mapped_id.rotation.clone(),
                        ),
                        East => Rotated::new(
                            MeabyAnimated::Single(p.1.clone()),
                            mapped_id.rotation.clone(),
                        ),
                        // TODO: Don't know if this is correct
                        South => Self::get_random_sprite(
                            mapped_id,
                            tilesheet_ids,
                            does_rotate,
                        )?,
                        West => Self::get_random_sprite(
                            mapped_id,
                            tilesheet_ids,
                            does_rotate,
                        )?,
                    },
                    false => match direction {
                        North => {
                            Rotated::none(MeabyAnimated::Single(p.0.clone()))
                        },
                        East => {
                            Rotated::none(MeabyAnimated::Single(p.1.clone()))
                        },
                        South => Self::get_random_sprite(
                            mapped_id,
                            tilesheet_ids,
                            does_rotate,
                        )?,
                        West => Self::get_random_sprite(
                            mapped_id,
                            tilesheet_ids,
                            does_rotate,
                        )?,
                    },
                },
                Rotates::Pre4(p) => match does_rotate {
                    true => match direction {
                        North => Rotated::new(
                            MeabyAnimated::Single(p.0.clone()),
                            mapped_id.rotation.clone(),
                        ),
                        East => Rotated::new(
                            MeabyAnimated::Single(p.1.clone()),
                            mapped_id.rotation.clone(),
                        ),
                        South => Rotated::new(
                            MeabyAnimated::Single(p.2.clone()),
                            mapped_id.rotation.clone(),
                        ),
                        West => Rotated::new(
                            MeabyAnimated::Single(p.3.clone()),
                            mapped_id.rotation.clone(),
                        ),
                    },
                    false => match direction {
                        North => {
                            Rotated::none(MeabyAnimated::Single(p.0.clone()))
                        },
                        East => {
                            Rotated::none(MeabyAnimated::Single(p.1.clone()))
                        },
                        South => {
                            Rotated::none(MeabyAnimated::Single(p.2.clone()))
                        },
                        West => {
                            Rotated::none(MeabyAnimated::Single(p.3.clone()))
                        },
                    },
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
        mapped_id: &MappedCDDAId,
        fallback_ids: &ForeBackIds<FinalIds, FinalIds>,
        direction: &CardinalDirection,
        additional_tile_type: &AdditionalTileType,
        multitile_sprite: Option<&SingleSprite>,
        does_rotate: bool,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        match multitile_sprite {
            None => match &fallback_ids.fg {
                None => None,
                Some(fg) => Self::get_random_sprite(mapped_id, fg, does_rotate),
            },
            Some(sprite) => match &sprite.ids.fg {
                None => None,
                Some(fg) => {
                    let fg_ids = match &fallback_ids.fg {
                        None => return None,
                        Some(fg_ids) => fg_ids,
                    };

                    Self::get_random_additional_tile_sprite(
                        mapped_id,
                        fg_ids,
                        fg,
                        direction.clone(),
                        additional_tile_type.clone(),
                        sprite.rotates,
                    )
                },
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
            Sprite::Single(s) => match s.animated {
                true => match &s.ids.fg {
                    None => None,
                    Some(fg) => Self::get_random_animated_sprite(
                        mapped_id, fg, s.rotates,
                    ),
                },
                false => match &s.ids.fg {
                    None => None,
                    Some(fg) => {
                        Self::get_random_sprite(mapped_id, fg, s.rotates)
                    },
                },
            },
            Sprite::Multitile {
                fallback,
                center,
                corner,
                t_connection,
                edge,
                unconnected,
                end_piece,
                broken,
                open,
            } => match fallback.animated {
                true => todo!(),
                false => {
                    if mapped_id.is_broken {
                        return match broken {
                            None => {
                                return None;
                            },
                            Some(broken) => match &broken.ids.fg {
                                None => match &fallback.ids.fg {
                                    None => None,
                                    Some(fg) => Self::get_random_sprite(
                                        mapped_id,
                                        fg,
                                        fallback.rotates,
                                    ),
                                },
                                Some(fg) => Self::get_random_sprite(
                                    mapped_id,
                                    fg,
                                    fallback.rotates,
                                ),
                            },
                        };
                    }

                    if mapped_id.is_open {
                        return match open {
                            None => {
                                return None;
                            },
                            Some(open) => match &open.ids.fg {
                                None => match &fallback.ids.fg {
                                    None => None,
                                    Some(fg) => Self::get_random_sprite(
                                        mapped_id,
                                        fg,
                                        fallback.rotates,
                                    ),
                                },
                                Some(fg) => Self::get_random_sprite(
                                    mapped_id,
                                    fg,
                                    fallback.rotates,
                                ),
                            },
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
                                mapped_id,
                                &fallback.ids,
                                &North,
                                &Center,
                                center.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, true, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &East,
                                &TConnection,
                                t_connection.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, true, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &North,
                                &TConnection,
                                t_connection.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, false, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &West,
                                &TConnection,
                                t_connection.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, true, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &South,
                                &TConnection,
                                t_connection.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, true, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &North,
                                &Corner,
                                corner.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, false, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &West,
                                &Corner,
                                corner.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, true, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &East,
                                &Corner,
                                corner.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, false, true, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &South,
                                &Corner,
                                corner.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, false, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &North,
                                &EndPiece,
                                end_piece.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, true, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &East,
                                &EndPiece,
                                end_piece.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, false, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &South,
                                &EndPiece,
                                end_piece.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, false, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &West,
                                &EndPiece,
                                end_piece.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, true, false, true) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &East,
                                &Edge,
                                edge.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (true, false, true, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &North,
                                &Edge,
                                edge.as_ref(),
                                fallback.rotates,
                            )
                        },
                        (false, false, false, false) => {
                            Self::get_sprite_from_multitile_sprite(
                                mapped_id,
                                &fallback.ids,
                                &North,
                                &Unconnected,
                                unconnected.as_ref(),
                                fallback.rotates,
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
            Sprite::Single(single) => match single.animated {
                true => match &single.ids.bg {
                    None => None,
                    Some(bg) => Self::get_random_animated_sprite(
                        mapped_id,
                        bg,
                        single.rotates,
                    ),
                },
                false => match &single.ids.bg {
                    None => None,
                    Some(bg) => {
                        Self::get_random_sprite(mapped_id, bg, single.rotates)
                    },
                },
            },
            Sprite::Multitile {
                fallback,
                center,
                corner,
                t_connection,
                edge,
                unconnected,
                end_piece,
                broken,
                open,
            } => match fallback.animated {
                true => todo!(),
                false => {
                    let random_fallback_sprite = match &fallback.ids.bg {
                        None => None,
                        Some(bg) => Self::get_random_sprite(
                            mapped_id,
                            bg,
                            fallback.rotates,
                        ),
                    };

                    if mapped_id.is_broken {
                        return match broken {
                            None => return None,
                            Some(broken) => match &broken.ids.bg {
                                None => random_fallback_sprite,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        };
                    }

                    if mapped_id.is_open {
                        return match open {
                            None => return None,
                            Some(open) => match &open.ids.bg {
                                None => random_fallback_sprite,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
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
                            None => random_fallback_sprite,
                            Some(center) => match &center.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        },
                        (true, true, true, false)
                        | (true, true, false, true)
                        | (true, false, true, true)
                        | (false, true, true, true) => match t_connection {
                            None => random_fallback_sprite,
                            Some(t_connection) => match &t_connection.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        },
                        (true, true, false, false)
                        | (true, false, false, true)
                        | (false, true, true, false)
                        | (false, false, true, true) => match corner {
                            None => random_fallback_sprite,
                            Some(corner) => match &corner.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        },
                        (true, false, false, false)
                        | (false, true, false, false)
                        | (false, false, true, false)
                        | (false, false, false, true) => match end_piece {
                            None => random_fallback_sprite,
                            Some(end_piece) => match &end_piece.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        },
                        (false, true, false, true)
                        | (true, false, true, false) => match edge {
                            None => random_fallback_sprite,
                            Some(edge) => match &edge.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        },
                        (false, false, false, false) => match unconnected {
                            None => random_fallback_sprite,
                            Some(unconnected) => match &unconnected.ids.bg {
                                None => None,
                                Some(bg) => Self::get_random_sprite(
                                    mapped_id,
                                    bg,
                                    fallback.rotates,
                                ),
                            },
                        },
                    }
                },
            },
        }
    }
}

#[derive(Debug)]
pub(super) struct ForeBackIds<FG, BG> {
    pub fg: FG,
    pub bg: BG,
}

impl<FG, BG> ForeBackIds<FG, BG> {
    pub fn new(fg: FG, bg: BG) -> Self {
        Self { fg, bg }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) enum SpriteLayer {
    Bg = 0,
    Fg = 1,
}
