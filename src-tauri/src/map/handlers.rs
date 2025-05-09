use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::TileLayer;
use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::{ProjectState, TabType};
use crate::editor_data::{EditorData, Project};
use crate::map::{
    CellRepresentation, ProjectContainer, VisibleMappingCommand, VisibleMappingCommandKind,
    VisibleMappingKind,
};
use crate::tileset;
use crate::tileset::legacy_tileset::MappedCDDAIds;
use crate::tileset::{AdjacentSprites, SpriteKind, SpriteLayer, Tilesheet, TilesheetKind};
use crate::util::{CDDAIdentifier, GetIdentifier, IVec3JsonKey, UVec2JsonKey};
use glam::{IVec3, UVec2};
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::sync::MutexGuard;

mod events {
    pub const OPENED_MAP: &'static str = "opened_map";
    pub const PLACE_SPRITES: &'static str = "place_sprites";
    pub const ITEM_DATA: &'static str = "item_data";
}

#[derive(Debug, Error, Serialize)]
pub enum GetCurrentMapDataError {
    #[error("No map has been opened")]
    NoMapOpened,
    #[error("Invalid map index")]
    InvalidMapIndex(usize),
}

#[derive(Debug, Error, Serialize)]
pub enum CDDADataError {
    #[error("No CDDA Data was loaded")]
    NotLoaded,
}

fn get_current_project<'a>(
    project_container: &'a MutexGuard<ProjectContainer>,
) -> Result<&'a Project, GetCurrentMapDataError> {
    let map_index = match project_container.current_project {
        None => return Err(GetCurrentMapDataError::NoMapOpened),
        Some(i) => i,
    };

    let data = match project_container.data.get(map_index) {
        None => return Err(GetCurrentMapDataError::InvalidMapIndex(map_index)),
        Some(d) => d,
    };

    Ok(data)
}

fn get_current_project_mut<'a>(
    project_container: &'a mut MutexGuard<ProjectContainer>,
) -> Result<&'a mut Project, GetCurrentMapDataError> {
    let map_index = match project_container.current_project {
        None => return Err(GetCurrentMapDataError::NoMapOpened),
        Some(i) => i,
    };

    let data = match project_container.data.get_mut(map_index) {
        None => return Err(GetCurrentMapDataError::InvalidMapIndex(map_index)),
        Some(d) => d,
    };

    Ok(data)
}

fn get_json_data<'a>(
    lock: &'a MutexGuard<Option<DeserializedCDDAJsonData>>,
) -> Result<&'a DeserializedCDDAJsonData, CDDADataError> {
    match lock.deref() {
        None => Err(CDDADataError::NotLoaded),
        Some(d) => Ok(d),
    }
}

