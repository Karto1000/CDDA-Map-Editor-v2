use crate::cdda_data::map_data::MapGenMonsterType;
use crate::cdda_data::{GetIdentifier, GetIdentifierError, WeightedIndexError};
use cdda_lib::types::{CDDAIdentifier, ParameterIdentifier};
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

const fn default_weight() -> i32 {
    1
}

const fn default_cost_multiplier() -> u32 {
    1
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonsterGroupEntry {
    #[serde(flatten)]
    pub id: MapGenMonsterType,
    #[serde(default = "default_weight")]
    pub weight: i32,
    #[serde(default = "default_cost_multiplier")]
    pub cost_multiplier: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMonsterGroup {
    pub id: CDDAIdentifier,
    pub default: Option<CDDAIdentifier>,
    pub monsters: Vec<MonsterGroupEntry>,
}

#[derive(Debug, Error)]
pub enum GetRandomMonsterError {
    #[error(transparent)]
    WeightedIndexError(#[from] WeightedIndexError),

    #[error(transparent)]
    GetIdentifierError(#[from] GetIdentifierError),

    #[error("Monstergroup {0} not found")]
    MissingMonstergroup(String),
}

impl CDDAMonsterGroup {
    pub fn get_random_monster(
        &self,
        monstergroups: &HashMap<CDDAIdentifier, CDDAMonsterGroup>,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, GetRandomMonsterError> {
        let mut weights = vec![];
        self.monsters.iter().for_each(|m| weights.push(m.weight));

        let weighted_index = WeightedIndex::new(weights.clone())
            .map_err(|_| WeightedIndexError::InvalidWeights(weights))?;

        // TODO: Replace with RANDOM; Random not here due to deadlock
        let chosen_index = weighted_index.sample(&mut rand::rng());

        let chosen_monster = &self.monsters[chosen_index];

        let id = match &chosen_monster.id {
            MapGenMonsterType::Monster { monster } => {
                monster.get_identifier(calculated_parameters)?
            },
            MapGenMonsterType::MonsterGroup { group } => {
                let id = group.get_identifier(calculated_parameters)?;
                let group = monstergroups.get(&id).ok_or(
                    GetRandomMonsterError::MissingMonstergroup(id.to_string()),
                )?;

                group
                    .get_random_monster(monstergroups, calculated_parameters)?
                    .clone()
            },
        };

        Ok(id)
    }
}
