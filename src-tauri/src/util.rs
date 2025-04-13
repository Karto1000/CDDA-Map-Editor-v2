use crate::cdda_data::furniture::CDDAFurniture;
use crate::cdda_data::region_settings::{CDDARegionSettings, RegionIdentifier};
use crate::cdda_data::terrain::CDDATerrain;
use crate::cdda_data::Switch;
use derive_more::with_trait::Display;
use glam::UVec2;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution as RandDistribution;
use rand::rng;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

pub trait GetRandom<T> {
    fn get_random(&self) -> &T;
}

impl<T> GetRandom<T> for HashMap<T, i32> {
    fn get_random(&self) -> &T {
        let mut weights = vec![];

        let mut vec = self.iter().collect::<Vec<(&T, &i32)>>();
        vec.iter().for_each(|(_, w)| weights.push(**w));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");
        let mut rng = rng();

        let chosen_index = weighted_index.sample(&mut rng);
        let item = vec.remove(chosen_index);

        &item.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Display)]
pub struct CDDAIdentifier(pub String);

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

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Display)]
pub struct ParameterIdentifier(pub String);
pub type Comment = Option<String>;

pub trait GetIdentifier {
    fn get_identifier(
        &self,
        calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier;
}

impl GetIdentifier for DistributionInner {
    fn get_identifier(
        &self,
        calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        match self {
            DistributionInner::Param { param, fallback } => calculated_parameters
                .get(&param)
                .map(|p| p.clone())
                .unwrap_or(fallback.clone()),
            DistributionInner::Normal(n) => n.clone(),
            DistributionInner::Switch { switch, cases } => {
                panic!()
            }
        }
    }
}

impl GetIdentifier for CDDAIdentifier {
    fn get_identifier(
        &self,
        _calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        self.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
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

impl<T: GetIdentifier + Clone> MeabyVec<MeabyWeighted<T>> {
    pub fn get(&self, parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>) -> CDDAIdentifier {
        let mut weights = vec![];
        let mut self_vec = self.clone().into_vec();

        self.for_each(|v| weights.push(v.weight_or_one()));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");
        let mut rng = rng();

        let chosen_index = weighted_index.sample(&mut rng);
        let item = self_vec.remove(chosen_index);

        item.data().get_identifier(parameters)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Weighted<T> {
    data: T,
    weight: i32,
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
                formatter.write_str("[T, i32] where T is the data and i32 is the weight")
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    Weighted(Weighted<T>),
    NotWeighted(T),
}

impl<T> MeabyWeighted<T> {
    pub fn data(self) -> T {
        match self {
            MeabyWeighted::NotWeighted(nw) => nw,
            MeabyWeighted::Weighted(w) => w.data,
        }
    }

    pub fn weighted(self) -> Weighted<T> {
        match self {
            MeabyWeighted::NotWeighted(d) => Weighted { data: d, weight: 0 },
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
pub struct JSONSerializableUVec2(pub UVec2);

impl Serialize for JSONSerializableUVec2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert the UVec2 into a string like "x,y"
        let s = format!("{},{}", self.0.x, self.0.y);
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for JSONSerializableUVec2 {
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
        Ok(JSONSerializableUVec2(UVec2::new(x, y)))
    }
}

pub trait Save<T> {
    fn save(&self, data: &T) -> Result<(), std::io::Error>;
}

pub trait Load<T> {
    fn load(&self) -> Result<T, anyhow::Error>;
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