#[tauri::command]
pub async fn get_current_project_data(
    map_data: State<'_, Mutex<ProjectContainer>>,
) -> Result<Project, GetCurrentMapDataError> {
    let lock = map_data.lock().await;
    let data = get_current_project(&lock)?;

    Ok(data.clone())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceCommand {
    position: UVec2JsonKey,
    character: char,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapChangEvent {
    kind: MapChangeEventKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MapChangeEventKind {
    Place(PlaceCommand),
    Delete(UVec2JsonKey),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceSpriteEvent {
    position: UVec2JsonKey,
    index: u32,
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
}

#[tauri::command]
pub async fn create_project(
    app: AppHandle,
    data: CreateMapData,
    project_container: State<'_, Mutex<ProjectContainer>>,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), ()> {
    let mut lock = project_container.lock().await;

    let project_name = lock.data.iter().fold(data.name, |name, project| {
        if project.name == name {
            let (name, num) = name.rsplit_once("_").unwrap_or((name.as_str(), "0"));
            let number: u32 = num.parse().unwrap_or(0) + 1;
            let new_name = format!("{}_{}", name, number);

            return new_name;
        }

        name
    });

    let project = Project::new(project_name, data.size.0);
    lock.data.push(project.clone());

    create_tab(
        project.name,
        TabType::MapEditor(ProjectState::Unsaved),
        app,
        editor_data,
    )
    .await
    .expect("Function to not fail");

    Ok(())
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
        self.position.eq(&other.position) && self.layer.eq(&other.layer) && self.z.eq(&other.z)
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
        self.position.eq(&other.position) && self.layer.eq(&other.layer) && self.z.eq(&other.z)
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

pub fn get_sprite_type_from_sprite(
    id: &CDDAIdentifier,
    position: IVec3,
    adjacent_sprites: &AdjacentSprites,
    layer: TileLayer,
    sprite_kind: &SpriteKind,
    // ----
    json_data: &DeserializedCDDAJsonData,
) -> (Option<SpriteType>, Option<SpriteType>) {
    let position_uvec2 = UVec2::new(position.x as u32, position.y as u32);

    match sprite_kind {
        SpriteKind::Exists(sprite) => {
            let fg = match sprite.get_fg_id(&id, json_data, &layer, adjacent_sprites) {
                None => None,
                Some(id) => match sprite.is_animated() {
                    true => {
                        let display_sprite = AnimatedSprite {
                            position: UVec2JsonKey(position_uvec2),
                            layer: (layer.clone() as u32) * 2 + SpriteLayer::Fg as u32,
                            indices: id.data.into_vec(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Animated(display_sprite))
                    }
                    false => {
                        let display_sprite = StaticSprite {
                            position: UVec2JsonKey(position_uvec2),
                            layer: (layer.clone() as u32) * 2 + SpriteLayer::Fg as u32,
                            index: id.data.into_single().unwrap(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Static(display_sprite))
                    }
                },
            };

            let bg = match sprite.get_bg_id(&id, json_data, &layer, adjacent_sprites) {
                None => None,
                Some(id) => match sprite.is_animated() {
                    true => {
                        let display_sprite = AnimatedSprite {
                            position: UVec2JsonKey(position_uvec2),
                            layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                            indices: id.data.into_vec(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Animated(display_sprite))
                    }
                    false => {
                        let display_sprite = StaticSprite {
                            position: UVec2JsonKey(position_uvec2),
                            layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                            index: id.data.into_single().unwrap(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Static(display_sprite))
                    }
                },
            };

            (fg, bg)
        }
        SpriteKind::Fallback(sprite_index) => (
            Some(SpriteType::Fallback(FallbackSprite {
                position: UVec2JsonKey(position_uvec2),
                index: *sprite_index,
                z: position.z,
            })),
            None,
        ),
    }
}

#[tauri::command]
pub async fn open_project(
    index: usize,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<TilesheetKind>>>,
    project_container: State<'_, Mutex<ProjectContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    mapped_cdda_ids: State<'_, Mutex<HashMap<IVec3, MappedCDDAIds>>>,
) -> Result<(), ()> {
    let mut lock = project_container.lock().await;

    let guard = json_data.lock().await;
    let json_data = match guard.deref() {
        None => return Err(()),
        Some(d) => d,
    };

    lock.current_project = Some(index);

    let project = match lock.data.get(index) {
        None => {
            warn!("Could not find map at index {}", index);
            return Err(());
        }
        Some(d) => d,
    };

    let region_settings = json_data
        .region_settings
        .get(&CDDAIdentifier("default".into()))
        .expect("Region settings to exist");

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(()),
        Some(t) => t,
    };

    app.emit(events::OPENED_MAP, index)
        .expect("Function to not fail");

    let mut static_sprites = HashSet::new();
    let mut animated_sprites = HashSet::new();
    let mut fallback_sprites = HashSet::new();

    macro_rules! insert_sprite_type {
        ($val: expr) => {
            match $val {
                SpriteType::Static(s) => {
                    static_sprites.insert(s);
                }
                SpriteType::Animated(a) => {
                    animated_sprites.insert(a);
                }
                SpriteType::Fallback(f) => {
                    // fallback_sprites.insert(f);
                }
            }
        };
    }

    for (z, map_data) in project.maps.iter() {
        let local_mapped_cdda_ids = map_data.get_mapped_cdda_ids(json_data, *z);

        for (p, identifier_group) in local_mapped_cdda_ids.iter() {
            let cell_3d_coords = IVec3::new(p.x, p.y, *z);

            if identifier_group.terrain.is_none() && identifier_group.furniture.is_none() {
                warn!(
                    "No sprites found for identifier_group {:?}",
                    identifier_group
                );
                continue;
            }

            // Layer here is done so furniture is above terrain
            for (layer, o_id) in [
                (TileLayer::Terrain, &identifier_group.terrain),
                (TileLayer::Furniture, &identifier_group.furniture),
                (TileLayer::Trap, &identifier_group.trap),
                (TileLayer::Monster, &identifier_group.monster),
            ] {
                let id = match o_id {
                    None => continue,
                    Some(id) => {
                        id.as_final_id(region_settings, &json_data.terrain, &json_data.furniture)
                    }
                };

                let sprite_kind = match tilesheet {
                    TilesheetKind::Legacy(l) => l.get_sprite(&id, &json_data),
                    TilesheetKind::Current(c) => c.get_sprite(&id, &json_data),
                };

                let adjacent_sprites = tileset::get_adjacent_sprites(
                    &local_mapped_cdda_ids,
                    cell_3d_coords.clone(),
                    &layer,
                );

                let (fg, bg) = get_sprite_type_from_sprite(
                    &id,
                    cell_3d_coords.clone(),
                    &adjacent_sprites,
                    layer.clone(),
                    &sprite_kind,
                    json_data,
                );

                if let Some(fg) = fg {
                    insert_sprite_type!(fg)
                }

                if let Some(bg) = bg {
                    insert_sprite_type!(bg)
                }
            }
        }
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

#[derive(Debug, Error, Serialize)]
pub enum GetProjectCellDataError {
    #[error(transparent)]
    MapError(#[from] GetCurrentMapDataError),

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),
}

#[tauri::command]
pub async fn get_project_cell_data(
    project_container: State<'_, Mutex<ProjectContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
) -> Result<HashMap<IVec3JsonKey, CellRepresentation>, GetProjectCellDataError> {
    let project_lock = project_container.lock().await;
    let project = get_current_project(&project_lock)?;

    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;

    let mut item_data: HashMap<IVec3JsonKey, CellRepresentation> = HashMap::new();

    for (z, map_data) in project.maps.iter() {
        for (cell_coordinates, cell) in map_data.cells.iter() {
            let cell_data = map_data.get_cell_data(&cell.character, &json_data);

            item_data.insert(
                IVec3JsonKey(IVec3::new(
                    cell_coordinates.x as i32,
                    cell_coordinates.y as i32,
                    *z,
                )),
                cell_data,
            );
        }
    }

    Ok(item_data)
}

#[tauri::command]
pub async fn close_project(
    project_container: State<'_, Mutex<ProjectContainer>>,
    mapped_sprites: State<'_, Mutex<HashMap<IVec3, MappedCDDAIds>>>,
) -> Result<(), ()> {
    let mut lock = project_container.lock().await;
    lock.current_project = None;
    mapped_sprites.lock().await.clear();
    Ok(())
}

#[derive(Debug, Error, Serialize)]
pub enum SaveError {
    #[error(transparent)]
    MapError(#[from] GetCurrentMapDataError),

    #[error("Failed to save map data")]
    SaveError,
}

#[tauri::command]
pub async fn save_current_project(
    project_container: State<'_, Mutex<ProjectContainer>>,
    editor_data: State<'_, Mutex<EditorData>>,
) -> Result<(), SaveError> {
    unimplemented!()
    // let lock = project_container.lock().await;
    // let map_data = get_current_project(&lock)?;
    //
    // let mut editor_data = editor_data.lock().await;
    // let new_tab_type = TabType::MapEditor(Saved {
    //     path: PathBuf::from(save_path).join(&map_data.name),
    // });
    // editor_data
    //     .tabs
    //     .get_mut(lock.current_project.unwrap())
    //     .unwrap()
    //     .tab_type = new_tab_type;
    //
    // let editor_data_saver = EditorDataSaver {
    //     path: editor_data.config.config_path.clone(),
    // };
    //
    // editor_data_saver
    //     .save(&editor_data)
    //     .expect("Saving to not fail");
    //
    // Ok(())
}
