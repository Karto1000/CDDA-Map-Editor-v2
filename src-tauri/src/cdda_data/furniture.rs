use crate::util::CDDAIdentifier;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CDDAFurniture {
    pub id: CDDAIdentifier,
    pub name: Option<String>,
    pub description: Option<String>,
    pub symbol: char,
    pub looks_like: Option<CDDAIdentifier>,
}
