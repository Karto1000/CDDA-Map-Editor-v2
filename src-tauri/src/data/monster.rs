use cdda_lib::types::{CDDAIdentifier, CDDAString};
use cdda_macros::cdda_entry;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonsterName {
    pub str: CDDAString,
    pub str_pl: Option<CDDAString>,
}

#[cdda_entry]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMonster {
    pub id: CDDAIdentifier,
    pub source: Option<PathBuf>,

    pub flags: Vec<String>,
    pub name: MonsterName,
    pub description: CDDAString,
}
