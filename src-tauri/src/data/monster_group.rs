use crate::data::{GetIdentifier, GetIdentifierError, WeightedIndexError};
use cdda_lib::types::{CDDAIdentifier, NumberOrRange, ParameterIdentifier};
use cdda_macros::cdda_entry;
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

const fn default_pack_size() -> NumberOrRange<u32> {
    NumberOrRange::Number(1)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MonsterGroupMonsterKind {
    Group { group: CDDAIdentifier },
    Monster { monster: CDDAIdentifier },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonsterGroupMonster {
    #[serde(flatten)]
    pub id: MonsterGroupMonsterKind,

    #[serde(default = "default_weight")]
    pub weight: i32,

    #[serde(default = "default_cost_multiplier")]
    pub cost_multiplier: u32,

    #[serde(default = "default_pack_size")]
    pub pack_size: NumberOrRange<u32>,
}

#[cdda_entry]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMonsterGroup {
    pub id: CDDAIdentifier,
    pub monsters: Vec<MonsterGroupMonster>,
    pub flags: Vec<String>,
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
            MonsterGroupMonsterKind::Monster { monster } => {
                monster.get_identifier(calculated_parameters).unwrap()
            },
            MonsterGroupMonsterKind::Group { group } => {
                let id = group.get_identifier(calculated_parameters).unwrap();
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
