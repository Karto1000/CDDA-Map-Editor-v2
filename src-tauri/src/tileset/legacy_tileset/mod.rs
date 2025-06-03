use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::terrain::CDDATerrain;
use crate::cdda_data::vehicle_parts::CDDAVehiclePart;
use crate::tileset::io::{TilesheetConfigLoader, TilesheetLoader};
use crate::tileset::legacy_tileset::tile_config::{
    AdditionalTile, AdditionalTileType, LegacyTileConfig, Spritesheet, Tile,
};
use crate::tileset::MeabyWeightedSprite::Weighted;
use crate::tileset::{
    ForeBackIds, MeabyWeightedSprite, SingleSprite, Sprite, SpriteOrFallback,
    Tilesheet, WeightedSprite, FALLBACK_TILE_MAPPING, FALLBACK_TILE_ROW_SIZE,
};
use crate::util::Load;
use anyhow::{anyhow, Error};
use cdda_lib::types::{CDDAIdentifier, MeabyVec};
use derive_more::Display;
use log::{debug, info, warn};
use rand::distr::Distribution;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::ops::{Add, BitAndAssign};
use std::ptr::write;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub(crate) mod tile_config;

pub type SpriteIndex = u32;
pub type FinalIds = Option<Vec<WeightedSprite<Rotates>>>;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Serialize for Rotation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clone().deg().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deg = u32::deserialize(deserializer)? % 360;

        match deg {
            0 => Ok(Rotation::Deg0),
            90 => Ok(Rotation::Deg90),
            180 => Ok(Rotation::Deg180),
            270 => Ok(Rotation::Deg270),
            _ => Err(SerdeError::custom(format!(
                "Invalid rotation value {}",
                deg
            ))),
        }
    }
}

impl Add<Rotation> for Rotation {
    type Output = Rotation;

    fn add(self, rhs: Rotation) -> Self::Output {
        let value = self.deg() + rhs.deg();
        Self::from(value)
    }
}

impl From<i32> for Rotation {
    fn from(value: i32) -> Self {
        let value = value % 360;

        match value {
            0..90 => Self::Deg0,
            90..180 => Self::Deg90,
            180..270 => Self::Deg180,
            270..360 => Self::Deg270,
            _ => unreachable!(),
        }
    }
}

impl Rotation {
    pub fn deg(&self) -> i32 {
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

    pub fn new(data: T, rotation: Rotation) -> Self {
        Self { data, rotation }
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
            },
            (Some(first), Some(second), Some(third), Some(fourth)) => {
                Ok(Rotates::Pre4((
                    first.clone(),
                    second.clone(),
                    third.clone(),
                    fourth.clone(),
                )))
            },
            (_, _, _, _) => Err(anyhow!(
                "Invalid vec supplied for rotation for sprite indices {:?}",
                value
            )),
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

#[derive(Clone, Debug, Deserialize, Serialize, Default, Eq, PartialEq)]
pub struct TilesheetCDDAId {
    pub id: CDDAIdentifier,
    pub prefix: Option<String>,
    pub postfix: Option<String>,
}

impl Display for TilesheetCDDAId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.prefix {
            None => {},
            Some(prefix) => {
                write!(f, "{}_", prefix)?;
            },
        }

        let res = write!(f, "{}", self.id);

        match &self.postfix {
            None => res,
            Some(postfix) => {
                write!(f, "_{}", postfix)
            },
        }
    }
}

impl TilesheetCDDAId {
    pub fn simple(id: impl Into<CDDAIdentifier>) -> TilesheetCDDAId {
        TilesheetCDDAId {
            id: id.into(),
            prefix: None,
            postfix: None,
        }
    }

