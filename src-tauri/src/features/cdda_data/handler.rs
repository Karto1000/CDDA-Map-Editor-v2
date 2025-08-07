use crate::data::io::{
    parse_json_entries, replace_data_in_cdda_data, set_intermediate_data_from_json_entries,
    DeserializedCDDAJsonData, IntermediateCDDAJsonData,
};
use crate::data::palettes::CDDAPalette;
use crate::data::terrain::CDDATerrain;
use crate::impl_serialize_for_error;
use crate::util::{get_json_data, get_json_data_mut, CDDADataError};
use cdda_lib::types::CDDAIdentifier;
use log::info;
use serde_json::{Error, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum GetFieldError {
    #[error("Field `{0}` not found in cdda data")]
    FieldNotFound(String),

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),
}

impl_serialize_for_error!(GetFieldError);

#[tauri::command(rename_all = "camelCase")]
pub async fn get_cdda_data_field(
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    field_name: String,
) -> Result<Value, GetFieldError> {
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;

    let value_result = match field_name.as_str() {
        "palettes" => serde_json::to_value(&json_data.palettes),
        "map_data" => serde_json::to_value(&json_data.map_data),
        "region_settings" => serde_json::to_value(&json_data.region_settings),
        "terrain" => serde_json::to_value(&json_data.terrain),
        "furniture" => serde_json::to_value(&json_data.furniture),
        "item_groups" => serde_json::to_value(&json_data.item_groups),
        "overmap_locations" => {
            serde_json::to_value(&json_data.overmap_locations)
        },
        "overmap_terrains" => serde_json::to_value(&json_data.overmap_terrains),
        "overmap_specials" => serde_json::to_value(&json_data.overmap_specials),
        "vehicles" => serde_json::to_value(&json_data.vehicles),
        "vehicle_parts" => serde_json::to_value(&json_data.vehicle_parts),
        "monster_groups" => serde_json::to_value(&json_data.monster_groups),
        "monsters" => serde_json::to_value(&json_data.monsters),
        _ => Err(GetFieldError::FieldNotFound(field_name))?,
    };

    Ok(value_result.expect("Serialization to not fail"))
}

#[tauri::command]
pub async fn update_cdda_data_at(
    paths: Vec<PathBuf>,
    cdda_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), ()> {
    info!("Updating {:?}", paths);

    let mut intermediate_data = IntermediateCDDAJsonData::default();

    for path in paths {
        let extension = match path.extension() {
            None => {
                info!(
                    "Skipping entry {} because it does not have an extension",
                    path.display()
                );
                continue;
            },
            Some(e) => e,
        };

        if extension != "json" {
            info!("Skipping {} because it is not a json file", path.display());
            continue;
        }

        let deserialized_entries = match parse_json_entries(&path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        set_intermediate_data_from_json_entries(
            &mut intermediate_data,
            &path,
            deserialized_entries,
        )
        .unwrap();
    }

    let mut cdda_data_lock = cdda_data.lock().await;
    let cdda_data = get_json_data_mut(&mut cdda_data_lock).unwrap();

    // TODO: The copy-from property will not work here since the intermediate data is only a
    // collection of the data in this one file. If this copies data from another file, it will not work
    replace_data_in_cdda_data(cdda_data, intermediate_data).unwrap();

    Ok(())
}
