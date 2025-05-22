use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::region_settings::{CDDARegionSettings, RegionIdentifier};
use crate::cdda_data::terrain::CDDATerrain;
use crate::cdda_data::{MapGenValue, Switch};
use crate::editor_data::{EditorData, MapDataCollection, Project, ZLevel};
use crate::map::DEFAULT_MAP_DATA_SIZE;
use crate::tileset::GetRandom;
use derive_more::with_trait::Display;
use glam::{IVec3, UVec2};
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution as RandDistribution;
use rand::rng;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt;
use std::ops::Deref;
use thiserror::Error;
use tokio::sync::MutexGuard;

#[derive(
    Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Display, Default,
)]
pub struct CDDAIdentifier(pub String);

impl From<&str> for CDDAIdentifier {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl CDDAIdentifier {
    /// This function is used to get the "final" id of the CDDA Identifier. This is used
    /// because a CDDA Identifier might be a region setting id which means that we have to do some other calculations
    /// Additionally there can be liquids which have a 'look_like' property that dictates what they look like.
    /// These are not defined in the tilesheet, but instead in terrain-liquids.json in data/json
    pub fn as_final_id(
        &self,
        region_setting: &CDDARegionSettings,
        terrain: &HashMap<CDDAIdentifier, CDDATerrain>,
        furniture: &HashMap<CDDAIdentifier, CDDAFurniture>,
    ) -> CDDAIdentifier {
        // If it starts with t_region, we know it is a regional setting
        if self.0.starts_with("t_region") {
            if self.0.starts_with("f_") {
                return region_setting
                    .region_terrain_and_furniture
                    .furniture
                    .get(&RegionIdentifier(self.0.clone()))
                    .expect("Furniture Region identifier to exist")
                    .get_random()
                    .as_final_id(region_setting, terrain, furniture);
            } else if self.0.starts_with("t_") {
                return region_setting
                    .region_terrain_and_furniture
                    .terrain
                    .get(&RegionIdentifier(self.0.clone()))
                    .expect("Terrain Region identifier to exist")
                    .get_random()
                    .as_final_id(region_setting, terrain, furniture);
            }
        }

        self.clone()
    }
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Display,
)]
pub struct ParameterIdentifier(pub String);

impl From<&str> for ParameterIdentifier {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

pub type Comment = Option<String>;

#[derive(Debug, Error)]
pub enum GetIdentifierError {
    #[error(transparent)]
    GetRandomError(#[from] GetRandomError),

    #[error("Missing fallback for non existing parameter {0}")]
    MissingFallback(String),

    #[error("Missing value in case {0} for switch {1}")]
    MissingSwitchCaseValue(String, String),
}

pub trait GetIdentifier {
    type Error;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Self::Error>;
}

impl GetIdentifier for DistributionInner {
    type Error = Infallible;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Infallible> {
        match self {
            DistributionInner::Param { param, fallback } => {
                Ok(calculated_parameters
                    .get(param)
                    .map(|p| p.clone())
                    .unwrap_or(fallback.clone()))
            },
            DistributionInner::Normal(n) => Ok(n.clone()),
            _ => todo!(),
        }
    }
}

impl GetIdentifier for CDDAIdentifier {
    type Error = Infallible;

