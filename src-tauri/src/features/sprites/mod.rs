pub mod handlers;

use crate::data::io::DeserializedCDDAJsonData;
use crate::data::TileLayer;
use crate::features::map::{
    InstantiatedOvermapStack, MappedCDDAId, MAX_MAP_DATA_SIZE,
};
use crate::features::program_data::ZLevel;
use crate::features::tileset::legacy_tileset::{
    FallbackSpriteIndex, Rotated, SpriteIndex, Tilesheet,
};
use crate::features::tileset::{
    GetSprite, PickRandomSpriteFromMappedIdContext, PickSpriteIndex,
    RandomSpritePicker, Sprite, SpriteLayer,
};
use cdda_lib::types::MeabyVec;
use glam::UVec2;
use rand::rng;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use strum::IntoEnumIterator;

pub fn instantiate_sprite(
    id: &MappedCDDAId,
    sprite: &Sprite,
    sprite_layer: SpriteLayer,
    indices: Rotated<MeabyVec<SpriteIndex>>,
    position: &UVec2,
    z: ZLevel,
    tile_layer: TileLayer,
) -> InstancedSprite {
    match sprite.is_animated() {
        true => {
            let display_sprite = InstancedAnimatedSprite {
                position: position.clone(),
                layer: (tile_layer.clone() as u32) * 2 + sprite_layer as u32,
                indices: indices.data.into_vec(),
                rotate_deg: indices.rotation.deg() + id.rotation.deg(),
                z,
            };

            InstancedSprite::Animated(display_sprite)
        },
        false => {
            let display_sprite = InstancedStaticSprite {
                position: position.clone(),
                layer: (tile_layer.clone() as u32) * 2 + sprite_layer as u32,
                index: indices.data.into_single().unwrap(),
                rotate_deg: indices.rotation.deg(),
                z,
            };

            InstancedSprite::Static(display_sprite)
        },
    }
}

