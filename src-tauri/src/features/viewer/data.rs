use crate::data::io::DeserializedCDDAJsonData;
use crate::data::TileLayer;
use crate::features::map::MappedCDDAId;
use crate::features::program_data::{AdjacentSprites, ProjectType};
use crate::features::tileset::{Sprite, SpriteLayer};
use crate::util::UVec2JsonKey;
use glam::{IVec3, UVec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sprites {
    pub static_sprites: HashSet<StaticSprite>,
    pub animated_sprites: HashSet<AnimatedSprite>,
    pub fallback_sprites: HashSet<FallbackSprite>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct CreateMapData {
    name: String,
    size: UVec2JsonKey,
    ty: ProjectType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct StaticSprite {
    pub position: UVec2JsonKey,
    pub index: u32,
    pub layer: u32,
    pub z: i32,
    pub rotate_deg: i32,
}

impl Hash for StaticSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.layer.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq<Self> for StaticSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
            && self.layer.eq(&other.layer)
            && self.z.eq(&other.z)
    }
}

impl Eq for StaticSprite {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct AnimatedSprite {
    pub position: UVec2JsonKey,
    pub indices: Vec<u32>,
    pub layer: u32,
    pub z: i32,
    pub rotate_deg: i32,
}

impl Hash for AnimatedSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.layer.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq for AnimatedSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
            && self.layer.eq(&other.layer)
            && self.z.eq(&other.z)
    }
}

impl Eq for AnimatedSprite {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(super) struct FallbackSprite {
    pub position: UVec2JsonKey,
    pub index: u32,
    pub z: i32,
}

impl Hash for FallbackSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq for FallbackSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position) && self.z.eq(&other.z)
    }
}

impl Eq for FallbackSprite {}

#[derive(Debug)]
pub enum DisplaySprite {
    Static(StaticSprite),
    Animated(AnimatedSprite),
    Fallback(FallbackSprite),
}

impl DisplaySprite {
    pub fn get_display_sprite_from_sprite(
        sprite: &Sprite,
        tile_id: &MappedCDDAId,
        tile_position: IVec3,
        tile_layer: TileLayer,
        adjacent_sprites: &AdjacentSprites,
        json_data: &DeserializedCDDAJsonData,
    ) -> (Option<DisplaySprite>, Option<DisplaySprite>) {
        let position_uvec2 =
            UVec2::new(tile_position.x as u32, tile_position.y as u32);

        let fg = match sprite.get_fg_id(
            &tile_id,
            &tile_layer,
            adjacent_sprites,
            json_data,
        ) {
            None => None,
            Some(sprite_id) => match sprite.is_animated() {
                true => {
                    let display_sprite = AnimatedSprite {
                        position: UVec2JsonKey(position_uvec2),
                        layer: (tile_layer.clone() as u32) * 2
                            + SpriteLayer::Fg as u32,
                        indices: sprite_id.data.into_vec(),
                        rotate_deg: sprite_id.rotation.deg()
                            + tile_id.rotation.deg(),
                        z: tile_position.z,
                    };

                    Some(DisplaySprite::Animated(display_sprite))
                },
                false => {
                    let display_sprite = StaticSprite {
                        position: UVec2JsonKey(position_uvec2),
                        layer: (tile_layer.clone() as u32) * 2
                            + SpriteLayer::Fg as u32,
                        index: sprite_id.data.into_single().unwrap(),
                        rotate_deg: sprite_id.rotation.deg(),
                        z: tile_position.z,
                    };

                    Some(DisplaySprite::Static(display_sprite))
                },
            },
        };

        let bg = match sprite.get_bg_id(
            &tile_id,
            &tile_layer,
            adjacent_sprites,
            json_data,
        ) {
            None => None,
            Some(id) => match sprite.is_animated() {
                true => {
                    let display_sprite = AnimatedSprite {
                        position: UVec2JsonKey(position_uvec2),
                        layer: (tile_layer as u32) * 2 + SpriteLayer::Bg as u32,
                        indices: id.data.into_vec(),
                        rotate_deg: id.rotation.deg(),
                        z: tile_position.z,
                    };

                    Some(DisplaySprite::Animated(display_sprite))
                },
                false => {
                    let display_sprite = StaticSprite {
                        position: UVec2JsonKey(position_uvec2),
                        layer: (tile_layer as u32) * 2 + SpriteLayer::Bg as u32,
                        index: id.data.into_single().unwrap(),
                        rotate_deg: id.rotation.deg(),
                        z: tile_position.z,
                    };

                    Some(DisplaySprite::Static(display_sprite))
                },
            },
        };

        (fg, bg)
    }
}
