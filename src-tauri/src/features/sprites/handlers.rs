use crate::data::io::DeserializedCDDAJsonData;
use crate::features::map::MappedCDDAId;
use crate::features::tileset::legacy_tileset::{
    FallbackSpriteIndex, SpriteIndex, Tilesheet, TilesheetCDDAId,
};
use crate::features::tileset::FgBgIds;
use crate::impl_serialize_for_error;
use crate::util::{get_json_data, CDDADataError};
use cdda_lib::types::CDDAIdentifier;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum GetRepresentationsForTerrainIdsError {
    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),
}

impl_serialize_for_error!(GetRepresentationsForTerrainIdsError);

#[derive(Debug, Serialize)]
pub enum DefinedOrFallbackIndices {
    Defined(FgBgIds<Option<SpriteIndex>, Option<SpriteIndex>>),
    Fallback(FallbackSpriteIndex),
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_representations_for_terrain_ids(
    ids: Vec<CDDAIdentifier>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    tilesheet: State<'_, Mutex<Option<Tilesheet>>>,
    fallback_tilesheet: State<'_, Arc<Tilesheet>>,
) -> Result<
    HashMap<CDDAIdentifier, DefinedOrFallbackIndices>,
    GetRepresentationsForTerrainIdsError,
> {
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;
    let tilesheet_lock = tilesheet.lock().await;

    let mut representations = HashMap::new();

    for id in ids {
        let mapped_cdda_id =
            MappedCDDAId::simple(TilesheetCDDAId::simple(id.clone()));
    }

    Ok(representations)
}
