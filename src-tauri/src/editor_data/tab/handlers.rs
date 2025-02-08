use crate::editor_data::tab::{Tab, TabType};
use crate::editor_data::{EditorData, EditorDataSaver};
use crate::util::Save;
use log::info;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn create_tab(
    name: String,
    tab_type: TabType,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), ()> {
    let tab = Tab { name, tab_type };

    let mut lock = editor_data.lock().await;
    lock.tabs.push(tab.clone());
    info!("Opened tab {}", tab.name);

    let editor_data_saver = EditorDataSaver {
        path: lock.config.config_path.clone(),
    };

    editor_data_saver.save(&lock).expect("Saving to not fail");

    app.emit("tab_created", tab).expect("Emit to not fail");

    Ok(())
}

#[tauri::command]
pub async fn close_tab(
    index: usize,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), ()> {
    let mut lock = editor_data.lock().await;

    assert!(index < lock.tabs.len());

    lock.tabs.remove(index);
    info!("Closed tab {}", index);

    let editor_data_saver = EditorDataSaver {
        path: lock.config.config_path.clone(),
    };

    editor_data_saver.save(&lock).expect("Saving to not fail");

    app.emit("tab_closed", index).expect("Emit to not fail");

    Ok(())
}
