use crate::data::io::DeserializedCDDAJsonData;
use crate::data::{GetIdentifier, TileLayer};
use crate::features::editor::data::ZLevels;
use crate::features::editor::{MapEditor, MapSize};
use crate::features::map::map_properties::TerrainProperty;
use crate::features::map::{
    MappedCDDAId, MappingKind, Property, Representation,
};
use crate::features::program_data::io::{ProgramDataSaver, ProjectSaver};
use crate::features::program_data::{
    AdjacentTiles, LoadedProjects, Overmap, ProgramData, Project, ProjectType,
    SavedProject, Tab, TabType,
};
use crate::features::tileset::legacy_tileset::{
    LegacyTilesheet, Rotated, SpriteIndex, TilesheetCDDAId,
};
use crate::features::tileset::{ForeBackIds, Tilesheet};
use crate::util::{
    get_current_project_mut, get_json_data, get_size, CDDADataError,
    GetCurrentProjectError, Save, SaveError,
};
use crate::{events, impl_serialize_for_error, InvalidProjectType};
use cdda_lib::types::{CDDAIdentifier, MapGenValue, MeabyVec};
use glam::{IVec2, IVec3, UVec2};
use indexmap::IndexMap;
use log::info;
use rayon::max_num_threads;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::remove_dir;
use std::ops::{Deref, Range};
use std::path::PathBuf;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tauri::{AppHandle, Emitter, State};
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum NewMapEditorError {
    #[error(transparent)]
    TauriError(#[from] tauri::Error),

    #[error(transparent)]
    SaveError(#[from] SaveError),
}

impl_serialize_for_error!(NewMapEditorError);

#[tauri::command(rename_all = "camelCase")]
pub async fn new_map_editor(
    app: AppHandle,
    program_data: State<'_, Mutex<ProgramData>>,
    project_name: String,
    map_size: MapSize,
    z_levels: ZLevels,
    path: PathBuf,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), NewMapEditorError> {
    info!("Creating new map editor");

    let mut program_data_lock = program_data.lock().await;

    let mut map_collection = HashMap::new();
    for z in z_levels.value().0..=z_levels.value().1 {
        let collection =
            Overmap::new(map_size.clone(), project_name.clone(), z);
        map_collection.insert(z, collection);
    }

    let map_editor = MapEditor {
        overmaps: map_collection,
        size: map_size.value(),
    };

    let new_project =
        Project::new(project_name.clone(), ProjectType::MapEditor(map_editor));

    let project_saver = ProjectSaver { path: path.clone() };
    project_saver.save(&new_project).await?;

    program_data_lock.create_and_open_project(new_project.name.clone(), path);

    let mut loaded_projects_lock = loaded_projects.lock().await;
    loaded_projects_lock.insert(new_project.name.clone(), new_project);

    let program_data_saver = ProgramDataSaver {
        path: program_data_lock.config.config_path.clone(),
    };
    program_data_saver.save(&program_data_lock).await?;

    app.emit(
        events::CREATE_TAB,
        Tab {
            name: project_name.clone(),
            tab_type: TabType::MapEditor,
        },
    )?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum ModifyPaletteError {
    #[error(transparent)]
    SaveError(#[from] SaveError),

    #[error(transparent)]
    GetCurrentProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    InvalidProjectType(#[from] InvalidProjectType),

    #[error("Mapgen at coordinates {0} (x,y,z) does not exist")]
    MissingMapgen(IVec3),
}

impl_serialize_for_error!(ModifyPaletteError);

#[derive(Debug, Clone, Deserialize)]
#[serde(
    rename_all = "camelCase",
    tag = "type",
    rename_all_fields = "camelCase"
)]
pub enum ModifyPaletteAction {
    AddPalette { palette_name: CDDAIdentifier },
    RemovePalette { index: usize },
}

#[tauri::command(rename_all = "camelCase")]
pub async fn modify_palette(
    app: AppHandle,
    coordinates: IVec3,
    action: ModifyPaletteAction,
    program_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), ModifyPaletteError> {
    let program_data_lock = program_data.lock().await;
    let mut loaded_projects_lock = loaded_projects.lock().await;

    let loaded_project =
        get_current_project_mut(&program_data_lock, &mut loaded_projects_lock)?;

    let maps = match &mut loaded_project.project_type {
        ProjectType::MapEditor(me) => &mut me.overmaps,
        ProjectType::MapViewer(_) => Err(InvalidProjectType::NotAMapEditor)?,
    };

    let map_data = maps
        .get_mut(&coordinates.z)
        .ok_or(ModifyPaletteError::MissingMapgen(coordinates.clone()))?
        .maps
        .get_mut(&UVec2::new(coordinates.x as u32, coordinates.y as u32).into())
        .ok_or(ModifyPaletteError::MissingMapgen(coordinates.clone()))?;

    match action {
        ModifyPaletteAction::AddPalette { palette_name } => {
            info!("Trying to add palette {} to project", palette_name);
            map_data.palettes.push(MapGenValue::String(palette_name));
        },
        ModifyPaletteAction::RemovePalette { index } => {
            info!("Trying to remove palette at index {} from project", index);
            map_data.palettes.remove(index);
        },
    }

    let path = match program_data_lock
        .openable_projects
        .get(&loaded_project.name)
    {
        None => unreachable!(),
        Some(p) => p,
    };

    let project_saver = ProjectSaver { path: path.clone() };
    project_saver.save(&loaded_project).await?;

    app.emit(events::CURRENT_PROJECT_CHANGED, loaded_project.clone())
        .unwrap();

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(
    rename_all = "camelCase",
    tag = "type",
    rename_all_fields = "camelCase"
)]
pub enum ModifyGlobalPaletteAction {
    AddPalette { palette_name: CDDAIdentifier },
    RemovePalette { palette_name: CDDAIdentifier },
}

#[derive(Error, Debug)]
pub enum ModifyGlobalPaletteError {
    #[error(transparent)]
    SaveError(#[from] SaveError),

    #[error(transparent)]
    GetCurrentProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    InvalidProjectType(#[from] InvalidProjectType),

    #[error("Mapgen at coordinates {0} (x,y,z) does not exist")]
    MissingMapgen(IVec3),
}

impl_serialize_for_error!(ModifyGlobalPaletteError);
#[tauri::command(rename_all = "camelCase")]
pub async fn modify_global_palette(
    app: AppHandle,
    action: ModifyGlobalPaletteAction,
    program_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<(), ModifyGlobalPaletteError> {
    let program_data_lock = program_data.lock().await;
    let mut loaded_projects_lock = loaded_projects.lock().await;

    let loaded_project =
        get_current_project_mut(&program_data_lock, &mut loaded_projects_lock)?;

    let maps = match &mut loaded_project.project_type {
        ProjectType::MapEditor(me) => &mut me.overmaps,
        ProjectType::MapViewer(_) => Err(InvalidProjectType::NotAMapEditor)?,
    };

    match action {
        ModifyGlobalPaletteAction::AddPalette { palette_name } => {
            info!("Trying to add global palette {} to project", palette_name);

            for map_collection in maps.values_mut() {
                for map in map_collection.maps.values_mut() {
                    map.palettes
                        .push(MapGenValue::String(palette_name.clone()));
                }
            }
        },
        ModifyGlobalPaletteAction::RemovePalette { palette_name } => {
            info!(
                "Trying to remove global palette {} from project",
                palette_name
            );

            for map_collection in maps.values_mut() {
                for map in map_collection.maps.values_mut() {
                    map.palettes.retain(|p| {
                        p != &MapGenValue::String(palette_name.clone())
                    });
                }
            }
        },
    }

    let path = match program_data_lock
        .openable_projects
        .get(&loaded_project.name)
    {
        None => unreachable!(),
        Some(p) => p,
    };

    let project_saver = ProjectSaver { path: path.clone() };
    project_saver.save(&loaded_project).await?;

    app.emit(events::CURRENT_PROJECT_CHANGED, loaded_project.clone())
        .unwrap();

    Ok(())
}
#[derive(Error, Debug)]
pub enum GetGlobalPalettesError {
    #[error(transparent)]
    GetCurrentProjectError(#[from] GetCurrentProjectError),

    #[error(transparent)]
    InvalidProjectType(#[from] InvalidProjectType),
}

impl_serialize_for_error!(GetGlobalPalettesError);

#[tauri::command(rename_all = "camelCase")]
pub async fn get_global_palettes(
    program_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
) -> Result<Vec<MapGenValue>, GetGlobalPalettesError> {
    let program_data_lock = program_data.lock().await;
    let mut loaded_projects_lock = loaded_projects.lock().await;

    let loaded_project =
        get_current_project_mut(&program_data_lock, &mut loaded_projects_lock)?;

    let maps = match &mut loaded_project.project_type {
        ProjectType::MapEditor(me) => &mut me.overmaps,
        ProjectType::MapViewer(_) => Err(InvalidProjectType::NotAMapEditor)?,
    };

    let mut removed = Vec::new();

    // We want to get the palettes which are present in all maps
    let palettes = maps
        .values()
        .map(|col| &col.maps)
        .flatten()
        .map(|(_, map)| map)
        .fold(Vec::new(), |mut global, map| {
            for palette in map.palettes.iter() {
                if removed.contains(palette) {
                    continue;
                }

                if global.contains(palette) {
                    continue;
                }

                global.push(palette.clone());
            }

            let mut new_global = global.clone();

            for palette in global.iter() {
                if !map.palettes.contains(palette) {
                    new_global.retain(|p| p != palette);
                    removed.push(palette.clone());
                }
            }

            new_global
        });

    Ok(palettes)
}

#[derive(Error, Debug)]
pub enum GetGlobalPalettesRepresentationError {
    #[error(transparent)]
    GetGlobalPalettesError(#[from] GetGlobalPalettesError),

    #[error(transparent)]
    CDDADataError(#[from] CDDADataError),
}

impl_serialize_for_error!(GetGlobalPalettesRepresentationError);

#[derive(Debug, Serialize)]
pub struct CharacterMapping {
    pub id: TilesheetCDDAId,
    pub ids: ForeBackIds<Option<u32>, Option<u32>>,
}

#[derive(Debug, Serialize, Default)]
pub struct CharacterMappingCollection {
    pub terrain: Option<CharacterMapping>,
    pub furniture: Option<CharacterMapping>,
    pub monster: Option<CharacterMapping>,
    pub field: Option<CharacterMapping>,
}

#[tauri::command(rename_all = "camelCase")]
pub async fn get_global_palette_representations(
    program_data: State<'_, Mutex<ProgramData>>,
    loaded_projects: State<'_, Mutex<LoadedProjects>>,
    json_data: State<'_, Mutex<Option<DeserializedCDDAJsonData>>>,
    tilesheet: State<'_, Mutex<Option<LegacyTilesheet>>>,
    fallback_tilesheet: State<'_, Arc<LegacyTilesheet>>,
) -> Result<
    HashMap<char, CharacterMappingCollection>,
    GetGlobalPalettesRepresentationError,
> {
    let global_palettes =
        get_global_palettes(program_data, loaded_projects).await?;
    let json_data_lock = json_data.lock().await;
    let json_data = get_json_data(&json_data_lock)?;
    let tilesheet_lock = tilesheet.lock().await;

    let mut character_mappings = HashMap::new();

    for global_palette in global_palettes {
        let palette_id = global_palette
            .get_constant_identifier(&IndexMap::new())
            .unwrap();
        let palette = json_data.palettes.get(&palette_id).unwrap();

        for kind in MappingKind::iter() {
            let properties = match palette.properties.get(&kind) {
                None => continue,
                Some(p) => p,
            };

            for (char, property) in properties {
                let repr = match property.representation(&IndexMap::new()) {
                    None => continue,
                    Some(r) => r,
                };

                let mapping = match character_mappings.get_mut(char) {
                    None => {
                        character_mappings.insert(
                            char.clone(),
                            CharacterMappingCollection::default(),
                        );
                        character_mappings.get_mut(&char).unwrap()
                    },
                    Some(c) => c,
                };

                let mapped_cdda_id = MappedCDDAId::simple(repr.id.clone());

                let index = match tilesheet_lock.deref() {
                    None => ForeBackIds::new(
                        Some(
                            fallback_tilesheet
                                .get_fallback(&mapped_cdda_id, json_data),
                        ),
                        None,
                    ),
                    Some(t) => t
                        .get_sprite(&mapped_cdda_id, json_data)
                        .map(|s| {
                            let fg_id = s
                                .get_fg_id(
                                    &mapped_cdda_id,
                                    &repr.tile_layer,
                                    &AdjacentTiles::none(),
                                    json_data,
                                )
                                .map(|i| i.data.into_single().unwrap());

                            let bg_id = s
                                .get_bg_id(
                                    &mapped_cdda_id,
                                    &repr.tile_layer,
                                    &AdjacentTiles::none(),
                                    json_data,
                                )
                                .map(|i| i.data.into_single().unwrap());

                            ForeBackIds::new(fg_id, bg_id)
                        })
                        .unwrap_or(ForeBackIds::new(
                            Some(
                                fallback_tilesheet
                                    .get_fallback(&mapped_cdda_id, json_data),
                            ),
                            None,
                        )),
                };

                match repr.tile_layer {
                    TileLayer::Terrain => {
                        mapping.terrain = Some(CharacterMapping {
                            ids: index,
                            id: repr.id,
                        })
                    },
                    TileLayer::Furniture => {
                        mapping.furniture = Some(CharacterMapping {
                            ids: index,
                            id: repr.id,
                        })
                    },
                    TileLayer::Monster => {
                        mapping.monster = Some(CharacterMapping {
                            ids: index,
                            id: repr.id,
                        })
                    },
                    TileLayer::Field => {
                        mapping.field = Some(CharacterMapping {
                            ids: index,
                            id: repr.id,
                        })
                    },
                }
            }
        }
    }

    Ok(character_mappings)
}
