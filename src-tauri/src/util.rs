use crate::data::io::DeserializedCDDAJsonData;
use crate::features::map::DEFAULT_MAP_DATA_SIZE;
use crate::features::program_data::{
    MapDataCollection, ProgramData, Project, ZLevel,
};
use cdda_lib::types::Weighted;
use derive_more::with_trait::Display;
use glam::{IVec3, UVec2};
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution as RandDistribution;
use rand::rng;
use serde::de::Error as SerdeError;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::ops::{Add, Deref, DerefMut};
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
        impl serde::Serialize for $ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
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
    ProjectNotFound(String),
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

pub fn get_current_project(
    editor_data: &ProgramData,
) -> Result<&Project, GetCurrentProjectError> {
    let project_name = match &editor_data.opened_project {
        None => return Err(GetCurrentProjectError::NoProjectOpen),
        Some(i) => i,
    };

    let data = match editor_data.loaded_projects.get(project_name) {
        None => {
            return Err(GetCurrentProjectError::ProjectNotFound(
                project_name.clone(),
            ));
        },
        Some(d) => d,
    };

    Ok(data)
}

pub fn get_current_project_mut<'a>(
    editor_data: &'a mut MutexGuard<ProgramData>,
) -> Result<&'a mut Project, GetCurrentProjectError> {
    let project_name = match editor_data.opened_project.clone() {
        None => return Err(GetCurrentProjectError::NoProjectOpen),
        Some(i) => i,
    };

    let data = match editor_data.loaded_projects.get_mut(&project_name) {
        None => {
            return Err(GetCurrentProjectError::ProjectNotFound(
                project_name.clone(),
            ));
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

pub fn get_json_data_mut<'a>(
    lock: &'a mut MutexGuard<Option<DeserializedCDDAJsonData>>,
) -> Result<&'a mut DeserializedCDDAJsonData, CDDADataError> {
    match lock.deref_mut() {
        None => Err(CDDADataError::NotLoaded),
        Some(d) => Ok(d),
    }
}

pub trait GetRandom<T> {
    fn get_random(&self) -> &T;
}

impl<T> GetRandom<T> for Vec<Weighted<T>> {
    fn get_random(&self) -> &T {
        let mut weights = vec![];
        self.iter().for_each(|v| weights.push(v.weight));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");

        let mut rng = rng();
        //let mut rng = RANDOM.write().unwrap();

        let chosen_index = weighted_index.sample(&mut rng);

        &self.get(chosen_index).unwrap().data
    }
}

impl<T> GetRandom<T> for IndexMap<T, i32> {
    fn get_random(&self) -> &T {
        let mut weights = vec![];

        let mut vec = self.iter().collect::<Vec<(&T, &i32)>>();
        vec.iter().for_each(|(_, w)| weights.push(**w));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");

        let mut rng = rng();
        //let mut rng = RANDOM.write().unwrap();

        let chosen_index = weighted_index.sample(&mut rng);
        let item = vec.remove(chosen_index);

        &item.0
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum Rotation {
    #[default]
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Serialize for Rotation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clone().deg().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rotation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let deg = u32::deserialize(deserializer)? % 360;

        match deg {
            0 => Ok(Rotation::Deg0),
            90 => Ok(Rotation::Deg90),
            180 => Ok(Rotation::Deg180),
            270 => Ok(Rotation::Deg270),
            _ => Err(SerdeError::custom(format!(
                "Invalid rotation value {}",
                deg
            ))),
        }
    }
}

impl Add<Rotation> for Rotation {
    type Output = Rotation;

    fn add(self, rhs: Rotation) -> Self::Output {
        let value = self.deg() + rhs.deg();
        Self::from(value)
    }
}

impl From<i32> for Rotation {
    fn from(value: i32) -> Self {
        let value = value % 360;

        match value {
            0..90 => Self::Deg0,
            90..180 => Self::Deg90,
            180..270 => Self::Deg180,
            270..360 => Self::Deg270,
            _ => unreachable!(),
        }
    }
}

impl Rotation {
    pub fn deg(&self) -> i32 {
        match self {
            Rotation::Deg0 => 0,
            Rotation::Deg90 => 90,
            Rotation::Deg180 => 180,
            Rotation::Deg270 => 270,
        }
    }
}

impl From<CardinalDirection> for Rotation {
    fn from(value: CardinalDirection) -> Self {
        match value {
            CardinalDirection::North => Self::Deg0,
            CardinalDirection::East => Self::Deg90,
            CardinalDirection::South => Self::Deg180,
            CardinalDirection::West => Self::Deg270,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum CardinalDirection {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}
