pub(crate) mod handlers;

use crate::map_data::Identifier;
use anyhow::Error;
use image::{GenericImageView, ImageResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Weighted<T> {
    weight: i32,
    sprite: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    Weighted(Weighted<T>),
    NotWeighted(T),
}

impl<T> MeabyWeighted<T> {
    pub fn value(&self) -> &T {
        match self {
            MeabyWeighted::Weighted(w) => &w.sprite,
            MeabyWeighted::NotWeighted(nw) => nw,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum MeabyVec<T> {
    Vec(Vec<T>),
    Single(T),
}

impl<T> MeabyVec<T> {
    pub fn map<F, V>(&self, mut f: F) -> Vec<V>
    where
        F: FnMut(&T) -> V,
    {
        match self {
            MeabyVec::Vec(v) => v.iter().map(f).collect(),
            MeabyVec::Single(s) => vec![f(s)],
        }
    }

    pub fn for_each<F>(&self, mut f: F)
    where
        F: FnMut(&T) -> (),
    {
        match self {
            MeabyVec::Vec(v) => v.iter().for_each(f),
            MeabyVec::Single(s) => f(s),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TilesetInfo {
    pixelscale: u32,
    width: u32,
    height: u32,
    zlevel_height: u32,
    iso: bool,
    retract_dist_min: f32,
    retract_dist_max: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectionType {
    #[serde(rename = "center")]
    Center,

    #[serde(rename = "corner")]
    Corner,

    #[serde(rename = "t_connection")]
    TConnection,

    #[serde(rename = "edge")]
    Edge,

    #[serde(rename = "end_piece")]
    EndPiece,

    #[serde(rename = "broken")]
    Broken,

    #[serde(rename = "unconnected")]
    Unconnected,

    #[serde(rename = "open")]
    Open,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdditionalTile {
    id: ConnectionType,
    fg: Option<MeabyVec<MeabyWeighted<MeabyVec<i32>>>>,
    bg: Option<MeabyVec<MeabyWeighted<MeabyVec<i32>>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpritesheetTile {
    id: MeabyVec<Identifier>,
    fg: Option<MeabyVec<MeabyWeighted<i32>>>,
    bg: Option<MeabyVec<MeabyWeighted<i32>>>,
    rotates: Option<bool>,
    animated: Option<bool>,
    multitile: Option<bool>,
    additional_tiles: Option<Vec<AdditionalTile>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TilesetTiles {
    file: PathBuf,

    spritesheet_dimensions: Option<(u32, u32)>,

    #[serde(rename = "//")]
    comment: Option<String>,

    sprite_width: Option<u32>,
    sprite_height: Option<u32>,
    sprite_offset_x: Option<i32>,
    sprite_offset_y: Option<i32>,

    tiles: Vec<SpritesheetTile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TilesetConfig {
    #[serde(rename = "tile_info")]
    info: Vec<TilesetInfo>,
    #[serde(rename = "tiles-new")]
    tiles: Vec<TilesetTiles>,
}

pub struct TilesetReader {
    path: PathBuf,
}

impl TilesetReader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn read(&self) -> Result<TilesetConfig, Error> {
        if !fs::exists(&self.path.join("tile_config.json"))? {
            return Err(Error::msg("tile_config.json does not exist"));
        }

        let tile_config = fs::read_to_string(&self.path.join("tile_config.json"))?;
        let mut config: TilesetConfig = serde_json::from_str(&tile_config)?;

        // TODO: Idk why there are multiple infos
        let info = config.info.get(0).expect("At least one info to exist");

        for tile in config.tiles.iter_mut() {
            let image_file = match image::open(self.path.join(&tile.file)) {
                Ok(i) => i,
                Err(e) => {
                    log::error!("Error while reading image {:?}, error: {}", tile.file, e);
                    continue;
                }
            };

            tile.spritesheet_dimensions = Some(image_file.dimensions());

            // If the width and height are not present, set them to the tileset default
            tile.sprite_width = match tile.sprite_width {
                None => Some(info.width),
                Some(w) => Some(w),
            };

            tile.sprite_height = match tile.sprite_height {
                None => Some(info.height),
                Some(h) => Some(h),
            };
        }

        Ok(config)
    }
}
