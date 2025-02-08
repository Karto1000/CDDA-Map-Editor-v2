use glam::UVec2;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
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
