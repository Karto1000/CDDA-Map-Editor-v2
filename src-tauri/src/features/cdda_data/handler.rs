use crate::data::io::{
    parse_json_entries, replace_data_in_cdda_data, set_intermediate_data_from_json_entries,
    DeserializedCDDAJsonData, IntermediateCDDAJsonData,
};
use crate::data::palettes::CDDAPalette;
use crate::impl_serialize_for_error;
use crate::util::{get_json_data, get_json_data_mut, CDDADataError};
use cdda_lib::types::CDDAIdentifier;
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::State;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Debug, Error)]
pub enum GetPalettesError {
    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),
}

impl_serialize_for_error!(GetPalettesError);

#[tauri::command]
pub async fn get_palettes(
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<HashMap<CDDAIdentifier, CDDAPalette>, GetPalettesError> {
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;
    Ok(json_data.palettes.clone())
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