    fn get_identifier(
        &self,
        _calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Infallible> {
        Ok(self.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DistributionInner {
    Param {
        param: ParameterIdentifier,
        fallback: CDDAIdentifier,
    },
    Switch {
        switch: Switch,
        cases: HashMap<CDDAIdentifier, CDDAIdentifier>,
    },
    Normal(CDDAIdentifier),
}

impl Into<MapGenValue> for DistributionInner {
    fn into(self) -> MapGenValue {
        match self {
            DistributionInner::Param { param, fallback } => {
                MapGenValue::Param {
                    param,
                    fallback: Some(fallback),
                }
            },
            DistributionInner::Switch { switch, cases } => {
                MapGenValue::Switch { switch, cases }
            },
            DistributionInner::Normal(n) => MapGenValue::String(n),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MeabyVec<T> {
    Single(T),
    Vec(Vec<T>),
}

impl<T: Clone> MeabyVec<T> {
    pub fn apply<F>(&mut self, fun: F)
    where
        F: Fn(&mut T),
    {
        match self {
            MeabyVec::Single(s) => fun(s),
            MeabyVec::Vec(v) => v.iter_mut().for_each(|v| fun(v)),
        };
    }

    pub fn map<F, R>(self, fun: F) -> Vec<R>
    where
        F: Fn(T) -> R,
    {
        match self {
            MeabyVec::Single(s) => vec![fun(s)],
            MeabyVec::Vec(v) => v.into_iter().map(|v| fun(v)).collect(),
        }
    }

    pub fn for_each<F>(&self, mut fun: F)
    where
        F: FnMut(&T),
    {
        match self {
            MeabyVec::Single(s) => fun(s),
            MeabyVec::Vec(v) => v.iter().for_each(fun),
        }
    }

    pub fn into_vec(self) -> Vec<T> {
        match self {
            MeabyVec::Single(s) => vec![s],
            MeabyVec::Vec(v) => v,
        }
    }

    pub fn into_single(self) -> Option<T> {
        match self {
            MeabyVec::Single(s) => Some(s),
            MeabyVec::Vec(v) => v.first().map(|v| v.clone()),
        }
    }
}

#[derive(Debug, Error, Serialize)]
pub enum WeightedIndexError {
    #[error("Invalid weights for weighted index {0:?}")]
    InvalidWeights(Vec<i32>),
}

#[derive(Debug, Error, Serialize)]
pub enum GetRandomError {
    #[error(transparent)]
    WeightedIndexError(#[from] WeightedIndexError),

    #[error("Failed to get the identifier for the chosen item at index {0}")]
    GetIdentifierError(usize),
}

impl<T: GetIdentifier + Clone> MeabyVec<MeabyWeighted<T>> {
    pub fn get_random(
        &self,
        parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, GetRandomError> {
        let mut weights = vec![];
        let mut self_vec = self.clone().into_vec();

        self.for_each(|v| weights.push(v.weight_or_one()));

        let weighted_index = WeightedIndex::new(weights.clone())
            .map_err(|_| WeightedIndexError::InvalidWeights(weights.clone()))?;

        // let mut rng = RANDOM.write().unwrap();
        let mut rng = rng();

        let chosen_index = weighted_index.sample(&mut rng);
        let item = self_vec.remove(chosen_index);

        item.data()
            .get_identifier(parameters)
            .map_err(|_| GetRandomError::GetIdentifierError(chosen_index))
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize)]
pub struct Weighted<T> {
    pub data: T,
    pub weight: i32,
}

impl<T> Weighted<T> {
    pub fn new(data: impl Into<T>, weight: i32) -> Self {
        Self {
            data: data.into(),
            weight,
        }
    }
}

impl<'de, T> Deserialize<'de> for Weighted<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Create a visitor to help with the deserialization process
        struct WeightedVisitor<T> {
            _marker: std::marker::PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for WeightedVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Weighted<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "[T, i32] where T is the data and i32 is the weight",
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // Extract the elements from the sequence (which should have two items)
                let data: T = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let weight: i32 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                // Create the Weighted struct
                Ok(Weighted { data, weight })
            }
        }

        // Deserialize using the custom visitor
        deserializer.deserialize_seq(WeightedVisitor {
            _marker: std::marker::PhantomData,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    Weighted(Weighted<T>),
    NotWeighted(T),
}

impl<T> From<T> for MeabyWeighted<T> {
    fn from(value: T) -> Self {
        Self::NotWeighted(value)
    }
}

impl<T> MeabyWeighted<T> {
    pub fn data(self) -> T {
        match self {
            MeabyWeighted::NotWeighted(nw) => nw,
            MeabyWeighted::Weighted(w) => w.data,
        }
    }

    pub fn to_weighted(self) -> Weighted<T> {
        match self {
            MeabyWeighted::NotWeighted(d) => Weighted { data: d, weight: 1 },
            MeabyWeighted::Weighted(w) => w,
        }
    }

    pub fn weight_or_one(&self) -> i32 {
        match self {
            MeabyWeighted::Weighted(w) => w.weight,
            MeabyWeighted::NotWeighted(_) => 1,
        }
    }
}

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
