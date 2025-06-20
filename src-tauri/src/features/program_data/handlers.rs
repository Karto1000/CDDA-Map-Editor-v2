use crate::data::io::{load_cdda_json_data, DeserializedCDDAJsonData};
use crate::events;
use crate::events::UPDATE_LIVE_VIEWER;
use crate::features::program_data::io::ProgramDataSaver;
use crate::features::program_data::{
    get_map_data_collection_from_live_viewer_data, EditorData, LiveViewerData,
    Project, ProjectName, ProjectType, Tab, TabType,
};
use crate::features::tileset::legacy_tileset::{
    load_tilesheet, LegacyTilesheet,
};
use crate::features::toast::ToastMessage;
use crate::util::{get_json_data, CDDADataError, Save};
use log::{error, info, warn};
use notify_debouncer_full::new_debouncer;
use serde::Serialize;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio_test::block_on;

#[tauri::command]
pub async fn get_editor_data(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<EditorData, ()> {
    Ok(editor_data.lock().await.clone())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum InstallationPickedError {
    #[error("Picked directory is not a valid CDDA Directory, reason: `{0}`")]
    InvalidCDDADirectory(String),

    #[error("IO Error, `{0}`")]
    Io(String),
}

#[tauri::command]
pub async fn cdda_installation_directory_picked(
    path: PathBuf,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), InstallationPickedError> {
    let gfx_dir = fs::read_dir(&path.join("gfx")).map_err(|_| {
        InstallationPickedError::InvalidCDDADirectory(
            "Missing 'gfx' directory".into(),
        )
    })?;

    let mut available_tilesets = vec![];

    for entry in gfx_dir {
        let entry =
            entry.map_err(|e| InstallationPickedError::Io(e.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| InstallationPickedError::Io(e.to_string()))?;

        if file_type.is_file() {
            continue;
        }

        available_tilesets
            .push(entry.file_name().to_string_lossy().into_owned());
    }

    let mut editor_data_lock = editor_data.lock().await;
    editor_data_lock.available_tilesets = Some(available_tilesets);
    editor_data_lock.config.cdda_path = Some(path);

    match load_cdda_json_data(
        &editor_data_lock.config.cdda_path.clone().unwrap(),
        &editor_data_lock.config.json_data_path,
    )
    .await
    {
        Ok(data) => {
            let mut json_data_lock = json_data.lock().await;
            json_data_lock.replace(data);
        },
        Err(e) => {
            warn!("{}", e);
            return Err(InstallationPickedError::InvalidCDDADirectory(
                "Failed to load json data".into(),
            ));
        },
    }

    app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .unwrap();

    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum TilesetPickedError {
    #[error("The selected tileset does not exist")]
    NotATileset,

    #[error("No CDDA game directory has been picked")]
    NoCDDADirPicked,
}

#[tauri::command]
pub async fn tileset_picked(
    tileset: String,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    tilesheet: State<'_, Mutex<Option<LegacyTilesheet>>>,
) -> Result<(), TilesetPickedError> {
    let mut editor_data_lock = editor_data.lock().await;
    let mut tilesheet_lock = tilesheet.lock().await;

    let tilesets = match &editor_data_lock.available_tilesets {
        None => return Err(TilesetPickedError::NoCDDADirPicked),
        Some(t) => t,
    };

    // This is the default tileset
    if tileset == "None" {
        editor_data_lock.config.selected_tileset = None;
        tilesheet_lock.take();
    } else {
        match tilesets.iter().find(|t| **t == tileset) {
            None => return Err(TilesetPickedError::NotATileset),
            Some(_) => {},
        }

        editor_data_lock.config.selected_tileset = Some(tileset.clone());
        *tilesheet_lock =
            load_tilesheet(&editor_data_lock).await.map_err(|e| {
                error!("Failed to load tilesheet, `{0}`", e);
                TilesetPickedError::NotATileset
            })?;
    }

    let saver = ProgramDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(&editor_data_lock).await.unwrap();
    app.emit(events::TILESET_CHANGED, ()).unwrap();

    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum SaveEditorDataError {
    #[error("Failed to save editor data, `{0}`")]
    SaveFailed(String),
}

#[tauri::command]
pub async fn save_editor_data(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), SaveEditorDataError> {
    let lock = editor_data.lock().await;

    let saver = ProgramDataSaver {
        path: lock.config.config_path.clone(),
    };

    saver.save(&lock).await.map_err(|e| {
        error!("Failed to save editor data, `{0}`", e);
        SaveEditorDataError::SaveFailed(e.to_string())
    })?;

    Ok(())
}

#[tauri::command]
pub async fn close_project(
    app: AppHandle,
    name: ProjectName,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), ()> {
    let mut editor_data_lock = editor_data.lock().await;

    match editor_data_lock.opened_project.clone() {
        None => {},
        Some(name) => {
            app.emit(events::TAB_REMOVED, name).unwrap();
        },
    }

    editor_data_lock.opened_project = None;
    editor_data_lock.loaded_projects.remove(&name);
    editor_data_lock.openable_projects.remove(&name);

    let saver = ProgramDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(&editor_data_lock).await.unwrap();

    app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .unwrap();

    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum OpenProjectError {
    #[error("No project with name `{0}` was found in recent projects")]
    NoRecentProject(String),

    #[error("The file is not a valid json project file")]
    InvalidContent,

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),
}

#[tauri::command]
pub async fn open_recent_project(
    name: ProjectName,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), OpenProjectError> {
    let mut editor_data_lock = editor_data.lock().await;
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;

    let recent_project = editor_data_lock
        .recent_projects
        .iter()
        .find(|p| p.name == name)
        .ok_or(OpenProjectError::NoRecentProject(name.clone()))?;

    let mut project: Project = serde_json::from_str(
        fs::read_to_string(recent_project.path.join(format!("{}.json", name)))
            .map_err(|_| OpenProjectError::NoRecentProject(name.clone()))?
            .as_str(),
    )
    .map_err(|_| OpenProjectError::InvalidContent)?;

    match &project.ty {
        ProjectType::MapEditor(_) => unimplemented!(),
        ProjectType::LiveViewer(lvd) => {
            let mut map_data_collection =
                match get_map_data_collection_from_live_viewer_data(&lvd).await
                {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("Failed to load map data for project; {}", e);
                        return Err(OpenProjectError::InvalidContent);
                    },
                };

            map_data_collection.iter_mut().for_each(|(_, m)| {
                match m.calculate_parameters(&json_data.palettes) {
                    Ok(_) => {},
                    Err(e) => {
                        warn!("{}", e);
                    },
                }
            });

            project.maps = map_data_collection;

            app.emit(
                events::TAB_CREATED,
                Tab {
                    name: project.name.clone(),
                    tab_type: TabType::LiveViewer,
                },
            )
            .unwrap();

            editor_data_lock
                .openable_projects
                .insert(project.name.clone());

            editor_data_lock
                .loaded_projects
                .insert(project.name.clone(), project);

            let saver = ProgramDataSaver {
                path: editor_data_lock.config.config_path.clone(),
            };

            saver.save(&editor_data_lock).await.unwrap();

            app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
                .unwrap();
        },
    }

    Ok(())
}

#[tauri::command]
pub async fn open_project(
    name: String,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    file_watcher: State<'_, Mutex<Option<tokio::task::JoinHandle<()>>>>,
) -> Result<(), ()> {
    let mut file_watcher_lock = file_watcher.lock().await;
    match file_watcher_lock.deref() {
        None => {},
        Some(s) => s.abort(),
    }

    let mut editor_data_lock = editor_data.lock().await;
    editor_data_lock.opened_project = Some(name.clone());

    app.emit(events::EDITOR_DATA_CHANGED, editor_data_lock.clone())
        .unwrap();

    let project = match editor_data_lock.loaded_projects.get(&name) {
        None => {
            warn!("Could not find project with name {}", name);
            return Err(());
        },
        Some(d) => d,
    };

    match &project.ty {
        ProjectType::MapEditor(_) => {},
        ProjectType::LiveViewer(lvd) => {
            app.emit(UPDATE_LIVE_VIEWER, {}).unwrap();

            let lvd_clone = lvd.clone();

            let join_handle = tokio::spawn(async move {
                info!("Spawning File Watcher for Live Viewer");

                let (tx, mut rx) = tokio::sync::mpsc::channel(1);

                // Thx -> https://github.com/notify-rs/notify/blob/d7e22791faffb7bd9bd10f031c260ae019d7f474/examples/async_monitor.rs
                // And -> https://docs.rs/notify-debouncer-full/latest/notify_debouncer_full/
                let mut debouncer = new_debouncer(
                    Duration::from_millis(100),
                    None,
                    move |res| {
                        block_on(async { tx.send(res).await.unwrap() });
                    },
                )
                .unwrap();

                let mapgen_paths = match lvd_clone {
                    LiveViewerData::Terrain {
                        mapgen_file_paths, ..
                    } => mapgen_file_paths,
                    LiveViewerData::Special {
                        mapgen_file_paths, ..
                    } => mapgen_file_paths,
                };

                for path in mapgen_paths.iter() {
                    debouncer
                        .watch(path, notify::RecursiveMode::NonRecursive)
                        .unwrap();
                }

                while let Some(Ok(_)) = rx.recv().await {
                    info!("Reloading Project");
                    app.emit(UPDATE_LIVE_VIEWER, {}).unwrap()
                }
            });
            file_watcher_lock.replace(join_handle);
        },
    }

    Ok(())
}
