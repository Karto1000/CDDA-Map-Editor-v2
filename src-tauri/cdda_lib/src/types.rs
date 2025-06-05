use derive_more::Display;
use num_traits::int::PrimInt;
use rand::distr::uniform::SampleUniform;
use rand::{rng, Rng};
use serde::de;
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::ops::{Add, Deref, Rem, Sub};

#[derive(Deserialize)]
#[serde(untagged)]
pub enum NumberOrArray<T: PrimInt + Clone + SampleUniform> {
    Number(T),
    Array(Vec<T>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAExtendOp {
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum IdOrAbstract<T> {
    #[serde(rename = "id")]
    Id(T),
    #[serde(rename = "abstract")]
    Abstract(CDDAIdentifier),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDADeleteOp {
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CDDAString {
    String(String),
    StringMap { str: String },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Switch {
    pub param: ParameterIdentifier,
    pub fallback: CDDAIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub distribution: MeabyVec<MeabyWeighted<CDDAIdentifier>>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Display, Default,
)]
pub struct CDDAIdentifier(pub String);

impl Deref for CDDAIdentifier {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for CDDAIdentifier {
    fn from(value: &str) -> Self {
        Self(value.to_string())
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
        // Helper visitor to handle both array and object formats
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
                    "expected [T, i32] or { \"weight\": i32, \"sprite\": T }",
                )
            }

            // Handle the sequence format: [data, weight]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let data: T = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let weight: i32 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(Weighted { data, weight })
            }

            // Handle the map format: { "weight": ..., "sprite": ... }
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut data: Option<T> = None;
                let mut weight: Option<i32> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "sprite" => {
                            data = Some(map.next_value()?);
                        },
                        "weight" => {
                            weight = Some(map.next_value()?);
                        },
                        _ => {
                            return Err(serde::de::Error::unknown_field(
                                &key,
                                &["sprite", "weight"],
                            ))
                        },
                    }
                }

                let data = data
                    .ok_or_else(|| serde::de::Error::missing_field("sprite"))?;
                let weight = weight
                    .ok_or_else(|| serde::de::Error::missing_field("weight"))?;

                Ok(Weighted { data, weight })
            }
        }

        deserializer.deserialize_any(WeightedVisitor {
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

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum NumberOrRange<T: PrimInt + Clone + SampleUniform> {
    Number(T),
    Range((T, T)),
}

impl<T: PrimInt + SampleUniform> Add<T> for NumberOrRange<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        match self {
            NumberOrRange::Number(n) => NumberOrRange::Number(n + rhs),
            NumberOrRange::Range(r) => {
                NumberOrRange::Range((r.0 + rhs, r.1 + rhs))
            },
        }
    }
}

impl<T: PrimInt + SampleUniform> Sub<T> for NumberOrRange<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        match self {
            NumberOrRange::Number(n) => NumberOrRange::Number(n - rhs),
            NumberOrRange::Range(r) => {
                NumberOrRange::Range((r.0 - rhs, r.1 - rhs))
            },
        }
    }
}

impl<T: PrimInt + SampleUniform> Rem<T> for NumberOrRange<T> {
    type Output = Self;

    fn rem(self, rhs: T) -> Self::Output {
        match self {
            NumberOrRange::Number(n) => NumberOrRange::Number(n % rhs),
            NumberOrRange::Range(r) => {
                NumberOrRange::Range((r.0 % rhs, r.1 % rhs))
            },
        }
    }
}

impl<T: PrimInt + SampleUniform> PartialEq<T> for NumberOrRange<T> {
    fn eq(&self, other: &T) -> bool {
        match self {
            NumberOrRange::Number(n) => n == other,
            NumberOrRange::Range((min, max)) => other >= min && other <= max,
        }
    }
}

impl<T: PrimInt + SampleUniform> PartialOrd<T> for NumberOrRange<T> {
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        match self {
            NumberOrRange::Number(n) => n.partial_cmp(other),
            NumberOrRange::Range((min, max)) => {
                if other < min {
                    Some(Ordering::Greater)
                } else if other > max {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Equal)
                }
            },
        }
    }
}

impl<'de, T: PrimInt + Clone + SampleUniform + Deserialize<'de>>
    Deserialize<'de> for NumberOrRange<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = NumberOrArray::<T>::deserialize(deserializer)?;
        match value {
            NumberOrArray::Number(n) => Ok(NumberOrRange::Number(n)),
            NumberOrArray::Array(arr) => match arr.len() {
                1 => Ok(NumberOrRange::Number(arr[0].clone())),
                2 => Ok(NumberOrRange::Range((arr[0].clone(), arr[1].clone()))),
                _ => Err(serde::de::Error::custom(
                    "Array must contain 1 or 2 elements",
                )),
            },
        }
    }
}

