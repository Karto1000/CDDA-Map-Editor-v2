use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::tileset::io::{TilesheetConfigLoader, TilesheetLoader};
use crate::tileset::{SpriteKind, Tilesheet};
use crate::util::{CDDAIdentifier, Load};
use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTileConfig;

pub struct CurrentTilesheet {}

impl Tilesheet for CurrentTilesheet {
    fn get_sprite(
        &self,
        id: &CDDAIdentifier,
        json_data: &DeserializedCDDAJsonData,
    ) -> SpriteKind {
        todo!()
    }
}

impl Load<CurrentTilesheet> for TilesheetLoader<CurrentTileConfig> {
    async fn load(&mut self) -> Result<CurrentTilesheet, Error> {
        Err(anyhow!("Not Implemented"))
    }
}

impl Load<CurrentTileConfig> for TilesheetConfigLoader {
    async fn load(&mut self) -> Result<CurrentTileConfig, Error> {
        Err(anyhow!("Not Implemented"))
    }
}
