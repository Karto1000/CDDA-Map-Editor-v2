use glam::UVec2;
use rand::distr::weighted::WeightedIndex;
use rand::prelude::Distribution as RandDistribution;
use rand::rng;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

pub type CDDAIdentifier = String;
pub type ParameterIdentifier = String;
pub type Comment = Option<String>;

pub trait GetIdentifier {
    fn get_identifier(
        &self,
        calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier;
}

impl GetIdentifier for MeabyParam {
    fn get_identifier(
        &self,
        calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        match self {
            MeabyParam::Param { param, fallback } => calculated_parameters
                .get(param)
                .map(|p| p.clone())
                .unwrap_or(fallback.clone()),
            MeabyParam::Normal(n) => n.clone(),
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
pub struct Switch {
    param: ParameterIdentifier,
    fallback: CDDAIdentifier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub distribution: MeabyVec<MeabyWeighted<CDDAIdentifier>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MeabyParam {
    Param {
        param: ParameterIdentifier,
        fallback: CDDAIdentifier,
    },
    Normal(CDDAIdentifier),
}

// https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/JSON/MAPGEN.md#mapgen-values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapGenValue {
    String(CDDAIdentifier),
    Distribution(MeabyVec<MeabyWeighted<MeabyParam>>),
    Param {
        param: ParameterIdentifier,
        fallback: Option<CDDAIdentifier>,
    },
    Switch {
        switch: Switch,
        cases: HashMap<ParameterIdentifier, CDDAIdentifier>,
    },
}

impl GetIdentifier for MapGenValue {
    fn get_identifier(
        &self,
        calculated_parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        match self {
            MapGenValue::String(s) => s.clone(),
            MapGenValue::Distribution(d) => d.get(calculated_parameters),
            MapGenValue::Param { param, fallback } => calculated_parameters
                .get(param)
                .map(|p| p.clone())
                .unwrap_or_else(|| fallback.clone().expect("Fallback to exist")),
            MapGenValue::Switch { switch, cases } => {
                let id = calculated_parameters
                    .get(&switch.param)
                    .map(|p| p.clone())
                    .unwrap_or_else(|| switch.fallback.clone());

                cases.get(&id).expect("MapTo to exist").clone()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CataVariant {
    ItemGroupId,
    ItypeId,
    MatypeId,
    MtypeId,
    MongroupId,
    MutagenTechnique,
    MutationCategoryId,
    NestedMapgenId,
    NpcTemplateId,
    OterId,
    OterTypeStrId,
    OvermapSpecialId,
    PaletteId,
    Point,
    Tripoint,
    ProfessionId,
    ProficiencyId,
    SkillId,
    SpeciesId,
    SpellId,
    TerId,
    TerFurnTransformId,
    TerStrId,
    TraitId,
    TrapId,
    TrapStrId,
    VgroupId,
    WidgetId,
    ZoneTypeId,
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
            MeabyVec::Vec(v) => v.iter().for_each(fun),
        }
    }

    pub fn vec(self) -> Vec<T> {
        match self {
            MeabyVec::Single(s) => vec![s],
            MeabyVec::Vec(v) => v,
        }
    }
}

impl<T: GetIdentifier + Clone> MeabyVec<MeabyWeighted<T>> {
    pub fn get(&self, parameters: &HashMap<ParameterIdentifier, CDDAIdentifier>) -> CDDAIdentifier {
        let mut weights = vec![];
        let mut self_vec = self.clone().vec();

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
