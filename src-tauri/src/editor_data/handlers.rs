use crate::editor_data::EditorData;
use tauri::async_runtime::Mutex;
use tauri::State;

#[tauri::command]
pub async fn get_editor_data(editor_data: State<'_, Mutex<EditorData>>) -> Result<EditorData, ()> {
    Ok(editor_data.lock().await.clone())
}
