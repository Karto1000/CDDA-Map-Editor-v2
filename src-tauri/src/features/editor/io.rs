use crate::features::program_data::Project;
use crate::util::{Load, Save, SaveError};
use anyhow::Error;
use std::path::PathBuf;

pub struct EditorSaver {
    pub path: PathBuf,
}

impl Save<Project> for EditorSaver {
    async fn save(&self, data: &Project) -> Result<(), SaveError> {
        todo!()
    }
}

pub struct EditorLoader {
    pub path: PathBuf,
}

impl Load<Project> for EditorLoader {
    async fn load(&mut self) -> Result<Project, Error> {
        todo!()
    }
}
