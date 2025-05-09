use crate::cdda_data::map_data::MapGenMonsterType;
use crate::util::{CDDAIdentifier, GetIdentifier, ParameterIdentifier};
use indexmap::IndexMap;
use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const fn default_weight() -> u32 {
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
    pub weight: u32,
    #[serde(default = "default_cost_multiplier")]
    pub cost_multiplier: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAMonsterGroup {
    pub id: CDDAIdentifier,
    pub default: Option<CDDAIdentifier>,
    pub monsters: Vec<MonsterGroupEntry>,
}

impl CDDAMonsterGroup {
    pub fn get_random_monster(
        &self,
        monstergroups: &HashMap<CDDAIdentifier, CDDAMonsterGroup>,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Option<CDDAIdentifier> {
        let mut weights = vec![];
        self.monsters.iter().for_each(|m| weights.push(m.weight));

        let weighted_index = WeightedIndex::new(weights).expect("No Error");
        // TODO: Replace with RANDOM; Random not here due to deadlock
        let chosen_index = weighted_index.sample(&mut rand::rng());

        let chosen_monster = &self.monsters[chosen_index];

        let id = match &chosen_monster.id {
            MapGenMonsterType::Monster { monster } => monster.get_identifier(calculated_parameters),
            MapGenMonsterType::MonsterGroup { group } => {
                let id = group.get_identifier(calculated_parameters);
                let group = monstergroups.get(&id)?;
                group
                    .get_random_monster(monstergroups, calculated_parameters)?
                    .clone()
            }
        };

        Some(id)
    }
}
