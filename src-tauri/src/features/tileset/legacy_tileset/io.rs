use crate::features::tileset::data::FALLBACK_TILE_MAPPING;
use crate::features::tileset::legacy_tileset::data::{
    LegacyTileConfig, Spritesheet,
};
use crate::features::tileset::legacy_tileset::LegacyTilesheet;
use crate::features::tileset::{
    legacy_tileset, ForeBackIds, SingleSprite, Sprite,
};
use crate::util::Load;
use anyhow::{anyhow, Error};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncReadExt;

pub struct LegacyTilesheetLoader {
    config: LegacyTileConfig,
}

impl LegacyTilesheetLoader {
    pub fn new(config: LegacyTileConfig) -> Self {
        Self { config }
    }
}

pub struct TileConfigLoader {
    pub path: PathBuf,
}

impl TileConfigLoader {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Load<LegacyTileConfig> for TileConfigLoader {
    async fn load(&mut self) -> Result<LegacyTileConfig, anyhow::Error> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(|e| anyhow::anyhow!(e))
    }
}

impl Load<LegacyTilesheet> for LegacyTilesheetLoader {
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
                    let fg = legacy_tileset::to_weighted_vec(tile.fg.clone());
                    let bg = legacy_tileset::to_weighted_vec(tile.bg.clone());

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
                            legacy_tileset::get_multitile_sprite_from_additional_tiles(
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
                    ascii_group.offset as u32 + offset,
                );
            }
        }

        Ok(LegacyTilesheet {
            id_map,
            fallback_map,
        })
    }
}

impl Load<LegacyTileConfig> for LegacyTilesheetConfigLoader {
    async fn load(&mut self) -> Result<LegacyTileConfig, Error> {
        let config_path = self.tileset_path.join("tile_config.json");

        let mut buffer = vec![];
        fs::File::open(config_path)
            .await?
            .read_to_end(&mut buffer)
            .await?;

        Ok(serde_json::from_slice::<LegacyTileConfig>(&buffer)
            .map_err(|e| anyhow!("{:?}", e))?)
    }
}

pub struct LegacyTilesheetConfigLoader {
    pub(crate) tileset_path: PathBuf,
}

impl LegacyTilesheetConfigLoader {
    pub fn new(tileset_path: PathBuf) -> Self {
        Self { tileset_path }
    }

    pub async fn load_value(&mut self) -> Result<Value, Error> {
        let legacy_tilesheet =
            <LegacyTilesheetConfigLoader as Load<LegacyTileConfig>>::load(self)
                .await;

        match legacy_tilesheet {
            Ok(v) => Ok(serde_json::to_value(v)?),
            Err(e) => {
                anyhow::bail!(e);
            },
        }
    }
}
