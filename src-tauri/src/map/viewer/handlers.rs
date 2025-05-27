use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::{replace_region_setting, TileLayer};
use crate::editor_data::{
    get_map_data_collection_live_viewer_data, EditorData, EditorDataSaver,
    GetLiveViewerDataError, LiveViewerData, Project, ProjectType,
};
use crate::events::UPDATE_LIVE_VIEWER;
use crate::map::Serializer;
use crate::map::{CalculateParametersError, CellRepresentation};
use crate::tileset::legacy_tileset::{
    LegacyTilesheet, MappedCDDAId, MappedCDDAIdsForTile, TilesheetCDDAId,
};
use crate::tileset::{AdjacentSprites, SpriteKind, SpriteLayer, Tilesheet};
use crate::util::{
    get_current_project_mut, get_json_data, CDDADataError,
    GetCurrentProjectError, IVec3JsonKey, Save, UVec2JsonKey,
};
use crate::{events, impl_serialize_for_error, tileset, util};
use cdda_lib::types::CDDAIdentifier;
use derive_more::Display;
use glam::{IVec3, UVec2};
use log::{debug, error, info, warn};
use notify::{PollWatcher, RecommendedWatcher, Watcher};
use notify_debouncer_full::{new_debouncer, new_debouncer_opt, Debouncer};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use strum::IntoEnumIterator;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::time::Instant;
use tokio_test::block_on;

#[tauri::command]
pub async fn get_current_project_data(
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<Project, GetCurrentProjectError> {
    let editor_data_lock = editor_data.lock().await;
    let data = util::get_current_project(&editor_data_lock)?;
    Ok(data.clone())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceSpritesEvent {
    static_sprites: HashSet<StaticSprite>,
    animated_sprites: HashSet<AnimatedSprite>,
    fallback_sprites: HashSet<FallbackSprite>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMapData {
    name: String,
    size: UVec2JsonKey,
    ty: ProjectType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StaticSprite {
    pub position: UVec2JsonKey,
    pub index: u32,
    pub layer: u32,
    pub z: i32,
    pub rotate_deg: i32,
}

impl Hash for StaticSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.layer.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq<Self> for StaticSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
            && self.layer.eq(&other.layer)
            && self.z.eq(&other.z)
    }
}

impl Eq for StaticSprite {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AnimatedSprite {
    pub position: UVec2JsonKey,
    pub indices: Vec<u32>,
    pub layer: u32,
    pub z: i32,
    pub rotate_deg: i32,
}

impl Hash for AnimatedSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.layer.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq for AnimatedSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position)
            && self.layer.eq(&other.layer)
            && self.z.eq(&other.z)
    }
}

impl Eq for AnimatedSprite {}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FallbackSprite {
    pub position: UVec2JsonKey,
    pub index: u32,
    pub z: i32,
}

impl Hash for FallbackSprite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.z.hash(state);
    }
}

impl PartialEq for FallbackSprite {
    fn eq(&self, other: &Self) -> bool {
        self.position.eq(&other.position) && self.z.eq(&other.z)
    }
}

impl Eq for FallbackSprite {}

#[derive(Debug)]
pub enum SpriteType {
    Static(StaticSprite),
    Animated(AnimatedSprite),
    Fallback(FallbackSprite),
}

impl SpriteType {
    pub fn get_sprite_type_from_sprite_kind(
        sprite_kind: &SpriteKind,
        tile_id: &MappedCDDAId,
        tile_position: IVec3,
        tile_layer: TileLayer,
        adjacent_sprites: &AdjacentSprites,
        json_data: &DeserializedCDDAJsonData,
    ) -> (Option<SpriteType>, Option<SpriteType>) {
        let position_uvec2 =
            UVec2::new(tile_position.x as u32, tile_position.y as u32);

        match sprite_kind {
            SpriteKind::Exists(sprite) => {
                let fg = match sprite.get_fg_id(
                    &tile_id,
                    &tile_layer,
                    adjacent_sprites,
                    json_data,
                ) {
                    None => None,
                    Some(sprite_id) => match sprite.is_animated() {
                        true => {
                            let display_sprite = AnimatedSprite {
                                position: UVec2JsonKey(position_uvec2),
                                layer: (tile_layer.clone() as u32) * 2
                                    + SpriteLayer::Fg as u32,
                                indices: sprite_id.data.into_vec(),
                                rotate_deg: sprite_id.rotation.deg()
                                    + tile_id.rotation.deg(),
                                z: tile_position.z,
                            };

                            Some(SpriteType::Animated(display_sprite))
                        },
                        false => {
                            let display_sprite = StaticSprite {
                                position: UVec2JsonKey(position_uvec2),
                                layer: (tile_layer.clone() as u32) * 2
                                    + SpriteLayer::Fg as u32,
                                index: sprite_id.data.into_single().unwrap(),
                                rotate_deg: sprite_id.rotation.deg(),
                                z: tile_position.z,
                            };

                            Some(SpriteType::Static(display_sprite))
                        },
                    },
                };

                let bg = match sprite.get_bg_id(
                    &tile_id,
                    &tile_layer,
                    adjacent_sprites,
                    json_data,
                ) {
                    None => None,
                    Some(id) => match sprite.is_animated() {
                        true => {
                            let display_sprite = AnimatedSprite {
                                position: UVec2JsonKey(position_uvec2),
                                layer: (tile_layer as u32) * 2
                                    + SpriteLayer::Bg as u32,
                                indices: id.data.into_vec(),
                                rotate_deg: id.rotation.deg(),
                                z: tile_position.z,
                            };

                            Some(SpriteType::Animated(display_sprite))
                        },
                        false => {
                            let display_sprite = StaticSprite {
                                position: UVec2JsonKey(position_uvec2),
                                layer: (tile_layer as u32) * 2
                                    + SpriteLayer::Bg as u32,
                                index: id.data.into_single().unwrap(),
                                rotate_deg: id.rotation.deg(),
                                z: tile_position.z,
                            };

                            Some(SpriteType::Static(display_sprite))
                        },
                    },
                };

                (fg, bg)
            },
            SpriteKind::Fallback(sprite_index) => (
                Some(SpriteType::Fallback(FallbackSprite {
                    position: UVec2JsonKey(position_uvec2),
                    index: *sprite_index,
                    z: tile_position.z,
                })),
                None,
            ),
        }
    }
}

