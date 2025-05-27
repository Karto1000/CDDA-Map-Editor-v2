use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::editor_data::{EditorData, MapDataCollection, Project, ZLevel};
use crate::map::DEFAULT_MAP_DATA_SIZE;
use derive_more::with_trait::Display;
use glam::{IVec3, UVec2};
use rand::prelude::Distribution as RandDistribution;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::ops::Deref;
use thiserror::Error;
use tokio::sync::MutexGuard;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct UVec2JsonKey(pub UVec2);

impl Serialize for UVec2JsonKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the UVec2 into a string like "x,y"
        let s = format!("{},{}", self.0.x, self.0.y);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for UVec2JsonKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize the string in the format "x,y"
        let s = String::deserialize(deserializer)?;

        // Split the string by comma to extract x and y values
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &"a string in the format 'x,y'",
            ));
        }

        // Parse the x and y values as u32
        let x = parts[0].parse::<u32>().map_err(serde::de::Error::custom)?;
        let y = parts[1].parse::<u32>().map_err(serde::de::Error::custom)?;

        // Return the JSONSerializableUVec2 wrapper with the parsed UVec2
        Ok(UVec2JsonKey(UVec2::new(x, y)))
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct IVec3JsonKey(pub IVec3);

impl Serialize for IVec3JsonKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the UVec2 into a string like "x,y"
        let s = format!("{},{},{}", self.0.x, self.0.y, self.0.z);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for IVec3JsonKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(&s),
                &"a string in the format 'x,y,z'",
            ));
        }

        let x = parts[0].parse::<i32>().map_err(serde::de::Error::custom)?;
        let y = parts[1].parse::<i32>().map_err(serde::de::Error::custom)?;
        let z = parts[2].parse::<i32>().map_err(serde::de::Error::custom)?;

        Ok(IVec3JsonKey(IVec3::new(x, y, z)))
    }
}

#[derive(Debug, Display, Error)]
pub enum SaveError {
    IoError(#[from] std::io::Error),
    JsonError(#[from] serde_json::Error),
}

pub trait Save<T> {
    async fn save(&self, data: &T) -> Result<(), SaveError>;
}

pub trait Load<T, E = anyhow::Error> {
    async fn load(&mut self) -> Result<T, E>;
}

pub fn bresenham_line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut points = Vec::new();

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let (mut x, mut y) = (x0, y0);

    loop {
        points.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }

    points
}

#[macro_export]
macro_rules! impl_merge_with_precedence {
    // First parameter: Struct name, second: normal fields, third: Option fields
    ($struct_name:ident, $( $field:ident ),*; $( $opt_field:ident ),*) => {
        impl $struct_name {
            pub fn merge_with_precedence(base: &Self, override_: &Self) -> Self {
                Self {
                    // Handle non-Option fields: Just copy the value from override_
                    $(
                        $field: override_.$field.clone(),
                    )*

                    // Handle Option<T> fields: Use the value from override_ if Some, else keep base value
                    $(
                        $opt_field: override_.$opt_field.clone().or_else(|| base.$opt_field.clone()),
                    )*
                }
            }
        }
    };
}

// https://stackoverflow.com/a/49806368
#[macro_export]
macro_rules! skip_err {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                warn!("Error for value: {:?}, Err: {:?}; Skipping", $res, e);
                continue;
            },
        }
    };
}

#[macro_export]
macro_rules! skip_none {
    ($res:expr) => {
        match $res {
            Some(val) => val,
            None => {
                warn!("Missing value for {:?}; Skipping", $res);
                continue;
            },
        }
    };
}

#[macro_export]
macro_rules! impl_serialize_for_error {
    (
        $ident: ident
    ) => {
        impl Serialize for $ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }
    };
}

#[derive(Debug, Error, Serialize)]
pub enum GetCurrentProjectError {
    #[error("No project has been opened")]
    NoProjectOpen,
    #[error("Invalid project name {0}")]
    InvalidProjectName(String),
}

#[derive(Debug, Error, Serialize)]
pub enum CDDADataError {
    #[error("No CDDA Data was loaded")]
    NotLoaded,
}

pub fn get_size(maps: &HashMap<ZLevel, MapDataCollection>) -> UVec2 {
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    // Find the maximum x and y coordinates across all maps
    for map_data in maps.values() {
        for pos in map_data.maps.keys() {
            max_x = max_x.max(pos.x);
            max_y = max_y.max(pos.y);
        }
    }

    // Add 1 since coordinates are 0-based
    UVec2::new(
        (max_x + 1) * DEFAULT_MAP_DATA_SIZE.x,
        (max_y + 1) * DEFAULT_MAP_DATA_SIZE.y,
    )
}

pub fn get_current_project<'a>(
    editor_data: &'a MutexGuard<EditorData>,
) -> Result<&'a Project, GetCurrentProjectError> {
    let project_name = match &editor_data.opened_project {
        None => return Err(GetCurrentProjectError::NoProjectOpen),
        Some(i) => i,
    };

    let data = match editor_data
        .projects
        .iter()
        .find(|p| *p.name == *project_name)
    {
        None => {
            return Err(GetCurrentProjectError::InvalidProjectName(
                project_name.clone(),
            ))
        },
        Some(d) => d,
    };

    Ok(data)
}

pub fn get_current_project_mut<'a>(
    editor_data: &'a mut MutexGuard<EditorData>,
) -> Result<&'a mut Project, GetCurrentProjectError> {
    let project_name = match editor_data.opened_project.clone() {
        None => return Err(GetCurrentProjectError::NoProjectOpen),
        Some(i) => i,
    };

    let data = match editor_data
        .projects
        .iter_mut()
        .find(|p| p.name == *project_name)
    {
        None => {
            return Err(GetCurrentProjectError::InvalidProjectName(
                project_name.clone(),
            ))
        },
        Some(d) => d,
    };

    Ok(data)
}

pub fn get_json_data<'a>(
    lock: &'a MutexGuard<Option<DeserializedCDDAJsonData>>,
) -> Result<&'a DeserializedCDDAJsonData, CDDADataError> {
    match lock.deref() {
        None => Err(CDDADataError::NotLoaded),
        Some(d) => Ok(d),
    }
}
