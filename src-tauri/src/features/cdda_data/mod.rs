use crate::data::io::DeserializedCDDAJsonData;
use crate::data::palettes::CDDAPalette;
use crate::impl_serialize_for_error;
use crate::util::{get_json_data, CDDADataError};
use cdda_lib::types::CDDAIdentifier;
use std::collections::HashMap;
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
