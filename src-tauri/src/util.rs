use crate::data::io::DeserializedCDDAJsonData;
use crate::features::map::DEFAULT_MAP_DATA_SIZE;
use crate::features::program_data::{
    LoadedProjects, Overmap, ProgramData, Project, ZLevel,
};
use cdda_lib::types::Weighted;
use derive_more::with_trait::Display;
use glam::{IVec2, IVec3, UVec2, UVec3};
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution as RandDistribution;
use rand::rng;
use serde::de::{Error as SerdeError, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Add, Deref, DerefMut};
use thiserror::Error;
use tokio::sync::MutexGuard;

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

pub fn get_size(maps: &HashMap<ZLevel, Overmap>) -> UVec2 {
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
    editor_data: &ProgramData,
    loaded_projects: &'a LoadedProjects,
) -> Result<&'a Project, GetCurrentProjectError> {
    let project_name = match &editor_data.opened_project {
        None => return Err(GetCurrentProjectError::NoProjectOpen),
        Some(i) => i,
    };

    let data = match loaded_projects.get(project_name) {
        None => {
            return Err(GetCurrentProjectError::ProjectNotFound(
                project_name.clone(),
            ));
        }
        Some(d) => d,
    };

    Ok(data)
}

pub fn get_current_project_mut<'a>(
    editor_data: &MutexGuard<ProgramData>,
    loaded_projects: &'a mut LoadedProjects,
) -> Result<&'a mut Project, GetCurrentProjectError> {
    let project_name = match editor_data.opened_project.clone() {
        None => return Err(GetCurrentProjectError::NoProjectOpen),
        Some(i) => i,
    };

    let data = match loaded_projects.get_mut(&project_name) {
        None => {
            return Err(GetCurrentProjectError::ProjectNotFound(
                project_name.clone(),
            ));
        }
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

macro_rules! impl_map_with_vec_keys_serializer_and_deserializer {
    (
        $vec_type: ty => [$($map_type: ty),*],
        $parse_key: expr,
        $error: literal
    ) => {
        $(
            paste::paste! {
                pub fn [<serialize_ $map_type:lower _with_ $vec_type:lower _keys>]<S, T>(
                    generic_tags: &$map_type<$vec_type, T>,
                    serializer: S,
                ) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                    T: serde::Serialize,
                {
                    let mut map = serializer.serialize_map(Some(generic_tags.len()))?;
                    for (uvec, values) in generic_tags {
                        map.serialize_entry(&format!("{},{}", uvec.x, uvec.y), values)?;
                    }
                    map.end()
                }

                struct [<$vec_type $map_type KeysVisitor>]<T>(PhantomData<T>);

                impl<'de, T: Deserialize<'de>> Visitor<'de> for [<$vec_type $map_type KeysVisitor>]<T> {
                    type Value = $map_type<$vec_type, T>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter
                            .write_str($error)
                    }

                    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                    where
                        M: MapAccess<'de>,
                    {
                        let mut values = $map_type::new();

                        while let Some(key) = map.next_key::<String>()? {
                            // Parse the coordinate string "x,y"
                            let transformed_vec = $parse_key(key)?;
                            let value = map.next_value()?;
                            values.insert(transformed_vec, value);
                        }

                        Ok(values)
                    }
                }

                pub fn [<deserialize_ $map_type:lower _with_ $vec_type:lower _keys>]<'de, D, T>(
                    deserializer: D,
                ) -> Result<$map_type<$vec_type, T>, D::Error>
                where
                    D: Deserializer<'de>,
                    T: Deserialize<'de>,
                {
                    deserializer.deserialize_map([<$vec_type $map_type KeysVisitor>](PhantomData))
                }
            }
        )*
    };
}

impl_map_with_vec_keys_serializer_and_deserializer!(
    UVec2 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 2 {
            return Err(M::Error::custom("map in which the keys are formatted as 'x,y' and where all the keys numbers are greater or equal to 0"));
        }

        let x = coords[0].parse::<u32>().map_err(M::Error::custom)?;
        let y = coords[1].parse::<u32>().map_err(M::Error::custom)?;

        Ok(UVec2::new(x, y))
    },
    "map in which the keys are formatted as 'x,y' and where all the keys numbers are greater or equal to 0"
);

impl_map_with_vec_keys_serializer_and_deserializer!(
    UVec3 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 3 {
            return Err(M::Error::custom("map in which the keys are formatted as 'x,y,z' and where all the keys numbers are greater or equal to 0"));
        }

        let x = coords[0].parse::<u32>().map_err(M::Error::custom)?;
        let y = coords[1].parse::<u32>().map_err(M::Error::custom)?;
        let z = coords[2].parse::<u32>().map_err(M::Error::custom)?;

        Ok(UVec3::new(x, y, z))
    },
    "map in which the keys are formatted as 'x,y,z' and where all the keys numbers are greater or equal to 0"
);

impl_map_with_vec_keys_serializer_and_deserializer!(
    IVec2 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 2 {
            return Err(M::Error::custom("map in which the keys are formatted as 'x,y'"));
        }

        let x = coords[0].parse::<i32>().map_err(M::Error::custom)?;
        let y = coords[1].parse::<i32>().map_err(M::Error::custom)?;

        Ok(IVec2::new(x, y))
    },
    "map in which the keys are formatted as 'x,y'"
);

impl_map_with_vec_keys_serializer_and_deserializer!(
    IVec3 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 3 {
            return Err(M::Error::custom("map in which the keys are formatted as 'x,y,z'"));
        }

        let x = coords[0].parse::<i32>().map_err(M::Error::custom)?;
        let y = coords[1].parse::<i32>().map_err(M::Error::custom)?;
        let z = coords[2].parse::<i32>().map_err(M::Error::custom)?;

        Ok(IVec3::new(x, y, z))
    },
    "map in which the keys are formatted as 'x,y,z'"
);