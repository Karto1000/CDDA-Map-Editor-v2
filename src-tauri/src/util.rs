use glam::UVec2;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weighted<T> {
    pub sprite: T,
    pub weight: i32,
}

impl<T> Weighted<T> {
    pub fn new(sprite: T, weight: i32) -> Self {
        Self { sprite, weight }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MeabyVec<T> {
    Single(T),
    Vec(Vec<T>),
}

impl<T> MeabyVec<T> {
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
            MeabyVec::Vec(v) => v.iter().for_each(fun)
        }
    }

    pub fn vec(self) -> Vec<T> {
        match self {
            MeabyVec::Single(s) => vec![s],
            MeabyVec::Vec(v) => v
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MeabyWeighted<T> {
    NotWeighted(T),
    Weighted(Weighted<T>),
}

impl<T> MeabyWeighted<T> {
    pub fn map<F, R>(self, fun: F) -> R
    where
        F: Fn(T) -> R,
    {
        match self {
            MeabyWeighted::NotWeighted(nw) => fun(nw),
            MeabyWeighted::Weighted(w) => fun(w.sprite)
        }
    }

    pub fn data(self) -> T {
        match self {
            MeabyWeighted::NotWeighted(nw) => nw,
            MeabyWeighted::Weighted(w) => w.sprite
        }
    }
}

impl<T> MeabyWeighted<T> {
    pub fn weighted(self) -> Weighted<T> {
        match self {
            MeabyWeighted::NotWeighted(d) => Weighted {
                sprite: d,
                weight: 0,
            },
            MeabyWeighted::Weighted(w) => w,
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
    fn load(&self) -> Result<T, std::io::Error>;
}