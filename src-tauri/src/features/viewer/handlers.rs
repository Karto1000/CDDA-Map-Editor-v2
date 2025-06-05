use super::data::PlaceSpritesEvent;
use crate::data::io::DeserializedCDDAJsonData;
use crate::data::replace_region_setting;
use crate::data::TileLayer;
use crate::events;
use crate::events::UPDATE_LIVE_VIEWER;
use crate::features::map::importing::{
    OvermapSpecialImporter, SingleMapDataImporter,
};
use crate::features::map::MappedCDDAId;
use crate::features::map::SPECIAL_EMPTY_CHAR;
use crate::features::map::{CalculateParametersError, DEFAULT_MAP_DATA_SIZE};
use crate::features::program_data::EditorData;
use crate::features::program_data::EditorDataSaver;
use crate::features::program_data::GetLiveViewerDataError;
use crate::features::program_data::LiveViewerData;
use crate::features::program_data::MappedCDDAIdContainer;
use crate::features::program_data::Project;
use crate::features::program_data::ProjectType;
use crate::features::program_data::ZLevel;
use crate::features::program_data::{
    get_map_data_collection_live_viewer_data, Tab, TabType,
};
use crate::features::tileset::legacy_tileset::LegacyTilesheet;
use crate::features::tileset::legacy_tileset::TilesheetCDDAId;
use crate::features::tileset::Tilesheet;
use crate::features::viewer::data::{DisplaySprite, FallbackSprite};
use crate::impl_serialize_for_error;
use crate::util;
use crate::util::get_json_data;
use crate::util::CDDADataError;
use crate::util::GetCurrentProjectError;
use crate::util::IVec3JsonKey;
use crate::util::Save;
use crate::util::UVec2JsonKey;
use crate::util::{get_current_project_mut, get_size, Load};
use cdda_lib::types::{CDDAIdentifier, ParameterIdentifier};
use cdda_lib::DEFAULT_EMPTY_CHAR_ROW;
use cdda_lib::DEFAULT_MAP_HEIGHT;
use cdda_lib::DEFAULT_MAP_ROWS;
use comfy_bounded_ints::types::Bound_usize;
use glam::IVec3;
use glam::UVec2;
use indexmap::IndexMap;
use log::error;
use log::info;
use log::warn;
use notify::Watcher;
use notify_debouncer_full::new_debouncer;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde::Serialize;
use serde::Serializer;
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::time::Duration;
use strum::IntoEnumIterator;
use tauri::async_runtime::Mutex;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::State;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio_test::block_on;

