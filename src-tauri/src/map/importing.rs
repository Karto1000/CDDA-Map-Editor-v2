use crate::cdda_data::map_data::{CDDAMapData, OmTerrain};
use crate::editor_data::Project;
use crate::map::DEFAULT_MAP_DATA_SIZE;
use crate::util::Load;
use anyhow::anyhow;
use glam::UVec2;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct MapDataImporter {
    pub path: PathBuf,
    pub om_terrain: String,
}

impl Load<Project> for MapDataImporter {
    fn load(&self) -> Result<Project, anyhow::Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        let importing_map_datas: Vec<CDDAMapData> =
            serde_json::from_reader(reader).map_err(|e| anyhow::Error::from(e))?;

        // TODO: Handle multiple z-levels
        let project = importing_map_datas
            .into_iter()
            .find_map(|md| match &md.om_terrain {
                OmTerrain::Single(s) => match &self.om_terrain == s {
                    true => {
                        let mut project = Project::new(s.clone(), DEFAULT_MAP_DATA_SIZE);
                        project.maps.insert(0, md.into());
                        Some(project)
                    }
                    false => None,
                },
                OmTerrain::Duplicate(duplicate) => {
                    match duplicate.iter().find(|d| *d == &self.om_terrain) {
                        Some(s) => {
                            let mut project = Project::new(s.clone(), DEFAULT_MAP_DATA_SIZE);
                            project.maps.insert(0, md.into());
                            Some(project)
                        }
                        None => None,
                    }
                }
                OmTerrain::Nested(n) => match n.iter().flatten().find(|s| *s == &self.om_terrain) {
                    None => None,
                    Some(s) => {
                        let mut project = Project::new(
                            s.clone(),
                            UVec2::new(
                                (n.len() * DEFAULT_MAP_DATA_SIZE.x as usize) as u32,
                                (n.first().unwrap().len() * DEFAULT_MAP_DATA_SIZE.y as usize)
                                    as u32,
                            ),
                        );
                        project.maps.insert(0, md.into());

                        Some(project)
                    }
                },
            })
            .ok_or(anyhow!("Could not find map data"))?;

        Ok(project)
    }
}
