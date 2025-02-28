use crate::map_data::{Cell, MapData};
use crate::palettes::Parameter;
use crate::util::{Load, MapGenValue, MeabyVec, ParameterIdentifier};
use anyhow::anyhow;
use glam::UVec2;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct ImportingMapDataObject {
    fill_ter: String,
    rows: Vec<String>,
    palettes: Vec<MapGenValue>,
    terrain: HashMap<char, MapGenValue>,
    parameters: Option<HashMap<ParameterIdentifier, Parameter>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImportingMapData {
    method: String,
    om_terrain: MeabyVec<String>,

    #[serde(rename = "//")]
    comment: Option<String>,

    #[serde(rename = "type")]
    ty: String,

    weight: Option<i32>,

    object: ImportingMapDataObject,
}

impl ImportingMapData {
    fn into(self, name: String) -> MapData {
        let mut cells = HashMap::new();

        for (row_index, row) in self.object.rows.into_iter().enumerate() {
            for (column_index, character) in row.chars().enumerate() {
                cells.insert(
                    UVec2::new(column_index as u32, row_index as u32),
                    Cell { character },
                );
            }
        }

        MapData::new(
            name,
            cells,
            self.object.terrain,
            self.object.palettes,
            self.object.parameters.unwrap_or_else(|| HashMap::new()),
        )
    }
}

pub struct MapDataImporter {
    pub path: PathBuf,
    pub om_terrain: String,
}

impl Load<MapData> for MapDataImporter {
    fn load(&self) -> Result<MapData, anyhow::Error> {
        let reader = BufReader::new(File::open(&self.path)?);
        let importing_map_datas: Vec<ImportingMapData> =
            serde_json::from_reader(reader).map_err(|e| anyhow::Error::from(e))?;

        let importing_map_data = importing_map_datas
            .into_iter()
            .find(|md| {
                md.om_terrain
                    .clone()
                    .vec()
                    .into_iter()
                    .find(|om_terrain| om_terrain == &self.om_terrain)
                    .is_some()
            })
            .ok_or(anyhow!("Could not find map data"))?;

        Ok(importing_map_data.into(self.om_terrain.clone()))
    }
}
