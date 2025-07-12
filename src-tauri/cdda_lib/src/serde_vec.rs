use glam::{IVec2, IVec3, UVec2, UVec3};
use indexmap::IndexMap;
use serde::de::Error;
use serde::ser::SerializeMap;
use serde::Deserializer;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

macro_rules! impl_map_with_vec_keys_serializer_and_deserializer {
    (
        $vec_type: ty => [$($map_type: ty),*],
        $deserialize_key: expr,
        $serialize_key: expr,
        $error: literal
    ) => {
        paste::paste! {
             pub struct [<$vec_type KeysVisitor>]();

             impl<'de> serde::de::Visitor<'de> for [<$vec_type KeysVisitor>] {
                 type Value = $vec_type;

                 fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                     formatter
                         .write_str($error)
                 }

                 fn visit_str<E>(self, value: &str) -> Result<$vec_type, E>
                     where
                         E: serde::de::Error,
                 {
                     $deserialize_key(value.to_string()).map_err(E::custom)
                 }
             }

             pub fn [<serialize_ $vec_type:lower _as_key>]<S>(
                [<$vec_type:lower>]: &[<$vec_type>],
                serializer: S,
             ) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
            {
                let serialized_key = ($serialize_key)(&[<$vec_type:lower>]);
                serializer.serialize_str(serialized_key.as_str())
            }

            pub fn [<deserialize_ $vec_type:lower _as_key>]<'de, D>(
                deserializer: D,
            ) -> Result<$vec_type, D::Error>
                where
                    D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str([<$vec_type KeysVisitor>]())
            }
        }

        $(
            paste::paste! {
                pub fn [<serialize_ $map_type:lower _with_ $vec_type:lower _keys>]<S, T>(
                    map_to_serialize: &$map_type<$vec_type, T>,
                    serializer: S,
                ) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                    T: serde::Serialize,
                {
                    let mut map = serializer.serialize_map(Some(map_to_serialize.len()))?;
                    for (vec, values) in map_to_serialize {
                        map.serialize_entry(&$serialize_key(&vec), values)?;
                    }
                    map.end()
                }

                struct [<$vec_type $map_type KeysVisitor>]<T>(PhantomData<T>);

                impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for [<$vec_type $map_type KeysVisitor>]<T> {
                    type Value = $map_type<$vec_type, T>;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter
                            .write_str($error)
                    }

                    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                    where
                        M: serde::de::MapAccess<'de>,
                    {
                        let mut values = $map_type::new();

                        while let Some(key) = map.next_key::<String>()? {
                            // Parse the coordinate string "x,y"
                            let transformed_vec = $deserialize_key(key).map_err(M::Error::custom)?;
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
                    D: serde::Deserializer<'de>,
                    T: serde::Deserialize<'de>,
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
            return Err("map in which the keys are formatted as 'x,y' and where all the keys numbers are greater or equal to 0");
        }

        let x = coords[0].parse::<u32>().map_err(|_| "Could not parse x coordinate")?;
        let y = coords[1].parse::<u32>().map_err(|_| "Could not parse y coordinate")?;

        Ok(UVec2::new(x, y))
    },
    |uvec2: &UVec2| format!("{},{}", uvec2.x, uvec2.y),
    "map in which the keys are formatted as 'x,y' and where all the keys numbers are greater or equal to 0"
);

impl_map_with_vec_keys_serializer_and_deserializer!(
    UVec3 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 3 {
            return Err("map in which the keys are formatted as 'x,y,z' and where all the keys numbers are greater or equal to 0");
        }

        let x = coords[0].parse::<u32>().map_err(|_| "Could not parse x coordinate")?;
        let y = coords[1].parse::<u32>().map_err(|_| "Could not parse y coordinate")?;
        let z = coords[2].parse::<u32>().map_err(|_| "Could not parse z coordinate")?;

        Ok(UVec3::new(x, y, z))
    },
    |uvec3: &UVec3| format!("{},{},{}", uvec3.x, uvec3.y, uvec3.z),
    "map in which the keys are formatted as 'x,y,z' and where all the keys numbers are greater or equal to 0"
);

impl_map_with_vec_keys_serializer_and_deserializer!(
    IVec2 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 2 {
            return Err("map in which the keys are formatted as 'x,y'");
        }

        let x = coords[0].parse::<i32>().map_err(|_| "Could not parse x coordinate")?;
        let y = coords[1].parse::<i32>().map_err(|_| "Could not parse y coordinate")?;

        Ok(IVec2::new(x, y))
    },
    |ivec2: &IVec2| format!("{},{}", ivec2.x, ivec2.y),
    "map in which the keys are formatted as 'x,y'"
);

impl_map_with_vec_keys_serializer_and_deserializer!(
    IVec3 => [HashMap, IndexMap],
    |key: String| {
        let coords: Vec<&str> = key.split(',').collect();
        if coords.len() != 3 {
            return Err("map in which the keys are formatted as 'x,y,z'");
        }

        let x = coords[0].parse::<i32>().map_err(|_| "Could not parse x coordinate")?;
        let y = coords[1].parse::<i32>().map_err(|_| "Could not parse y coordinate")?;
        let z = coords[2].parse::<i32>().map_err(|_| "Could not parse z coordinate")?;

        Ok(IVec3::new(x, y, z))
    },
    |ivec3: &IVec3| format!("{},{},{}", ivec3.x, ivec3.y, ivec3.z),
    "map in which the keys are formatted as 'x,y,z'"
);
