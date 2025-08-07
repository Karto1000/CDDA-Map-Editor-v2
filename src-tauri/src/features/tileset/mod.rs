mod data;
pub mod handlers;
pub mod legacy_tileset;

use crate::data::io::DeserializedCDDAJsonData;
use crate::data::TileLayer;
use crate::features::map::MappedCDDAId;
use crate::features::program_data::AdjacentTiles;
use crate::features::tileset::legacy_tileset::{
    FallbackSpriteIndex, FinalIds, Rotated, Rotates, SpriteIndex,
};
use crate::util::{GetRandom, Rotation};
use cdda_lib::types::{CDDAIdentifier, Weighted};
use data::MeabyAnimated;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait GetSprite {
    fn get_fallback(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> &FallbackSpriteIndex;

    fn get_sprite(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<&Sprite>;
}

#[derive(Debug, Clone)]
pub struct SingleSprite {
    ids: FgBgIds<FinalIds, FinalIds>,
    rotates: bool,
    animated: bool,
}

#[derive(Debug)]
pub enum Sprite {
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

pub trait PickSpriteIndex {
    type Context;

    fn pick(
        &mut self,
        context: Self::Context,
        sprite: &Sprite,
        sprite_layer: SpriteLayer,
        tile_layer: TileLayer,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>>;
}

fn add_connections_from_flags(
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

fn does_id_connect(
    from: &MappedCDDAId,
    to: &Option<MappedCDDAId>,
    layer: TileLayer,
    json_data: &DeserializedCDDAJsonData,
) -> bool {
    let mut this_connects_to = json_data
        .get_connects_to(from.tilesheet_id.id.clone(), &layer)
        .unwrap_or_default();

    let this_flags = json_data
        .get_flags(from.tilesheet_id.id.clone(), &layer)
        .unwrap_or_default();

    let (mut to_connect_groups, to_flags) = match to.clone() {
        None => (HashSet::new(), Vec::new()),
        Some(to_tile) => (
            json_data
                .get_connect_groups(to_tile.tilesheet_id.id.clone(), &layer)
                .unwrap_or_default(),
            json_data
                .get_flags(to_tile.tilesheet_id.id, &layer)
                .unwrap_or_default(),
        ),
    };

    add_connections_from_flags(&this_flags, &mut this_connects_to);
    add_connections_from_flags(&to_flags, &mut to_connect_groups);

    let can_connect = this_connects_to
        .intersection(&to_connect_groups)
        .next()
        // We have the second check here since the tile can also connect to itself
        // TODO: I think there's a no self connect flag to toggle this behaviour
        // although im not sure
        .is_some()
        || from.tilesheet_id.id
            == to
                .clone()
                .map(|t| t.tilesheet_id.id)
                .clone()
                .unwrap_or(CDDAIdentifier("".to_string()));

    can_connect
}

pub struct RandomSpritePicker<'a> {
    rng: &'a mut dyn RngCore,
}

impl<'a> RandomSpritePicker<'a> {
    pub fn new(rng: &'a mut dyn RngCore) -> Self {
        Self { rng }
    }

    fn pick_random_sprite_from_indices(
        &mut self,
        context: &PickRandomSpriteFromMappedIdContext,
        indices: &Vec<Weighted<Rotates>>,
        can_rotate: bool,
    ) -> Rotated<MeabyAnimated<SpriteIndex>> {
        let layer_index = indices.get_random(self.rng);

        let (index, rotation) = layer_index.get_index_with_rotation(
            context.mapped_cdda_id.rotation,
            Rotation::Deg0,
            can_rotate,
        );

        Rotated::new(MeabyAnimated::Single(index), rotation)
    }
}

pub struct PickRandomSpriteFromMappedIdContext {
    mapped_cdda_id: MappedCDDAId,
    adjacent_tiles: AdjacentTiles,
}

impl PickRandomSpriteFromMappedIdContext {
    pub fn new(
        mapped_cdda_id: MappedCDDAId,
        adjacent_tiles: AdjacentTiles,
    ) -> Self {
        Self {
            mapped_cdda_id,
            adjacent_tiles,
        }
    }
}

impl<'a> PickSpriteIndex for RandomSpritePicker<'a> {
    type Context = PickRandomSpriteFromMappedIdContext;

    fn pick(
        &mut self,
        context: Self::Context,
        sprite: &Sprite,
        sprite_layer: SpriteLayer,
        tile_layer: TileLayer,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        match sprite {
            Sprite::Single(s) => match s.animated {
                true => {
                    let layer_indices =
                        match s.ids.of_sprite_layer(sprite_layer) {
                            None => return None,
                            Some(i) => i,
                        };

                    if layer_indices.len() == 0 {
                        return None;
                    }

                    let final_indices: Vec<SpriteIndex> = layer_indices
                        .iter()
                        .map(|index| {
                            index
                                .data
                                .get_index_with_rotation(
                                    context.mapped_cdda_id.rotation,
                                    Rotation::Deg0,
                                    s.rotates,
                                )
                                // We don't care about the rotation of individual sprites so only get
                                // the index
                                .0
                        })
                        .collect();

                    let rotated = match s.rotates {
                        true => Rotated::new(
                            MeabyAnimated::Vec(final_indices),
                            context.mapped_cdda_id.rotation,
                        ),
                        false => {
                            Rotated::none(MeabyAnimated::Vec(final_indices))
                        },
                    };

                    Some(rotated)
                },
                false => {
                    let layer_indices =
                        match s.ids.of_sprite_layer(sprite_layer) {
                            None => return None,
                            // Here we want to choose a random sprite of the selection since we know
                            // that this sprite isn't animated
                            Some(i) => i,
                        };

                    if layer_indices.len() == 0 {
                        return None;
                    }

                    let picked_index = self.pick_random_sprite_from_indices(
                        &context,
                        layer_indices,
                        s.rotates,
                    );

                    Some(picked_index)
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
            } => {
                let random_fallback_index = fallback
                    .clone()
                    .ids
                    .fg
                    .map(|fg| {
                        Some(self.pick_random_sprite_from_indices(
                            &context,
                            &fg,
                            fallback.rotates,
                        ))
                    })
                    .flatten();

                if context.mapped_cdda_id.is_broken {
                    return match broken {
                        None => random_fallback_index,
                        Some(broken) => match &broken.ids.fg {
                            None => None,
                            Some(indices) => {
                                Some(self.pick_random_sprite_from_indices(
                                    &context,
                                    indices,
                                    broken.rotates,
                                ))
                            },
                        },
                    };
                }

                if context.mapped_cdda_id.is_open {
                    return match open {
                        None => random_fallback_index,
                        Some(open) => match &open.ids.fg {
                            None => None,
                            Some(indices) => {
                                Some(self.pick_random_sprite_from_indices(
                                    &context,
                                    indices,
                                    open.rotates,
                                ))
                            },
                        },
                    };
                }

                let matching_list = (
                    does_id_connect(
                        &context.mapped_cdda_id,
                        &context.adjacent_tiles.top,
                        tile_layer,
                        json_data,
                    ),
                    does_id_connect(
                        &context.mapped_cdda_id,
                        &context.adjacent_tiles.right,
                        tile_layer,
                        json_data,
                    ),
                    does_id_connect(
                        &context.mapped_cdda_id,
                        &context.adjacent_tiles.bottom,
                        tile_layer,
                        json_data,
                    ),
                    does_id_connect(
                        &context.mapped_cdda_id,
                        &context.adjacent_tiles.left,
                        tile_layer,
                        json_data,
                    ),
                );

                let (sprite, rotation) = match matching_list {
                    // .o.
                    // oOo
                    // .o.
                    (true, true, true, true) => match center {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg0),
                    },
                    // .o.
                    // oOo
                    // ...
                    (true, true, false, true) => match t_connection {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg0),
                    },
                    // .o.
                    // .Oo
                    // .o.
                    (true, true, true, false) => match t_connection {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg90),
                    },
                    // ...
                    // oOo
                    // .o.
                    (false, true, true, true) => match t_connection {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg180),
                    },
                    // .o.
                    // oO.
                    // .o.
                    (true, false, true, true) => match t_connection {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg270),
                    },
                    // TODO: These corner rotations make no sense
                    // .o.
                    // .Oo
                    // ...
                    (true, true, false, false) => match corner {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg0),
                    },
                    // ...
                    // .Oo
                    // .o.
                    (false, true, true, false) => match corner {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg90),
                    },
                    // ...
                    // oO.
                    // .o.
                    (false, false, true, true) => match corner {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg180),
                    },
                    // .o.
                    // oO.
                    // ...
                    (true, false, false, true) => match corner {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg270),
                    },
                    // .o.
                    // .O.
                    // ...
                    (true, false, false, false) => match end_piece {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg0),
                    },
                    // ...
                    // .Oo
                    // ...
                    (false, true, false, false) => match end_piece {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg90),
                    },
                    // ...
                    // .O.
                    // .o.
                    (false, false, true, false) => match end_piece {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg180),
                    },
                    // ...
                    // oO.
                    // ...
                    (false, false, false, true) => match end_piece {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg270),
                    },
                    // .o.
                    // .O.
                    // .o.
                    (true, false, true, false) => match edge {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg0),
                    },
                    // ...
                    // oOo
                    // ...
                    (false, true, false, true) => match edge {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg90),
                    },
                    // ...
                    // .O.
                    // ...
                    (false, false, false, false) => match unconnected {
                        None => return random_fallback_index,
                        Some(sprite) => (sprite, Rotation::Deg0),
                    },
                };

                match &sprite.ids.of_sprite_layer(sprite_layer) {
                    None => None,
                    Some(indices) => {
                        let random_index = indices.get_random(self.rng);
                        let (index, rotation) = random_index
                            .get_index_with_rotation(
                                context.mapped_cdda_id.rotation,
                                rotation,
                                sprite.rotates,
                            );

                        Some(Rotated::new(
                            MeabyAnimated::Single(index),
                            rotation,
                        ))
                    },
                }
            },
        }
    }
}

