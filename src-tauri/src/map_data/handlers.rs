use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::TileLayer;
use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::ProjectState::Saved;
use crate::editor_data::tab::{ProjectState, TabType};
use crate::editor_data::{EditorData, EditorDataSaver, Project};
use crate::map_data::io::ProjectSaver;
use crate::map_data::{MapData, PlaceableSetType, ProjectContainer, SetSquare};
use crate::tileset::legacy_tileset::{MappedSprite, SpriteIndex};
use crate::tileset::{SpriteKind, SpriteLayer, Tilesheet, TilesheetKind};
use crate::util::{CDDAIdentifier, GetIdentifier, JSONSerializableUVec2, Save};
use glam::{IVec3, UVec2, UVec3, Vec3};
use image::imageops::tile;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use tauri::async_runtime::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::MutexGuard;

mod events {
    pub const OPENED_MAP: &'static str = "opened_map";
    pub const PLACE_SPRITES: &'static str = "place_sprites";
}

#[derive(Debug, thiserror::Error, Serialize)]
pub enum GetCurrentMapDataError {
    #[error("No map has been opened")]
    NoMapOpened,
    #[error("Invalid map index")]
    InvalidMapIndex(usize),
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
    position: JSONSerializableUVec2,
    character: char,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapChangEvent {
    kind: MapChangeEventKind,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MapChangeEventKind {
    Place(PlaceCommand),
    Delete(JSONSerializableUVec2),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaceSpriteEvent {
    position: JSONSerializableUVec2,
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
    size: JSONSerializableUVec2,
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
    pub position: JSONSerializableUVec2,
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
    pub position: JSONSerializableUVec2,
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
    pub position: JSONSerializableUVec2,
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

fn get_id_from_mapped_sprites(
    mapped_sprites_lock: &MutexGuard<HashMap<IVec3, MappedSprite>>,
    cords: &IVec3,
    layer: &TileLayer,
) -> Option<CDDAIdentifier> {
    mapped_sprites_lock
        .get(cords)
        .map(|v| match layer {
            TileLayer::Terrain => v.terrain.clone(),
            TileLayer::Furniture => v.furniture.clone(),
            TileLayer::Trap => v.trap.clone(),
        })
        .flatten()
}

#[derive(Debug)]
pub enum SpriteType {
    Static(StaticSprite),
    Animated(AnimatedSprite),
    Fallback(FallbackSprite),
}

pub fn get_fg_from_sprite(
    id: &CDDAIdentifier,
    position: IVec3,
    json_data: &DeserializedCDDAJsonData,
    layer: TileLayer,
    sprite_kind: &SpriteKind,
    mapped_sprites_lock: &mut MutexGuard<HashMap<IVec3, MappedSprite>>,
) -> Option<SpriteType> {
    let (top, right, bottom, left) =
        get_adjacent_coordinates(mapped_sprites_lock, position, &layer);
    let position_uvec2 = UVec2::new(position.x as u32, position.y as u32);

    match sprite_kind {
        SpriteKind::Exists(sprite) => {
            match sprite.get_fg_id(&id, json_data, &layer, top, right, bottom, left) {
                None => None,
                Some(id) => match sprite.is_animated() {
                    true => {
                        let display_sprite = AnimatedSprite {
                            position: JSONSerializableUVec2(position_uvec2),
                            layer: (layer as u32) * 2 + SpriteLayer::Fg as u32,
                            indices: id.data.into_vec(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Animated(display_sprite))
                    }
                    false => {
                        let display_sprite = StaticSprite {
                            position: JSONSerializableUVec2(position_uvec2),
                            layer: (layer as u32) * 2 + SpriteLayer::Fg as u32,
                            index: id.data.into_single().unwrap(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Static(display_sprite))
                    }
                },
            }
        }
        SpriteKind::Fallback(sprite_index) => Some(SpriteType::Fallback(FallbackSprite {
            position: JSONSerializableUVec2(position_uvec2),
            index: *sprite_index,
            z: position.z,
        })),
    }
}

pub fn get_bg_from_sprite(
    id: &CDDAIdentifier,
    position: IVec3,
    json_data: &DeserializedCDDAJsonData,
    layer: TileLayer,
    sprite_kind: &SpriteKind,
    mapped_sprites_lock: &mut MutexGuard<HashMap<IVec3, MappedSprite>>,
) -> Option<SpriteType> {
    let (top, right, bottom, left) =
        get_adjacent_coordinates(mapped_sprites_lock, position, &layer);
    let position_uvec2 = UVec2::new(position.x as u32, position.y as u32);

    match sprite_kind {
        SpriteKind::Exists(sprite) => {
            match sprite.get_bg_id(&id, json_data, &layer, top, right, bottom, left) {
                None => None,
                Some(id) => match sprite.is_animated() {
                    true => {
                        let display_sprite = AnimatedSprite {
                            position: JSONSerializableUVec2(position_uvec2),
                            layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                            indices: id.data.into_vec(),
                            rotate_deg: id.rotation.deg(),
                            z: position.z,
                        };

                        Some(SpriteType::Animated(display_sprite))
                    }
                    false => {
                        let display_sprite = StaticSprite {
                            position: JSONSerializableUVec2(position_uvec2),
                            layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                            index: id.data.into_single().unwrap(),
                            z: position.z,
                            rotate_deg: id.rotation.deg(),
                        };

                        Some(SpriteType::Static(display_sprite))
                    }
                },
            }
        }
        SpriteKind::Fallback(sprite_index) => Some(SpriteType::Fallback(FallbackSprite {
            position: JSONSerializableUVec2(position_uvec2),
            index: *sprite_index,
            z: position.z,
        })),
    }
}

#[tauri::command]
pub async fn open_project(
    index: usize,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<TilesheetKind>>>,
    project_container: State<'_, Mutex<ProjectContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    mapped_sprites: State<'_, Mutex<HashMap<IVec3, MappedSprite>>>,
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

    let tilesheet_lock = tilesheet.lock().await;
    let tilesheet = match tilesheet_lock.as_ref() {
        None => return Err(()),
        Some(t) => t,
    };

    app.emit(events::OPENED_MAP, index)
        .expect("Function to not fail");

    let region_settings = json_data
        .region_settings
        .get(&CDDAIdentifier("default".into()))
        .expect("Region settings to exist");

    let mut static_sprites = HashSet::new();
    let mut animated_sprites = HashSet::new();
    let mut fallback_sprites = HashSet::new();

    for (z, map_data) in project.maps.iter() {
        let fill_terrain_sprite = match &map_data.fill {
            None => None,
            Some(id) => {
                let id = id.get_identifier(&map_data.calculated_parameters);

                Some(id.as_final_id(&region_settings, &json_data.terrain, &json_data.furniture))
            }
        };

        let mut mapped_sprites_lock = mapped_sprites.lock().await;

        // Store the identifiers here since they use random numbers to determine them and if we used
        // this function in the next iteration through map_data, we would get other identifiers
        let mut identifiers = HashMap::new();

        for set in map_data.set.iter() {
            let sprites = set.get_sprites(*z, tilesheet, json_data, &mut mapped_sprites_lock);

            for sprite in sprites {
                match sprite {
                    SpriteType::Static(s) => {
                        static_sprites.replace(s);
                    }
                    SpriteType::Animated(a) => {
                        animated_sprites.replace(a);
                    }
                    SpriteType::Fallback(f) => {
                        fallback_sprites.replace(f);
                    }
                }
            }
        }

        // We need to insert the mapped_sprite before we get the fg and bg of this sprite since
        // the function relies on the mapped sprite of this sprite to already exist
        map_data.cells.iter().for_each(|(p, cell)| {
            let mut identifier_group =
                map_data.get_identifiers(&cell.character, &json_data.palettes);

            match identifier_group.terrain {
                None => {
                    debug!(
                        "No terrain found for {}, trying to use fill_sprite",
                        cell.character
                    );

                    match &fill_terrain_sprite {
                        None => {
                            warn!("terrain was not defined for {}", cell.character);
                            todo!();
                        }
                        Some(fill_sprite) => {
                            identifier_group.terrain = Some(fill_sprite.clone());
                        }
                    };
                }
                Some(_) => {}
            }

            let mut mapped_sprite = MappedSprite::default();
            let mut new_identifier_group = identifier_group.clone();

            for (layer, o_id) in [
                (TileLayer::Terrain, &identifier_group.terrain),
                (TileLayer::Furniture, &identifier_group.furniture),
            ] {
                let id = match o_id {
                    None => continue,
                    Some(id) => {
                        id.as_final_id(region_settings, &json_data.terrain, &json_data.furniture)
                    }
                };

                match layer {
                    TileLayer::Terrain => {
                        mapped_sprite.terrain = Some(id.clone());
                        new_identifier_group.terrain = Some(id.clone());
                    }
                    TileLayer::Furniture => {
                        mapped_sprite.furniture = Some(id.clone());
                        new_identifier_group.furniture = Some(id.clone());
                    }
                    _ => unreachable!(),
                };
            }

            let mapped_cords = IVec3::new(p.x as i32, p.y as i32, *z);
            match mapped_sprites_lock.get(&mapped_cords) {
                None => {
                    mapped_sprites_lock.insert(mapped_cords, mapped_sprite);
                }
                Some(_) => {}
            }

            identifiers.insert(p, new_identifier_group);
        });

        map_data.cells.iter().for_each(|(p, cell)| {
            let identifier_group = identifiers.remove(p).expect("Identifier group to exist");
            let mapped_sprite_coords = IVec3::new(p.x as i32, p.y as i32, *z);

            if identifier_group.terrain.is_none() && identifier_group.furniture.is_none() {
                warn!("No sprites found for char {:?}", cell);
                return;
            }

            // Layer here is done so furniture is above terrain
            for (layer, o_id) in [
                (TileLayer::Terrain, identifier_group.terrain),
                (TileLayer::Furniture, identifier_group.furniture),
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

                if let Some(fg) = get_fg_from_sprite(
                    &id,
                    mapped_sprite_coords.clone(),
                    json_data,
                    layer.clone(),
                    &sprite_kind,
                    &mut mapped_sprites_lock,
                ) {
                    match fg {
                        SpriteType::Static(s) => {
                            static_sprites.insert(s);
                        }
                        SpriteType::Animated(a) => {
                            animated_sprites.insert(a);
                        }
                        SpriteType::Fallback(f) => {
                            fallback_sprites.insert(f);
                        }
                    }
                }

                if let Some(bg) = get_bg_from_sprite(
                    &id,
                    mapped_sprite_coords.clone(),
                    json_data,
                    layer.clone(),
                    &sprite_kind,
                    &mut mapped_sprites_lock,
                ) {
                    match bg {
                        SpriteType::Static(s) => {
                            static_sprites.insert(s);
                        }
                        SpriteType::Animated(a) => {
                            animated_sprites.insert(a);
                        }
                        SpriteType::Fallback(f) => {
                            fallback_sprites.insert(f);
                        }
                    }
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

fn get_adjacent_coordinates(
    mapped_sprites_lock: &mut MutexGuard<HashMap<IVec3, MappedSprite>>,
    coordinates: IVec3,
    layer: &TileLayer,
) -> (
    Option<CDDAIdentifier>,
    Option<CDDAIdentifier>,
    Option<CDDAIdentifier>,
    Option<CDDAIdentifier>,
) {
    let top_cords = coordinates + IVec3::new(0, 1, 0);
    let top = get_id_from_mapped_sprites(&mapped_sprites_lock, &top_cords, &layer);

    let right_cords = coordinates + IVec3::new(1, 0, 0);
    let right = get_id_from_mapped_sprites(&mapped_sprites_lock, &right_cords, &layer);

    let bottom = match coordinates.y > 0 {
        true => {
            let bottom_cords = coordinates - IVec3::new(0, 1, 0);
            get_id_from_mapped_sprites(&mapped_sprites_lock, &bottom_cords, &layer)
        }
        false => None,
    };

    let left = match coordinates.x > 0 {
        true => {
            let left_cords = coordinates - IVec3::new(1, 0, 0);
            get_id_from_mapped_sprites(&mapped_sprites_lock, &left_cords, &layer)
        }
        false => None,
    };

    (top, right, bottom, left)
}

#[tauri::command]
pub async fn close_project(
    project_container: State<'_, Mutex<ProjectContainer>>,
    mapped_sprites: State<'_, Mutex<HashMap<IVec3, MappedSprite>>>,
) -> Result<(), ()> {
    let mut lock = project_container.lock().await;
    lock.current_project = None;
    mapped_sprites.lock().await.clear();
    Ok(())
}

#[derive(Debug, thiserror::Error, Serialize)]
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
    let lock = project_container.lock().await;
    let map_data = get_current_project(&lock)?;

    let save_path = r"C:\Users\Kartoffelbauer\Downloads\test";
    let saver = ProjectSaver {
        path: save_path.into(),
    };

    // saver.save(&map_data).map_err(|e| {
    //     error!("{}", e);
    //     SaveError::SaveError
    // })?;

    let mut editor_data = editor_data.lock().await;
    let new_tab_type = TabType::MapEditor(Saved {
        path: PathBuf::from(save_path).join(&map_data.name),
    });
    editor_data
        .tabs
        .get_mut(lock.current_project.unwrap())
        .unwrap()
        .tab_type = new_tab_type;

    let editor_data_saver = EditorDataSaver {
        path: editor_data.config.config_path.clone(),
    };

    editor_data_saver
        .save(&editor_data)
        .expect("Saving to not fail");

    Ok(())
}