#[tauri::command]
pub async fn get_sprites(
    name: String,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<LegacyTilesheet>>>,
    editor_data: State<'_, Mutex<EditorData>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
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
        ($val: expr) => {
            match $val {
                SpriteType::Static(s) => {
                    static_sprites.insert(s);
                },
                SpriteType::Animated(a) => {
                    animated_sprites.insert(a);
                },
                SpriteType::Fallback(f) => {
                    // fallback_sprites.insert(f);
                },
            }
        };
    }

    for (_, map_collection) in project.maps.iter_mut() {
        // we need to calculate the parameters for the predecessor here because we cannot borrow
        // json data as mutable inside the get_mapped_cdda_ids function
        map_collection.calculate_predecessor_parameters(&mut json_data);
    }

    let region_settings = json_data
        .region_settings
        .get(&CDDAIdentifier("default".into()))
        .expect("Region settings to exist");

    for (z, map_collection) in project.maps.iter() {
        let local_mapped_cdda_ids =
            map_collection.get_mapped_cdda_ids(json_data, *z).unwrap();

        let tile_map: Vec<
            HashMap<TileLayer, (Option<SpriteType>, Option<SpriteType>)>,
        > = local_mapped_cdda_ids
            .ids
            .par_iter()
            .map(|(p, identifier_group)| {
                let cell_3d_coords = IVec3::new(p.x, p.y, *z);

                if identifier_group.terrain.is_none()
                    && identifier_group.furniture.is_none()
                {
                    warn!(
                        "No sprites found for identifier_group {:?} at coordinates {}",
                        identifier_group,
                        cell_3d_coords
                    );

                    return HashMap::new();
                }

                let mut layer_map = HashMap::new();

                // Layer is used here so furniture is above terrain
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

                    let sprite_kind = tilesheet.get_sprite(&id, &json_data);

                    let adjacent_idents = local_mapped_cdda_ids
                        .get_adjacent_identifiers(cell_3d_coords, &layer);

                    let (fg, bg) = SpriteType::get_sprite_type_from_sprite_kind(
                        &sprite_kind,
                        &id,
                        cell_3d_coords.clone(),
                        layer.clone(),
                        &adjacent_idents,
                        json_data,
                    );

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
    }

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
}

#[tauri::command]
pub async fn get_project_cell_data(
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<HashMap<IVec3JsonKey, CellRepresentation>, GetProjectCellDataError>
{
    let json_data_lock = json_data.lock().await;
    let json_data = util::get_json_data(&json_data_lock)?;

    let editor_data_lock = editor_data.lock().await;
    let project = util::get_current_project(&editor_data_lock)?;

    let mut item_data: HashMap<IVec3JsonKey, CellRepresentation> =
        HashMap::new();

    for (z, map_data) in project.maps.iter() {
        let map_cell_data = map_data.get_representations(json_data);

        map_cell_data
            .into_iter()
            .for_each(|(cell_coordinates, cell_data)| {
                item_data.insert(
                    IVec3JsonKey(IVec3::new(
                        cell_coordinates.x as i32,
                        cell_coordinates.y as i32,
                        *z,
                    )),
                    cell_data,
                );
            });
    }

    Ok(item_data)
}

#[tauri::command]
pub async fn close_project(
    app: AppHandle,
    name: String,
    mapped_sprites: State<'_, Mutex<HashMap<IVec3, MappedCDDAIdsForTile>>>,
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

    mapped_sprites.lock().await.clear();
    Ok(())
}