pub struct RepresentativeSpritePicker;

impl PickSpriteIndex for RepresentativeSpritePicker {
    type Context = ();

    fn pick(
        &mut self,
        _context: Self::Context,
        sprite: &Sprite,
        sprite_layer: SpriteLayer,
        _tile_layer: TileLayer,
        _json_data: &DeserializedCDDAJsonData,
    ) -> Option<Rotated<MeabyAnimated<SpriteIndex>>> {
        let single_sprite = match sprite {
            Sprite::Single(s) => s,
            Sprite::Multitile { fallback, .. } => fallback,
        };

        match single_sprite.ids.of_sprite_layer(sprite_layer) {
            None => None,
            Some(ids) => {
                let first_element = &ids.first()?.data;

                let first_index = match first_element {
                    Rotates::Auto(a) => a,
                    Rotates::Pre2((a, _)) => a,
                    Rotates::Pre4((a, _, _, _)) => a,
                };

                Some(Rotated::none(MeabyAnimated::Single(*first_index)))
            },
        }
    }
}

impl Sprite {
    pub fn is_animated(&self) -> bool {
        match self {
            Sprite::Single(single) => single.animated.clone(),
            Sprite::Multitile { fallback, .. } => fallback.animated.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FgBgIds<FG, BG> {
    pub fg: FG,
    pub bg: BG,
}

impl<FG, BG> FgBgIds<FG, BG> {
    pub fn new(fg: FG, bg: BG) -> Self {
        Self { fg, bg }
    }
}

impl<Ids> FgBgIds<Ids, Ids> {
    pub fn of_sprite_layer(&self, layer: SpriteLayer) -> &Ids {
        match layer {
            SpriteLayer::Bg => &self.bg,
            SpriteLayer::Fg => &self.fg,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum SpriteLayer {
    Bg = 0,
    Fg = 1,
}
