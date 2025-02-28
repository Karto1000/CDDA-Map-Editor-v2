use crate::palettes::Palette;
use crate::util::Load;
use log::{debug, info};
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct PaletteLoader {
    pub path: PathBuf,
}

impl Load<Vec<Palette>> for PaletteLoader {
    fn load(&self) -> Result<Vec<Palette>, anyhow::Error> {
        debug!("Loading {:?}", &self.path);

        let reader = BufReader::new(File::open(&self.path)?);
        let object: Value = serde_json::from_reader(reader)?;

        let array = object.as_array().unwrap();

        Ok(array
            .iter()
            .filter_map(|val| match serde_json::from_value::<Palette>(val.clone()) {
                Ok(p) => Some(p),
                Err(e) => {
                    info!("Could not deserialize entry in mapgen_palettes file {}", e);
                    return None;
                }
            })
            .collect())
    }
}
