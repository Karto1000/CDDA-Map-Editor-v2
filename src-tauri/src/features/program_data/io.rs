use crate::features::program_data::{EditorData, Project};
use crate::util::{Load, Save, SaveError};
use anyhow::Error;
use log::{error, info};
use std::fs;
use std::path::PathBuf;

pub struct ProgramDataSaver {
    pub path: PathBuf,
}

impl Save<EditorData> for ProgramDataSaver {
    async fn save(&self, data: &EditorData) -> Result<(), SaveError> {
        let serialized_data = serde_json::to_string_pretty(data)?;

        for project in data.loaded_projects.iter() {
            let serialized_project = serde_json::to_string_pretty(project)?;

            fs::write(
                self.path.join(format!("{}.json", project.name)),
                serialized_project,
            )?;
            info!("Saved project {} to {}", project.name, self.path.display());
        }

        fs::write(self.path.join("config.json"), serialized_data)?;
        info!("Saved EditorData to {}", self.path.display());
        Ok(())
    }
}

pub struct ProgramDataLoader {
    pub path: PathBuf,
}

impl ProgramDataLoader {
    pub fn load(&mut self) -> Result<EditorData, Error> {
        let data = fs::read_to_string(self.path.join("config.json"))?;

        let mut editor_data: EditorData = serde_json::from_str(&data)?;
        info!("Loaded EditorData from {}", self.path.display());

        for project_name in editor_data.openable_projects.iter() {
            let data = match fs::read_to_string(
                self.path.join(format!("{}.json", project_name)),
            ) {
                Ok(d) => d,
                Err(e) => {
                    error!("Cannot find project with name {} at path {:?} skipping; {}", project_name, self.path, e);
                    continue;
                },
            };

            let project: Project = serde_json::from_str(&data)?;
            editor_data.loaded_projects.push(project);
        }

        Ok(editor_data)
    }
}
