use crate::cdda_data::map_data::CDDAMapData;
use crate::cdda_data::palettes::CDDAPalette;
use crate::cdda_data::region_settings::CDDARegionSettings;
use crate::cdda_data::CDDAJsonEntry;
use crate::util::{CDDAIdentifier, Load};
use anyhow::Error;
use log::{info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Default)]
pub struct DeserializedCDDAJsonData {
    pub palettes: HashMap<CDDAIdentifier, CDDAPalette>,
    pub mapgens: HashMap<CDDAIdentifier, CDDAMapData>,
    pub region_settings: HashMap<CDDAIdentifier, CDDARegionSettings>,
}

pub struct CDDADataLoader {
    pub json_path: PathBuf,
}

impl Load<DeserializedCDDAJsonData> for CDDADataLoader {
    fn load(&self) -> Result<DeserializedCDDAJsonData, Error> {
        let walkdir = WalkDir::new(&self.json_path);

        let mut cdda_data = DeserializedCDDAJsonData::default();

        for entry in walkdir {
            let entry = entry?;

            let extension = match entry.path().extension() {
                None => {
                    warn!("Entry {:?} does not have an extension", entry.path());
                    continue;
                }
                Some(e) => e,
            };

            if extension != "json" {
                info!("Skipping {:?} because it is not a json file", entry.path());
                continue;
            }

            info!("Reading and parsing json file at {:?}", entry.path());
            let reader = BufReader::new(File::open(entry.path())?);

            let des = match serde_json::from_reader::<BufReader<File>, Vec<CDDAJsonEntry>>(reader) {
                Ok(des) => des,
                Err(e) => {
                    warn!("Failed to deserialize {:?}, error: {}", entry.path(), e);
                    continue;
                }
            };

            for des_entry in des {
                match des_entry {
                    CDDAJsonEntry::Mapgen(mg) => {
                        for om_terrain in mg.om_terrain.clone().vec().into_iter() {
                            cdda_data
                                .mapgens
                                .insert(CDDAIdentifier(om_terrain), mg.clone());
                        }
                    }
                    CDDAJsonEntry::RegionSettings(rs) => {
                        cdda_data.region_settings.insert(rs.id.clone(), rs);
                    }
                    CDDAJsonEntry::Palette(p) => {
                        cdda_data.palettes.insert(p.id.clone(), p);
                    }
                }
            }
        }

        Ok(cdda_data)
    }
}