pub fn get_sprites_from_instantiated_overmaps_stack(
    instantiated_overmap_stack: &InstantiatedOvermapStack,
    tilesheet: Option<&Tilesheet>,
    fallback_tilesheet: &Tilesheet,
    cdda_data: &DeserializedCDDAJsonData,
) -> InstancedSprites {
    let mut instanced_sprites: InstancedSprites = InstancedSprites::default();

    let mut temp_rng = rng();
    let mut random_sprite_picker = RandomSpritePicker::new(&mut temp_rng);

    for (z, overmap) in instantiated_overmap_stack.instantiated_overmaps.iter()
    {
        for (overmap_coordinates, mapgen) in overmap.instantiated_mapgens.iter()
        {
            let instanced_tiles = mapgen
                .instantiated_tiles
                .iter()
                .map(|(tile_coordinates, tile)| {
                    let tile_overmap_coordinates = tile_coordinates.uvec()
                        + overmap_coordinates * MAX_MAP_DATA_SIZE;

                    let mut instanced_sprites_for_tile: HashMap<
                        TileLayer,
                        (Option<InstancedSprite>, Option<InstancedSprite>),
                    > = HashMap::new();

                    for (tile_layer, mapped_id) in [
                        (TileLayer::Terrain, &tile.terrain),
                        (TileLayer::Furniture, &tile.furniture),
                        (TileLayer::Monster, &tile.monster),
                        (TileLayer::Field, &tile.field),
                    ] {
                        let id = match mapped_id {
                            None => continue,
                            Some(mapped_id) => mapped_id,
                        };

                        match tilesheet {
                            None => {
                                let fallback_index = fallback_tilesheet
                                    .get_fallback(id, cdda_data);

                                let instanced_fallback =
                                    InstancedFallbackSprite {
                                        position: tile_overmap_coordinates,
                                        index: *fallback_index,
                                        z: *z,
                                    };

                                instanced_sprites_for_tile.insert(
                                    tile_layer,
                                    (
                                        None,
                                        Some(InstancedSprite::Fallback(
                                            instanced_fallback,
                                        )),
                                    ),
                                );
                            }
                            Some(t) => match t.get_sprite(id, cdda_data) {
                                None => {
                                    let fallback_index =
                                        t.get_fallback(id, cdda_data);

                                    let instanced_fallback =
                                        InstancedFallbackSprite {
                                            position: tile_overmap_coordinates,
                                            index: *fallback_index,
                                            z: *z,
                                        };

                                    instanced_sprites_for_tile.insert(
                                        tile_layer,
                                        (
                                            None,
                                            Some(InstancedSprite::Fallback(
                                                instanced_fallback,
                                            )),
                                        ),
                                    );
                                }
                                Some(sprite) => {
                                    let adjacent = overmap
                                        .get_adjacent_tile_identifiers(
                                            &tile_overmap_coordinates,
                                            &tile_layer,
                                        );

                                    let fg = random_sprite_picker.pick(
                                        PickRandomSpriteFromMappedIdContext::new(id.clone(), adjacent.clone()),
                                        sprite,
                                        SpriteLayer::Fg,
                                        tile_layer,
                                        cdda_data,
                                    ).map(|indices| instantiate_sprite(
                                        id,
                                        sprite,
                                        SpriteLayer::Fg,
                                        indices,
                                        &tile_overmap_coordinates,
                                        *z,
                                        tile_layer,
                                    ));

                                    let bg = random_sprite_picker.pick(
                                        PickRandomSpriteFromMappedIdContext::new(id.clone(), adjacent.clone()),
                                        sprite,
                                        SpriteLayer::Bg,
                                        tile_layer,
                                        cdda_data,
                                    ).map(|indices| instantiate_sprite(
                                        id,
                                        sprite,
                                        SpriteLayer::Bg,
                                        indices,
                                        &tile_overmap_coordinates,
                                        *z,
                                        tile_layer,
                                    ));

                                    instanced_sprites_for_tile
                                        .insert(tile_layer, (fg, bg));
                                }
                            },
                        }
                    }

                    instanced_sprites_for_tile
                })
                .collect::<Vec<_>>();

            for mut tile in instanced_tiles {
                for layer in TileLayer::iter() {
                    let (fg, bg) = match tile.remove(&layer) {
                        None => continue,
                        Some((fg, bg)) => (fg, bg),
                    };

                    if let Some(fg) = fg {
                        instanced_sprites.insert_instanced_sprite(fg);
                    }

                    if let Some(bg) = bg {
                        instanced_sprites.insert_instanced_sprite(bg);
                    }
                }
            }
        }
    }

    instanced_sprites
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InstancedSprites {
    pub static_sprites: HashSet<InstancedStaticSprite>,
    pub animated_sprites: HashSet<InstancedAnimatedSprite>,
    pub fallback_sprites: HashSet<InstancedFallbackSprite>,
}

impl InstancedSprites {
    pub fn insert_instanced_sprite(
        &mut self,
        instanced_sprite: InstancedSprite,
    ) {
        match instanced_sprite {
            InstancedSprite::Static(s) => {
                self.static_sprites.insert(s);
            },
            InstancedSprite::Animated(a) => {
                self.animated_sprites.insert(a);
            },
            InstancedSprite::Fallback(f) => {
                self.fallback_sprites.insert(f);
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstancedStaticSprite {
    pub position: UVec2,
    pub index: SpriteIndex,
    pub layer: u32,
    pub z: i32,
    pub rotate_deg: i32,
}

impl Hash for InstancedStaticSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.layer.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq<Self> for InstancedStaticSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
            && self.layer.eq(&other.layer)
            && self.z.eq(&other.z)
    }
}

impl Eq for InstancedStaticSprite {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstancedAnimatedSprite {
    pub position: UVec2,
    pub indices: Vec<SpriteIndex>,
    pub layer: u32,
    pub z: i32,
    pub rotate_deg: i32,
}

impl Hash for InstancedAnimatedSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.layer.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq for InstancedAnimatedSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
            && self.layer.eq(&other.layer)
            && self.z.eq(&other.z)
    }
}

impl Eq for InstancedAnimatedSprite {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstancedFallbackSprite {
    pub position: UVec2,
    pub index: FallbackSpriteIndex,
    pub z: i32,
}

impl Hash for InstancedFallbackSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq for InstancedFallbackSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position) && self.z.eq(&other.z)
    }
}

impl Eq for InstancedFallbackSprite {}

#[derive(Debug)]
pub enum InstancedSprite {
    Static(InstancedStaticSprite),
    Animated(InstancedAnimatedSprite),
    Fallback(InstancedFallbackSprite),
}
