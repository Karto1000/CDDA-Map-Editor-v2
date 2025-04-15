use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::TileLayer;
use crate::editor_data::tab::handlers::create_tab;
use crate::editor_data::tab::ProjectState::Saved;
use crate::editor_data::tab::{ProjectState, TabType};
use crate::editor_data::{EditorData, EditorDataSaver, Project};
use crate::map_data::io::ProjectSaver;
use crate::map_data::{MapData, ProjectContainer};
use crate::tileset::legacy_tileset::{MappedSprite, SpriteIndex, SpriteLayer};
use crate::tileset::{SpriteKind, Tilesheet, TilesheetKind};
use crate::util::{CDDAIdentifier, GetIdentifier, JSONSerializableUVec2, Save};
use glam::{UVec2, UVec3};
use image::imageops::tile;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    static_sprites: Vec<StaticSprite>,
    animated_sprites: Vec<AnimatedSprite>,
    fallback_sprites: Vec<FallbackSprite>,
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AnimatedSprite {
    pub position: JSONSerializableUVec2,
    pub indices: Vec<u32>,
    pub layer: u32,
    pub z: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FallbackSprite {
    pub position: JSONSerializableUVec2,
    pub index: u32,
    pub z: i32,
}

fn get_id_from_mapped_sprites(
    mapped_sprites_lock: &MutexGuard<HashMap<UVec3, MappedSprite>>,
    cords: &UVec3,
    layer: &TileLayer,
) -> Option<CDDAIdentifier> {
    mapped_sprites_lock
        .get(cords)
        .map(|v| match layer {
            TileLayer::Terrain => v.terrain.clone(),
            TileLayer::Furniture => v.furniture.clone(),
        })
        .flatten()
}

#[tauri::command]
pub async fn open_project(
    index: usize,
    app: AppHandle,
    tilesheet: State<'_, Mutex<Option<TilesheetKind>>>,
    project_container: State<'_, Mutex<ProjectContainer>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    mapped_sprites: State<'_, Mutex<HashMap<UVec3, MappedSprite>>>,
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

    let mut static_sprites = vec![];
    let mut animated_sprites = vec![];
    let mut fallback_sprites = vec![];

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
                };
            }

            mapped_sprites_lock.insert(UVec3::new(p.x, p.y, *z as u32), mapped_sprite);
            identifiers.insert(p, new_identifier_group);
        });

        map_data.cells.iter().for_each(|(p, cell)| {
            let identifier_group = identifiers.remove(p).expect("Identifier group to exist");
            let mapped_sprite_coords = UVec3::new(p.x, p.y, *z as u32);

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

                let top_cords = mapped_sprite_coords + UVec3::new(0, 1, 0);
                let top = get_id_from_mapped_sprites(&mapped_sprites_lock, &top_cords, &layer);

                let right_cords = mapped_sprite_coords + UVec3::new(1, 0, 0);
                let right = get_id_from_mapped_sprites(&mapped_sprites_lock, &right_cords, &layer);

                // We need this to protect against underflow of coordinates since they're stored as a UVec2
                let bottom = match p.y > 0 {
                    true => {
                        let bottom_cords = mapped_sprite_coords - UVec3::new(0, 1, 0);
                        get_id_from_mapped_sprites(&mapped_sprites_lock, &bottom_cords, &layer)
                    }
                    false => None,
                };

                let left = match p.x > 0 {
                    true => {
                        let left_cords = mapped_sprite_coords - UVec3::new(1, 0, 0);
                        get_id_from_mapped_sprites(&mapped_sprites_lock, &left_cords, &layer)
                    }
                    false => None,
                };

                let sprite_kind = match tilesheet {
                    TilesheetKind::Legacy(l) => l.get_sprite(&id, &json_data),
                    TilesheetKind::Current(c) => c.get_sprite(&id, &json_data),
                };

                match sprite_kind {
                    SpriteKind::Exists(sprite) => {
                        match sprite.get_fg_id(
                            &id,
                            json_data,
                            &layer,
                            top.clone(),
                            right.clone(),
                            bottom.clone(),
                            left.clone(),
                        ) {
                            None => {}
                            // TODO: Mapped Sprite
                            Some(id) => match sprite.is_animated() {
                                true => {
                                    let display_sprite = AnimatedSprite {
                                        position: JSONSerializableUVec2(p.clone()),
                                        layer: (layer.clone() as u32) * 2 + SpriteLayer::Fg as u32,
                                        indices: id.into_vec(),
                                        z: *z,
                                    };

                                    animated_sprites.push(display_sprite)
                                }
                                false => {
                                    let display_sprite = StaticSprite {
                                        position: JSONSerializableUVec2(p.clone()),
                                        layer: (layer.clone() as u32) * 2 + SpriteLayer::Bg as u32,
                                        index: id.into_single().unwrap(),
                                        z: *z,
                                    };

                                    static_sprites.push(display_sprite);
                                }
                            },
                        }

                        match sprite.get_bg_id(&id, json_data, &layer, top, right, bottom, left) {
                            None => {}
                            Some(id) => match sprite.is_animated() {
                                true => {
                                    let display_sprite = AnimatedSprite {
                                        position: JSONSerializableUVec2(p.clone()),
                                        layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                                        indices: id.into_vec(),
                                        z: *z,
                                    };

                                    animated_sprites.push(display_sprite);
                                }
                                false => {
                                    let display_sprite = StaticSprite {
                                        position: JSONSerializableUVec2(p.clone()),
                                        layer: (layer as u32) * 2 + SpriteLayer::Bg as u32,
                                        index: id.into_single().unwrap(),
                                        z: *z,
                                    };

                                    static_sprites.push(display_sprite);
                                }
                            },
                        }
                    }
                    SpriteKind::Fallback(sprite_index) => fallback_sprites.push(FallbackSprite {
                        position: JSONSerializableUVec2(p.clone()),
                        index: sprite_index,
                        z: *z,
                    }),
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

#[tauri::command]
pub async fn close_project(
    project_container: State<'_, Mutex<ProjectContainer>>,
    mapped_sprites: State<'_, Mutex<HashMap<UVec3, MappedSprite>>>,
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