    pub fn full(&self) -> CDDAIdentifier {
        CDDAIdentifier(format!(
            "{}{}{}",
            self.prefix
                .clone()
                .map(|p| format!("{}_", p))
                .unwrap_or_default(),
            self.id,
            self.postfix
                .clone()
                .map(|p| format!("_{}", p))
                .unwrap_or_default(),
        ))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MappedCDDAId {
    pub tilesheet_id: TilesheetCDDAId,
    pub rotation: Rotation,
    pub is_broken: bool,
    pub is_open: bool,
}

impl MappedCDDAId {
    pub fn simple(id: impl Into<TilesheetCDDAId>) -> Self {
        Self {
            tilesheet_id: id.into(),
            rotation: Default::default(),
            is_broken: false,
            is_open: false,
        }
    }

    ///
    /// Some parts can have multiple variants; each variant can define the symbols and broken symbols,
    /// also each variant is a tileset sprite, if the tileset defines one for the variant.
    //
    // If a part has variants, the specific variant can be specified in the vehicle prototype by
    // appending the variant to the part id after a # symbol. Thus, "frame#cross" is the "cross" variant of the "frame" part.
    //
    // Variants perform a mini-lookup chain by slicing variant string until the next _ from the
    // right until a match is found. For example the tileset lookups for seat_leather#windshield_left are as follows:
    //
    //     vp_seat_leather_windshield_left
    //
    //     vp_seat_leather_windshield
    //
    // ( At this point variant is completely gone and default tile is looked for: )
    //
    //     vp_seat_leather
    //
    // ( If still no match is found then the looks_like field of vp_seat_leather is used and tileset looks for: )
    //
    //     vp_seat
    ///
    ///
    pub fn slice_right(&self) -> MappedCDDAId {
        let new_postfix = self
            .tilesheet_id
            .postfix
            .clone()
            .map(|p| p.rsplit_once('_').map(|(s, _)| s.to_string()));

        MappedCDDAId {
            tilesheet_id: TilesheetCDDAId {
                id: self.tilesheet_id.id.clone(),
                prefix: self.tilesheet_id.prefix.clone(),
                postfix: new_postfix.flatten(),
            },
            rotation: self.rotation.clone(),
            is_broken: self.is_broken.clone(),
            is_open: self.is_open.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct MappedCDDAIdsForTile {
    pub terrain: Option<MappedCDDAId>,
    pub furniture: Option<MappedCDDAId>,
    pub monster: Option<MappedCDDAId>,
    pub field: Option<MappedCDDAId>,
}

impl MappedCDDAIdsForTile {
    pub fn override_none(&mut self, other: MappedCDDAIdsForTile) {
        if other.terrain.is_some() {
            self.terrain = other.terrain;
        }

        if other.furniture.is_some() {
            self.furniture = other.furniture;
        }

        if other.monster.is_some() {
            self.monster = other.monster;
        }

        if other.field.is_some() {
            self.field = other.field;
        }
    }
}

fn to_weighted_vec(
    indices: Option<MeabyVec<MeabyWeightedSprite<MeabyVec<SpriteIndex>>>>,
) -> Option<Vec<WeightedSprite<Rotates>>> {
    let mut mapped_indices = Vec::new();

    for fg_indices_outer in indices?.into_vec() {
        let (indices_vec, weight) = match fg_indices_outer {
            MeabyWeightedSprite::NotWeighted(nw) => (nw.into_vec(), 1),
            MeabyWeightedSprite::Weighted(w) => (w.sprite.into_vec(), w.weight),
        };

        match Rotates::try_from(indices_vec) {
            Ok(v) => {
                mapped_indices.push(WeightedSprite::new(v, weight));
            },
            Err(e) => {
                // TODO: This happens when the supplied fg or bg is an empty array
                warn!("{}, this is probably due to an empty array. Ignoring this entry ", e);
                continue;
            },
        }
    }

    Some(mapped_indices)
}

fn get_multitile_sprite_from_additional_tiles(
    tile: &Tile,
    additional_tiles: &Vec<AdditionalTile>,
) -> Result<Sprite, Error> {
    let mut additional_tile_ids = HashMap::new();
    // Special cases for open and broken
    let mut broken: Option<SingleSprite> = None;
    let mut open: Option<SingleSprite> = None;

    for additional_tile in additional_tiles {
        match additional_tile.id {
            AdditionalTileType::Broken => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                broken = Some(SingleSprite {
                    ids: ForeBackIds::new(fg, bg),
                    animated: false,
                    rotates: false,
                });
            },
            AdditionalTileType::Open => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                open = Some(SingleSprite {
                    ids: ForeBackIds::new(fg, bg),
                    animated: false,
                    rotates: false,
                });
            },
            _ => {
                let fg = to_weighted_vec(additional_tile.fg.clone());
                let bg = to_weighted_vec(additional_tile.bg.clone());

                additional_tile_ids.insert(
                    additional_tile.id.clone(),
                    SingleSprite {
                        ids: ForeBackIds::new(fg, bg),
                        animated: additional_tile.animated.unwrap_or(false),
                        rotates: additional_tile.rotates.unwrap_or(true),
                    },
                );
            },
        }
    }

    let fg = to_weighted_vec(tile.fg.clone());
    let bg = to_weighted_vec(tile.bg.clone());

    Ok(Sprite::Multitile {
        fallback: SingleSprite {
            ids: ForeBackIds::new(fg, bg),
            rotates: tile.rotates.unwrap_or(false),
            animated: tile.animated.unwrap_or(false),
        },
        center: additional_tile_ids.remove(&AdditionalTileType::Center),
        corner: additional_tile_ids.remove(&AdditionalTileType::Corner),
        edge: additional_tile_ids.remove(&AdditionalTileType::Edge),
        t_connection: additional_tile_ids
            .remove(&AdditionalTileType::TConnection),
        unconnected: additional_tile_ids
            .remove(&AdditionalTileType::Unconnected),
        end_piece: additional_tile_ids.remove(&AdditionalTileType::EndPiece),
        broken,
        open,
    })
}

pub struct LegacyTilesheet {
    pub id_map: HashMap<CDDAIdentifier, Sprite>,
    pub fallback_map: HashMap<String, SpriteIndex>,
}

impl Tilesheet for LegacyTilesheet {
    fn get_fallback(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> SpriteIndex {
        match json_data.terrain.get(&id.tilesheet_id.id) {
            None => {},
            Some(t) => {
                return self
                    .fallback_map
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
                    .clone()
            },
        }

        match json_data.furniture.get(&id.tilesheet_id.id) {
            None => {},
            Some(f) => {
                return self
                    .fallback_map
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
                    .clone()
            },
        };

        FALLBACK_TILE_MAPPING.first().unwrap().1
    }
    fn get_sprite(
        &self,
        id: &MappedCDDAId,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<&Sprite> {
        match self.id_map.get(&id.tilesheet_id.full()) {
            None => {
                debug!(
                    "Could not find {} in tilesheet ids, trying to use looks_like property",
                    id.tilesheet_id.full(),
                );

                let sliced_postfix = id.slice_right();
                debug!(
                    "Slicing postfix and trying to get sprite again, new id {}",
                    &sliced_postfix.tilesheet_id
                );

                match sliced_postfix.tilesheet_id.postfix {
                    None => {
                        // We want to get the sprites one more time after the entire postfix has been sliced
                        if id.tilesheet_id.postfix.is_some() {
                            return self.get_sprite(&sliced_postfix, json_data);
                        }
                    },
                    Some(_) => {
                        return self.get_sprite(&sliced_postfix, json_data)
                    },
                }

                self.get_looks_like_sprite(
                    &sliced_postfix.tilesheet_id.id,
                    &json_data,
                )
            },
            Some(s) => {
                debug!("Found sprite with id {}", id.tilesheet_id.full());
                Some(s)
            },
        }
    }
}

impl LegacyTilesheet {
    fn get_looks_like_sprite(
        &self,
        id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
    ) -> Option<&Sprite> {
        // Id of a similar item that this item looks like. The tileset loader will try to load the
        // tile for that item if this item doesn't have a tile. Looks_like entries are implicitly
        // chained, so if 'throne' has looks_like 'big_chair' and 'big_chair' has looks_like 'chair',
        // a throne will be displayed using the chair tile if tiles for throne and big_chair do not exist.
        // If a tileset can't find a tile for any item in the looks_like chain, it will default to the ascii symbol.

        // The tiles with this property do not have a corresponding entry in the tilesheet which
        // means that we have to check this here dynamically
        match json_data.terrain.get(&id) {
            None => {},
            Some(s) => {
                return match &s.looks_like {
                    None => None,
                    Some(ident) => {
                        // "looks_like entries are implicitly chained"
                        if ident == id {
                            return self.id_map.get(ident);
                        }

                        match self.id_map.get(ident) {
                            None => {
                                self.get_looks_like_sprite(ident, json_data)
                            },
                            Some(s) => Some(s),
                        }
                    },
                };
            },
        };

        // Do again with furniture
        match json_data.furniture.get(&id) {
            None => {},
            Some(s) => {
                return match &s.looks_like {
                    None => None,
                    Some(ident) => {
                        if ident == id {
                            return self.id_map.get(ident);
                        }

                        match self.id_map.get(ident) {
                            None => {
                                self.get_looks_like_sprite(ident, json_data)
                            },
                            Some(s) => Some(s),
                        }
                    },
                }
            },
        }

        match json_data.vehicle_parts.get(&id) {
            None => None,
            Some(s) => match &s.looks_like {
                None => None,
                Some(ident) => {
                    debug!("Looking for looks like {} for {}", ident, id);

                    // Stop stackoverflow when object "looks_like" itself
                    if ident == id {
                        return self.id_map.get(ident);
                    }

                    match self
                        .id_map
                        .get(&CDDAIdentifier(format!("vp_{}", ident)))
                    {
                        None => self.get_looks_like_sprite(ident, json_data),
                        Some(s) => Some(s),
                    }
                },
            },
        }
    }
}

impl Load<LegacyTilesheet> for TilesheetLoader<LegacyTileConfig> {
    async fn load(&mut self) -> Result<LegacyTilesheet, Error> {
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
                let is_multitile = tile.multitile.unwrap_or_else(|| false)
                    && tile.additional_tiles.is_some();

                if !is_multitile {
                    let fg = to_weighted_vec(tile.fg.clone());
                    let bg = to_weighted_vec(tile.bg.clone());

                    tile.id.for_each(|id| {
                        id_map.insert(
                            id.clone(),
                            Sprite::Single(SingleSprite {
                                ids: ForeBackIds::new(fg.clone(), bg.clone()),
                                animated: tile.animated.unwrap_or(false),
                                rotates: tile.rotates.unwrap_or(false),
                            }),
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
                            get_multitile_sprite_from_additional_tiles(
                                tile,
                                additional_tiles,
                            )
                            .unwrap(),
                        );
                    });
                }
            }
        }

        let fallback_spritesheet =
            fallback_spritesheet.expect("Fallback spritesheet to exist");

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
    async fn load(&mut self) -> Result<LegacyTileConfig, Error> {
        let config_path = self.tileset_path.join("tile_config.json");

        let mut buffer = vec![];
        File::open(config_path)
            .await?
            .read_to_end(&mut buffer)
            .await?;

        Ok(serde_json::from_slice::<LegacyTileConfig>(&buffer)
            .map_err(|e| anyhow!("{:?}", e))?)
    }
}
