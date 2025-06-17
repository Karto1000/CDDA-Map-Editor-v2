use crate::features::program_data::Project;
use crate::util::{Save, SaveError};
use std::path::PathBuf;

pub struct EditorSaver {
    pub path: PathBuf,
}

impl Save<Project> for EditorSaver {
    async fn save(&self, data: &Project) -> Result<(), SaveError> {
        todo!()
    }
}