#[tauri::command]
pub async fn get_current_project_data(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<Project, GetCurrentProjectError> {
    let editor_data_lock = editor_data.lock().await;
    let data = util::get_current_project(&editor_data_lock)?;
    Ok(data.clone())
}

#[derive(Debug, Error)]
pub enum GetCalculatedParametersError {
    #[error(transparent)]
    ProjectError(#[from] GetCurrentProjectError),
}

impl_serialize_for_error!(GetCalculatedParametersError);

#[tauri::command]
pub async fn get_calculated_parameters(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<
    HashMap<IVec3JsonKey, IndexMap<ParameterIdentifier, CDDAIdentifier>>,
    GetCalculatedParametersError,
> {
    let editor_data_lock = editor_data.lock().await;
    let data = util::get_current_project(&editor_data_lock)?;

    let mut calculated_parameters = HashMap::new();

    for (z, z_maps) in data.maps.iter() {
        for (map_coords, map) in z_maps.maps.iter() {
            calculated_parameters.insert(
                IVec3JsonKey(IVec3::new(
                    map_coords.x as i32,
                    map_coords.y as i32,
                    *z,
                )),
                map.calculated_parameters.clone(),
            );
        }
    }

    Ok(calculated_parameters)
}

#[tauri::command]
pub async fn get_sprites(
    name: String,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<LegacyTilesheet>>>,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    mapped_cdda_ids: State<
        '_,
        Mutex<Option<HashMap<ZLevel, MappedCDDAIdContainer>>>,
    >,
) -> Result<(), ()> {
    let mut json_data_lock = json_data.lock().await;

    let mut json_data = match json_data_lock.deref_mut() {
        None => return Err(()),
        Some(d) => d,
    };

    let mut editor_data_lock = editor_data.lock().await;

    let project = match editor_data_lock
        .projects
        .iter_mut()
        .find(|p| p.name == name)
    {
        None => {
            warn!("Could not find project with name {}", name);
            return Err(());
        },
        Some(d) => d,
    };

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(()),
        Some(t) => t,
    };

    let mut static_sprites = HashSet::new();
    let mut animated_sprites = HashSet::new();
    let fallback_sprites = HashSet::new();

    macro_rules! insert_sprite_type {
        ($val:expr) => {
            match $val {
                DisplaySprite::Static(s) => {
                    static_sprites.insert(s);
                },
                DisplaySprite::Animated(a) => {
                    animated_sprites.insert(a);
                },
                DisplaySprite::Fallback(f) => {
                    // fallback_sprites.
                    // insert(f);
                },
            }
        };
    }

    for (_, map_collection) in project.maps.iter_mut() {
        // we need to calculate the parameters for the predecessor here because we
        // cannot borrow json data as mutable inside the get_mapped_cdda_ids function
        map_collection.calculate_predecessor_parameters(&mut json_data);
    }

    let region_settings = json_data
        .region_settings
        .get(&CDDAIdentifier("default".into()))
        .expect("Region settings to exist");

    let mut saved_cdda_ids = HashMap::new();

    for (z, map_collection) in project.maps.iter() {
        let local_mapped_cdda_ids =
            map_collection.get_mapped_cdda_ids(json_data, *z).unwrap();

        let tile_map: Vec<
            HashMap<TileLayer, (Option<DisplaySprite>, Option<DisplaySprite>)>,
        > = local_mapped_cdda_ids
            .ids
            .par_iter()
            .map(|(p, identifier_group)| {
                let tile_3d_coords = IVec3::new(p.x, p.y, *z);

                if identifier_group.terrain.is_none()
                    && identifier_group.furniture.is_none()
                {
                    warn!(
                        "No sprites found for identifier_group {:?} at \
                         coordinates {}",
                        identifier_group, tile_3d_coords
                    );

                    return HashMap::new();
                }

                let mut layer_map = HashMap::new();

                // Layer is used here so furniture is
                // above terrain
                for (layer, o_id) in [
                    (TileLayer::Terrain, &identifier_group.terrain),
                    (TileLayer::Furniture, &identifier_group.furniture),
                    (TileLayer::Monster, &identifier_group.monster),
                    (TileLayer::Field, &identifier_group.field),
                ] {
                    let id = match o_id {
                        None => continue,
                        Some(mapped_id) => MappedCDDAId {
                            tilesheet_id: TilesheetCDDAId {
                                id: replace_region_setting(
                                    &mapped_id.tilesheet_id.id,
                                    region_settings,
                                    &json_data.terrain,
                                    &json_data.furniture,
                                ),
                                prefix: mapped_id.tilesheet_id.prefix.clone(),
                                postfix: mapped_id.tilesheet_id.postfix.clone(),
                            },
                            rotation: mapped_id.rotation.clone(),
                            is_broken: mapped_id.is_broken,
                            is_open: mapped_id.is_open,
                        },
                    };

                    let sprite = tilesheet.get_sprite(&id, &json_data);

                    let adjacent_idents = local_mapped_cdda_ids
                        .get_adjacent_identifiers(tile_3d_coords, &layer);

                    let (fg, bg) = match sprite {
                        None => {
                            let fallback =
                                tilesheet.get_fallback(&id, &json_data);
                            let position_uvec2 = UVec2::new(
                                tile_3d_coords.x as u32,
                                tile_3d_coords.y as u32,
                            );

                            (
                                Some(DisplaySprite::Fallback(FallbackSprite {
                                    position: UVec2JsonKey(position_uvec2),
                                    index: fallback,
                                    z: tile_3d_coords.z,
                                })),
                                None,
                            )
                        },
                        Some(sprite) => {
                            DisplaySprite::get_display_sprite_from_sprite(
                                &sprite,
                                &id,
                                tile_3d_coords.clone(),
                                layer.clone(),
                                &adjacent_idents,
                                json_data,
                            )
                        },
                    };

                    layer_map.insert(layer.clone(), (fg, bg));
                }

                layer_map
            })
            .collect();

        tile_map.into_iter().for_each(|mut layer_map| {
            for tile_layer in TileLayer::iter() {
                match layer_map.remove(&tile_layer) {
                    None => {},
                    Some((fg, bg)) => {
                        if let Some(fg) = fg {
                            insert_sprite_type!(fg);
                        }
                        if let Some(bg) = bg {
                            insert_sprite_type!(bg);
                        }
                    },
                }
            }
        });

        saved_cdda_ids.insert(*z, local_mapped_cdda_ids);
    }

    let mut mapped_cdda_ids_lock = mapped_cdda_ids.lock().await;
    mapped_cdda_ids_lock.replace(saved_cdda_ids);

    app.emit(
        events::PLACE_SPRITES,
        PlaceSpritesEvent {
            static_sprites,
            animated_sprites,
            fallback_sprites,
        },
    )
    .unwrap();

    Ok(())
}

#[derive(Debug, Error)]
pub enum ReloadProjectError {
    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error(transparent)]
    ProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    GetLiveViewerError(#[from] GetLiveViewerDataError),

    #[error(transparent)]
    CalculateParametersError(#[from] CalculateParametersError),
}

impl_serialize_for_error!(ReloadProjectError);

#[tauri::command]
pub async fn reload_project(
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), ReloadProjectError> {
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;
    let mut editor_data_lock = editor_data.lock().await;
    let project = get_current_project_mut(&mut editor_data_lock)?;

    match &project.ty {
        ProjectType::MapEditor(_) => unimplemented!(),
        ProjectType::LiveViewer(lvd) => {
            let mut map_data_collection =
                get_map_data_collection_live_viewer_data(lvd).await?;

            for (_, map_data) in map_data_collection.iter_mut() {
                map_data.calculate_parameters(&json_data.palettes)?
            }

            project.maps = map_data_collection;
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

    let project = match editor_data_lock
        .projects
        .iter_mut()
        .find(|p| p.name == name)
    {
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

#[derive(Debug, Error, Serialize)]
pub enum GetProjectCellDataError {
    #[error(transparent)]
    MapError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error("No map is opened")]
    NoMapOpened,
}

#[tauri::command]
pub async fn get_project_cell_data(
    mapped_cdda_ids: State<
        '_,
        Mutex<Option<HashMap<ZLevel, MappedCDDAIdContainer>>>,
    >,
) -> Result<HashMap<ZLevel, MappedCDDAIdContainer>, GetProjectCellDataError> {
    let mapped_cdda_ids_lock = mapped_cdda_ids.lock().await;
    let mapped_cdda_ids = match mapped_cdda_ids_lock.deref() {
        None => return Err(GetProjectCellDataError::NoMapOpened),
        Some(m) => m,
    };

    Ok(mapped_cdda_ids.clone())
}

#[tauri::command]
pub async fn close_project(
    app: AppHandle,
    name: String,
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

    let project_index = editor_data_lock
        .projects
        .iter()
        .position(|p| p.name == name)
        .unwrap();

    editor_data_lock.projects.remove(project_index);

    let saver = EditorDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(&editor_data_lock).await.unwrap();

    Ok(())
}

#[derive(Debug, Error)]
pub enum NewMapgenViewerError {
    #[error(transparent)]
    OpenViewerError(#[from] OpenViewerError),
}

impl_serialize_for_error!(NewMapgenViewerError);

#[tauri::command]
pub async fn new_single_mapgen_viewer(
    path: PathBuf,
    om_terrain_name: String,
    project_name: String,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), NewMapgenViewerError> {
    let data = serde_json::to_string_pretty(&json!(
        [
            {
                "type": "mapgen",
                "method": "json",
                "om_terrain": om_terrain_name,
                "object": {
                    "fill_ter": "t_region_groundcover",
                    "rows": DEFAULT_MAP_ROWS
                }
            }
        ]
    ))
    .unwrap();

    let mut file = File::create(&path).await.unwrap();

    file.write_all(data.as_bytes()).await.unwrap();

    open_viewer(
        app,
        OpenViewerData::Terrain {
            mapgen_file_paths: vec![path],
            project_name,
            om_id: CDDAIdentifier(om_terrain_name),
        },
        editor_data,
        json_data,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn new_special_mapgen_viewer(
    path: PathBuf,
    om_terrain_name: String,
    project_name: String,
    special_width: Bound_usize<1, { usize::MAX }>,
    special_height: Bound_usize<1, { usize::MAX }>,
    special_z_from: i32,
    special_z_to: i32,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), NewMapgenViewerError> {
    let mut data = Vec::new();

    let mut overmaps_list = Vec::new();

    for z in special_z_from..=special_z_to {
        for y in 0..special_height.get() {
            for x in 0..special_width.get() {
                let om_terrain_name =
                    format!("{}_{}_{}_{}", om_terrain_name, x, y, z);

                overmaps_list.push(json!({
                   "point": [x, y, z],
                    "overmap": om_terrain_name,
                }));
            }
        }
    }

    data.push(json!({
        "type": "overmap_special",
        "id": om_terrain_name,
        "overmaps": overmaps_list
    }));

    for z in special_z_from..=special_z_to {
        let mut z_om_terrain_names = Vec::new();
        z_om_terrain_names.reserve(special_height.get());

        for y in 0..special_height.get() {
            let mut y_om_terrain_names = Vec::new();
            y_om_terrain_names.reserve(special_width.get());

            for x in 0..special_width.get() {
                let om_terrain_name =
                    format!("{}_{}_{}_{}", om_terrain_name, x, y, z);
                y_om_terrain_names.push(om_terrain_name.clone());
            }

            z_om_terrain_names.push(y_om_terrain_names)
        }

        let mut rows = Vec::new();

        for _ in 0..special_height.get() * DEFAULT_MAP_HEIGHT {
            rows.push(DEFAULT_EMPTY_CHAR_ROW.repeat(special_width.get()));
        }

        data.push(json!({
            "type": "mapgen",
            "method": "json",
            "om_terrain": z_om_terrain_names,
            "object": {
                "fill_ter": "t_region_groundcover",
                "rows": rows
            }
        }));
    }

    let data_ser = serde_json::to_string_pretty(&data).unwrap();
    let mut file = File::create(&path).await.unwrap();
    file.write_all(data_ser.as_bytes()).await.unwrap();

    open_viewer(
        app,
        OpenViewerData::Special {
            mapgen_file_paths: vec![path.clone()],
            om_file_paths: vec![path.clone()],
            project_name,
            om_id: CDDAIdentifier(om_terrain_name),
        },
        editor_data,
        json_data,
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn new_nested_mapgen_viewer(
    path: PathBuf,
    om_terrain_name: String,
    project_name: String,
    nested_width: Bound_usize<1, 24>,
    nested_height: Bound_usize<1, 24>,
    app: AppHandle,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), NewMapgenViewerError> {
    let mut rows = Vec::new();

    for _ in 0..nested_height.get() {
        rows.push(SPECIAL_EMPTY_CHAR.to_string().repeat(nested_width.get()));
    }

    let data = json!(
        [
            {
            "type": "mapgen",
            "method": "json",
            "nested_mapgen_id": om_terrain_name,
            "object": {
                    "mapgensize": [nested_width, nested_height],
                    "fill_ter": "t_region_groundcover",
                    "rows": rows
                }
            }
        ]
    );

    let data_ser = serde_json::to_string_pretty(&data).unwrap();
    let mut file = File::create(&path).await.unwrap();
    file.write_all(data_ser.as_bytes()).await.unwrap();

    open_viewer(
        app,
        OpenViewerData::Terrain {
            mapgen_file_paths: vec![path.clone()],
            project_name,
            om_id: CDDAIdentifier(om_terrain_name),
        },
        editor_data,
        json_data,
    )
    .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "type"
)]
pub enum OpenViewerData {
    Terrain {
        mapgen_file_paths: Vec<PathBuf>,
        project_name: String,
        om_id: CDDAIdentifier,
    },
    Special {
        mapgen_file_paths: Vec<PathBuf>,
        om_file_paths: Vec<PathBuf>,
        project_name: String,
        om_id: CDDAIdentifier,
    },
}

#[derive(Debug, Error)]
pub enum OpenViewerError {
    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),

    #[error(transparent)]
    TauriError(#[from] tauri::Error),

    #[error("Another project with the same name already exists")]
    ProjectAlreadyExists,

    #[error(transparent)]
    CalculateParametersError(#[from] CalculateParametersError),
}
impl_serialize_for_error!(OpenViewerError);

#[tauri::command]
pub async fn open_viewer(
    app: AppHandle,
    data: OpenViewerData,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<(), OpenViewerError> {
    info!("Opening Live viewer");

    let mut editor_data_lock = editor_data.lock().await;
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;

    match data {
        OpenViewerData::Terrain {
            project_name,
            mapgen_file_paths,
            om_id,
        } => {
            if editor_data_lock
                .projects
                .iter()
                .find(|p| p.name == project_name)
                .is_some()
            {
                return Err(OpenViewerError::ProjectAlreadyExists);
            }

            let mut overmap_terrain_importer = SingleMapDataImporter {
                om_terrain: om_id.clone(),
                paths: mapgen_file_paths.clone(),
            };

            let mut collection = overmap_terrain_importer.load().await.unwrap();
            collection.calculate_parameters(&json_data.palettes)?;

            let mut new_project = Project::new(
                project_name.clone(),
                DEFAULT_MAP_DATA_SIZE,
                ProjectType::LiveViewer(LiveViewerData::Terrain {
                    mapgen_file_paths,
                    project_name: project_name.clone(),
                    om_id,
                }),
            );

            new_project.maps.insert(0, collection);
            editor_data_lock.projects.push(new_project);

            editor_data_lock.opened_project = Some(project_name.clone());
            app.emit(
                events::TAB_CREATED,
                Tab {
                    name: project_name.clone(),
                    tab_type: TabType::LiveViewer,
                },
            )?;
        },
        OpenViewerData::Special {
            project_name,
            mapgen_file_paths,
            om_file_paths,
            om_id,
        } => {
            if editor_data_lock
                .projects
                .iter()
                .find(|p| p.name == project_name)
                .is_some()
            {
                return Err(OpenViewerError::ProjectAlreadyExists);
            }

            let mut overmap_special_importer = OvermapSpecialImporter {
                om_special_id: om_id.clone(),
                overmap_special_paths: om_file_paths.clone(),
                mapgen_entry_paths: mapgen_file_paths.clone(),
            };

            let mut maps = overmap_special_importer.load().await.unwrap();

            for (_, m) in maps.iter_mut() {
                m.calculate_parameters(&json_data.palettes)?
            }

            let mut new_project = Project::new(
                project_name.clone(),
                get_size(&maps),
                ProjectType::LiveViewer(LiveViewerData::Special {
                    mapgen_file_paths,
                    om_file_paths,
                    project_name: project_name.clone(),
                    om_id,
                }),
            );

            new_project.maps = maps;
            editor_data_lock.projects.push(new_project);

            editor_data_lock.opened_project = Some(project_name.clone());
            app.emit(
                events::TAB_CREATED,
                Tab {
                    name: project_name.clone(),
                    tab_type: TabType::LiveViewer,
                },
            )?;
        },
    }

    let saver = EditorDataSaver {
        path: editor_data_lock.config.config_path.clone(),
    };

    saver.save(editor_data_lock.deref()).await.unwrap();

    Ok(())
}