impl<T: PrimInt + Clone + SampleUniform> NumberOrRange<T> {
    pub fn rand_number(&self) -> T {
        match self.clone() {
            NumberOrRange::Number(n) => n,
            NumberOrRange::Range((from, to)) => {
                let mut rng = rng();
                //let mut rng = RANDOM.write().unwrap();
                let num = rng.random_range(from..to);
                num
            },
        }
    }

    pub fn is_random_hit(&self, default_upper_bound: T) -> bool {
        match self.clone() {
            NumberOrRange::Number(n) => {
                // This will always be true
                if n >= default_upper_bound {
                    return true;
                }

                let mut rng = rng();
                //let mut rng = RANDOM.write().unwrap();
                let num = rng.random_range(n..default_upper_bound);

                num == n
            },
            NumberOrRange::Range((from, to)) => {
                let mut rng = rng();
                //let mut rng = RANDOM.write().unwrap();
                let num = rng.random_range(from..to);

                num == from
            },
        }
    }

    pub fn get_from_to(&self) -> (T, T) {
        match self.clone() {
            NumberOrRange::Number(n) => (n, n),
            NumberOrRange::Range((from, to)) => (from, to),
        }
    }
}

// TODO: Kind of a hacky solution to a Stack Overflow problem that i experienced when using
// a self-referencing MapGenValue enum
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CDDADistributionInner {
    String(CDDAIdentifier),
    Param {
        param: ParameterIdentifier,
        fallback: Option<CDDAIdentifier>,
    },
    Switch {
        switch: Switch,
        cases: HashMap<CDDAIdentifier, CDDAIdentifier>,
    },
    // The distribution inside another distribution must have the 'distribution'
    // property. This is why we're using the Distribution struct here
    //                 --- Here ---
    // "palettes": [ { "distribution": [ [ "cabin_palette", 1 ], [ "cabin_palette_abandoned", 1 ] ] } ],
    Distribution(Distribution),
}

impl From<&str> for CDDADistributionInner {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

// https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/JSON/MAPGEN.md#mapgen-values
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapGenValue {
    String(CDDAIdentifier),
    Param {
        param: ParameterIdentifier,
        fallback: Option<CDDAIdentifier>,
    },
    Switch {
        switch: Switch,
        cases: HashMap<CDDAIdentifier, CDDAIdentifier>,
    },
    // TODO: We could probably use a MapGenValue instead of this DistributionInner type, but that would
    // require a Box<> since we don't know the size. I tried this but for some reason it causes a Stack Overflow
    // because serde keeps infinitely calling the Deserialize function even though it should deserialize to the String variant.
    // I'm not sure if this is a bug with my logic or if this is some sort of oversight in serde.
    Distribution(MeabyVec<MeabyWeighted<CDDADistributionInner>>),
}

pub trait ImportCDDAObject: Clone + Debug {
    fn merge(base: &Self, override_: &Self) -> Self;

    fn copy_from(&self) -> Option<&CDDAIdentifier>;

    fn extend(&self) -> Option<&CDDAExtendOp>;
    fn delete(&self) -> Option<&CDDADeleteOp>;

    fn flags(&self) -> &Vec<String>;
    fn set_flags(&mut self, flags: Vec<String>);

    fn calculate_copy(
        &self,
        all_intermediate_objects: &HashMap<CDDAIdentifier, Self>,
    ) -> Self {
        match self.copy_from() {
            None => self.clone(),
            Some(copy_from_id) => {
                let mut copy_from_special =
                    match all_intermediate_objects.get(copy_from_id) {
                        None => {
                            log::warn!(
                                "Could not copy {:?} due to it not existing",
                                copy_from_id,
                            );
                            return self.clone();
                        },
                        Some(t) => t.clone(),
                    };

                if copy_from_special.copy_from().is_some() {
                    copy_from_special = copy_from_special
                        .calculate_copy(all_intermediate_objects);
                }

                match &self.extend() {
                    None => {},
                    Some(extend) => match &extend.flags {
                        None => {},
                        Some(new_flags) => {
                            let mut old_flags =
                                copy_from_special.flags().clone();
                            old_flags.extend(new_flags.clone());
                            copy_from_special.set_flags(old_flags);
                        },
                    },
                }

                match &self.delete() {
                    None => {},
                    Some(extend) => match &extend.flags {
                        None => {},
                        Some(new_flags) => {
                            let mut old_flags =
                                copy_from_special.flags().clone();

                            old_flags = old_flags
                                .into_iter()
                                .filter(|f| {
                                    new_flags
                                        .iter()
                                        .find(|nf| *nf == f)
                                        .is_none()
                                })
                                .collect();

                            copy_from_special.set_flags(old_flags);
                        },
                    },
                }

                Self::merge(&copy_from_special, self)
            },
        }
    }
}
