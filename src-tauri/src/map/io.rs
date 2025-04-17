use crate::editor_data::Project;
use crate::map::MapData;
use crate::util::{Load, Save};
use std::fs::File;
use std::io::{BufReader, Error};
use std::path::PathBuf;

pub struct MapDataLoader {
    pub path: PathBuf,
}

impl Load<MapData> for MapDataLoader {
    fn load(&self) -> Result<MapData, anyhow::Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        serde_json::from_reader(reader).map_err(|e| anyhow::Error::from(e))
    }
}

pub struct ProjectSaver {
    pub path: PathBuf,
}

impl Save<Project> for ProjectSaver {
    fn save(&self, data: &Project) -> Result<(), Error> {
        let mut file = File::create(&self.path.join(&data.name))?;
        serde_json::to_writer(&mut file, data).map_err(|e| e.into())
    }
}
