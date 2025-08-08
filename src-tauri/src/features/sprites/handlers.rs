use crate::data::io::DeserializedCDDAJsonData;
use crate::data::{ConstantRng, TileLayer};
use crate::features::map::MappedCDDAId;
use crate::features::tileset::legacy_tileset::{
    FallbackSpriteIndex, SpriteIndex, Tilesheet, TilesheetCDDAId,
};
use crate::features::tileset::{
    FgBgIds, GetSprite, PickSpriteIndex, RepresentativeSpritePicker,
    SpriteLayer,
};
use crate::impl_serialize_for_error;
use crate::util::{get_json_data, CDDADataError};
use cdda_lib::types::{CDDAIdentifier, MeabyVec};
use serde::Serialize;
use std::collections::HashMap;
use std::ops::Deref;
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
#[serde(tag = "type", content = "ids")]
pub enum DefinedOrFallbackIndices {
    Defined(
        FgBgIds<Option<MeabyVec<SpriteIndex>>, Option<MeabyVec<SpriteIndex>>>,
    ),
    Fallback(FallbackSpriteIndex),
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_representations_for_ids(
    ids: Vec<CDDAIdentifier>,
    tile_layer: TileLayer,
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
    let mut representation_picker = RepresentativeSpritePicker;

    for id in ids {
        let mapped_cdda_id = MappedCDDAId::simple(TilesheetCDDAId::simple(
            json_data
                .replace_possible_region_settings(&mut ConstantRng, id.clone()),
        ));

        match tilesheet_lock.deref() {
            None => {
                let fallback_index =
                    fallback_tilesheet.get_fallback(&mapped_cdda_id, json_data);

                representations.insert(
                    id,
                    DefinedOrFallbackIndices::Fallback(
                        (*fallback_index).into(),
                    ),
                );
            },
            Some(t) => {
                match t.get_sprite(&mapped_cdda_id, json_data) {
                    None => {
                        let fallback_index =
                            t.get_fallback(&mapped_cdda_id, json_data);

                        representations.insert(
                            id,
                            DefinedOrFallbackIndices::Fallback(
                                (*fallback_index).into(),
                            ),
                        );
                    },
                    Some(s) => {
                        let fg = representation_picker
                            .pick((), s, SpriteLayer::Fg, tile_layer, json_data)
                            .map(|fg| fg.data);

                        let bg = representation_picker
                            .pick((), s, SpriteLayer::Bg, tile_layer, json_data)
                            .map(|bg| bg.data);

                        representations.insert(
                            id,
                            DefinedOrFallbackIndices::Defined(FgBgIds::new(
                                fg, bg,
                            )),
                        );
                    },
                };
            },
        }
    }

    Ok(representations)
}
