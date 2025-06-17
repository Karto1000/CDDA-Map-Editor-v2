use crate::features::program_data::{ProgramData, Project};
use crate::util::{Load, Save, SaveError};
use anyhow::Error;
use log::{error, info};
use std::fs;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct ProgramDataSaver {
    pub path: PathBuf,
}

impl Save<ProgramData> for ProgramDataSaver {
    async fn save(&self, data: &ProgramData) -> Result<(), SaveError> {
        let serialized_data = serde_json::to_string_pretty(data)?;
        fs::write(self.path.join("config.json"), serialized_data)?;
        info!("Saved EditorData to {}", self.path.display());
        Ok(())
    }
}

pub struct ProgramDataLoader {
    pub path: PathBuf,
}

impl ProgramDataLoader {
    pub fn load(&mut self) -> Result<ProgramData, Error> {
        let data = fs::read_to_string(self.path.join("config.json"))?;

        let editor_data: ProgramData = serde_json::from_str(&data)?;
        info!("Loaded EditorData from {}", self.path.display());

        Ok(editor_data)
    }
}

pub struct ProjectSaver {
    pub path: PathBuf,
}

impl Save<Project> for ProjectSaver {
    async fn save(&self, data: &Project) -> Result<(), SaveError> {
        let serialized_project = serde_json::to_string_pretty(data)?;

        let mut file = File::create(&self.path).await?;
        file.write_all(serialized_project.as_bytes()).await?;

        info!("Saved project {} to {}", data.name, self.path.display());

        Ok(())
    }
}

pub struct ProjectLoader {
    pub path: PathBuf,
}

impl ProjectLoader {
    pub fn load(&mut self) -> Result<Project, Error> {
        let data = match fs::read_to_string(&self.path) {
            Ok(d) => d,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Cannot find project at path {:?} skipping; {}",
                    self.path,
                    e
                ));
            },
        };

        Ok(serde_json::from_str(&data)?)
    }
}
