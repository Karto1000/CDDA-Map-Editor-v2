use serde::de::Error;
use serde::{Deserialize, Deserializer};
#[derive(Debug)]
pub struct ZLevels((i32, i32));

impl<'de> Deserialize<'de> for ZLevels {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let zlevels = <(i32, i32)>::deserialize(deserializer)?;

        if zlevels.0 < zlevels.1 {
            return Err(Error::custom(
                "ZLevel start must be less than ZLevel end",
            ));
        }

        Ok(ZLevels(zlevels))
    }
}

impl ZLevels {
    pub fn value(&self) -> (i32, i32) {
        self.0
    }
}
