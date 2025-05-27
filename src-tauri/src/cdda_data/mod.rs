use rand::distr::Distribution;
pub mod furniture;
pub mod io;
pub mod item;
pub mod map_data;
mod monster;
mod monster_group;
pub mod overmap;
pub mod palettes;
pub mod region_settings;
pub mod terrain;
pub mod vehicle_parts;
pub mod vehicles;

use crate::cdda_data::furniture::{CDDAFurniture, CDDAFurnitureIntermediate};
use crate::cdda_data::item::CDDAItemGroupIntermediate;
use crate::cdda_data::map_data::CDDAMapDataIntermediate;
use crate::cdda_data::monster_group::CDDAMonsterGroupIntermediate;
use crate::cdda_data::overmap::{
    CDDAOvermapLocationIntermediate, CDDAOvermapSpecialIntermediate,
    CDDAOvermapTerrainIntermediate,
};
use crate::cdda_data::palettes::CDDAPaletteIntermediate;
use crate::cdda_data::region_settings::{CDDARegionSettings, RegionIdentifier};
use crate::cdda_data::terrain::{CDDATerrain, CDDATerrainIntermediate};
use crate::cdda_data::vehicle_parts::CDDAVehiclePartIntermediate;
use crate::cdda_data::vehicles::CDDAVehicleIntermediate;
use crate::tileset::GetRandom;
use cdda_lib::types::{
    CDDADistributionInner, CDDAIdentifier, DistributionInner, IdOrAbstract,
    MapGenValue, MeabyVec, MeabyWeighted, ParameterIdentifier,
};
use derive_more::Display;
use indexmap::IndexMap;
use num_traits::PrimInt;
use rand::distr::uniform::SampleUniform;
use rand::distr::weighted::WeightedIndex;
use rand::{rng, Rng};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::ops::{Add, Rem, Sub};
use strum_macros::EnumIter;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GetIdentifierError {
    #[error(transparent)]
    GetRandomError(#[from] GetRandomError),

    #[error("Missing fallback for non existing parameter {0}")]
    MissingFallback(String),

    #[error("Missing value in case {0} for switch {1}")]
    MissingSwitchCaseValue(String, String),
}

#[derive(Debug, Error, Serialize)]
pub enum WeightedIndexError {
    #[error("Invalid weights for weighted index {0:?}")]
    InvalidWeights(Vec<i32>),
}

#[derive(Debug, Error, Serialize)]
pub enum GetRandomError {
    #[error(transparent)]
    WeightedIndexError(#[from] WeightedIndexError),

    #[error("Failed to get the identifier for the chosen item at index {0}")]
    GetIdentifierError(usize),
}

pub fn extract_comments<'de, D>(
    deserializer: D,
) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut comments = Vec::new();

    let map: BTreeMap<String, Value> = Deserialize::deserialize(deserializer)?;

    for (key, value) in &map {
        if key.starts_with("//") {
            if let Some(comment) = value.as_str() {
                comments.push(comment.to_string());
            }
        }
    }

    Ok(comments)
}

pub fn replace_region_setting(
    id: &CDDAIdentifier,
    region_setting: &CDDARegionSettings,
    terrain: &HashMap<CDDAIdentifier, CDDATerrain>,
    furniture: &HashMap<CDDAIdentifier, CDDAFurniture>,
) -> CDDAIdentifier {
    // If it starts with t_region, we know it is a regional setting
    if id.starts_with("t_region") {
        if id.starts_with("f_") {
            return replace_region_setting(
                region_setting
                    .region_terrain_and_furniture
                    .furniture
                    .get(&RegionIdentifier(id.0.clone()))
                    .expect("Furniture Region identifier to exist")
                    .get_random(),
                region_setting,
                terrain,
                furniture,
            );
        } else if id.0.starts_with("t_") {
            return replace_region_setting(
                region_setting
                    .region_terrain_and_furniture
                    .terrain
                    .get(&RegionIdentifier(id.0.clone()))
                    .expect("Terrain Region identifier to exist")
                    .get_random(),
                region_setting,
                terrain,
                furniture,
            );
        }
    }

    id.clone()
}

impl GetIdentifier for DistributionInner {
    type Error = Infallible;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Infallible> {
        match self {
            DistributionInner::Param { param, fallback } => {
                Ok(calculated_parameters
                    .get(param)
                    .map(|p| p.clone())
                    .unwrap_or(fallback.clone()))
            },
            DistributionInner::Normal(n) => Ok(n.clone()),
            _ => todo!(),
        }
    }
}

impl GetIdentifier for CDDAIdentifier {
    type Error = Infallible;

    fn get_identifier(
        &self,
        _calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Infallible> {
        Ok(self.clone())
    }
}

impl GetIdentifier for CDDADistributionInner {
    type Error = GetIdentifierError;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, GetIdentifierError> {
        match self {
            CDDADistributionInner::String(s) => Ok(s.clone()),
            CDDADistributionInner::Distribution(d) => {
                Ok(d.distribution.get_identifier(calculated_parameters)?)
            },
            CDDADistributionInner::Param { param, fallback } => {
                let calculated = calculated_parameters
                    .get(param)
                    .map(|p| Ok(p.clone()))
                    .unwrap_or_else(|| {
                        fallback.clone().ok_or(
                            GetIdentifierError::MissingFallback(
                                param.0.clone(),
                            ),
                        )
                    })?;

                Ok(calculated)
            },
            CDDADistributionInner::Switch { switch, cases } => {
                let id = calculated_parameters
                    .get(&switch.param)
                    .map(|p| p.clone())
                    .unwrap_or_else(|| switch.fallback.clone());

                cases
                    .get(&id)
                    .ok_or(GetIdentifierError::MissingSwitchCaseValue(
                        id.0,
                        switch.param.0.clone(),
                    ))
                    .map(Clone::clone)
            },
        }
    }
}

impl GetIdentifier for MapGenValue {
    type Error = GetIdentifierError;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, GetIdentifierError> {
        match self {
            MapGenValue::String(s) => Ok(s.clone()),
            MapGenValue::Distribution(d) => {
                Ok(d.get_identifier(calculated_parameters)?)
            },
            MapGenValue::Param { param, fallback } => calculated_parameters
                .get(param)
                .map(|p| Ok(p.clone()))
                .unwrap_or_else(|| {
                    fallback.clone().ok_or(GetIdentifierError::MissingFallback(
                        param.0.clone(),
                    ))
                }),
            MapGenValue::Switch { switch, cases } => {
                let id = calculated_parameters
                    .get(&switch.param)
                    .map(|p| p.clone())
                    .unwrap_or_else(|| switch.fallback.clone());

                cases
                    .get(&id)
                    .ok_or(GetIdentifierError::MissingSwitchCaseValue(
                        id.0,
                        switch.param.0.clone(),
                    ))
                    .map(Clone::clone)
            },
        }
    }
}

impl<T: Clone + GetIdentifier> GetIdentifier for MeabyVec<MeabyWeighted<T>> {
    type Error = GetRandomError;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Self::Error> {
        let mut weights = vec![];
        let mut self_vec = self.clone().into_vec();

        self.for_each(|v| weights.push(v.weight_or_one()));

        let weighted_index = WeightedIndex::new(weights.clone())
            .map_err(|_| WeightedIndexError::InvalidWeights(weights.clone()))?;

        // let mut rng = RANDOM.write().unwrap();
        let mut rng = rng();

        let chosen_index = weighted_index.sample(&mut rng);
        let item = self_vec.remove(chosen_index);

        item.data()
            .get_identifier(calculated_parameters)
            .map_err(|_| GetRandomError::GetIdentifierError(chosen_index))
    }
}

#[derive(
    Debug,
    Clone,
    Hash,
    Eq,
    Ord,
    PartialOrd,
    PartialEq,
    EnumIter,
    Deserialize,
    Serialize,
)]
pub enum TileLayer {
    Terrain = 0,
    Furniture = 1,
    Monster = 2,
    Field = 3,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnknownEntry {
    #[serde(flatten)]
    pub id: IdOrAbstract<CDDAIdentifier>,

    #[serde(rename = "type")]
    pub ty: KnownCataVariant,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectGroup {
    pub id: CDDAIdentifier,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CDDAJsonEntry {
    // TODO: Handle update_mapgen_id
    Mapgen(CDDAMapDataIntermediate),
    RegionSettings(CDDARegionSettings),
    Palette(CDDAPaletteIntermediate),
    Terrain(CDDATerrainIntermediate),
    Furniture(CDDAFurnitureIntermediate),
    ConnectGroup(ConnectGroup),
    ItemGroup(CDDAItemGroupIntermediate),
    #[serde(rename = "monstergroup")]
    MonsterGroup(CDDAMonsterGroupIntermediate),
    OvermapLocation(CDDAOvermapLocationIntermediate),
    OvermapTerrain(CDDAOvermapTerrainIntermediate),
    OvermapSpecial(CDDAOvermapSpecialIntermediate),
    Vehicle(CDDAVehicleIntermediate),
    VehiclePart(CDDAVehiclePartIntermediate),

    // -- UNUSED
    WeatherType,
    FieldType,
    #[serde(rename = "LOOT_ZONE")]
    LootZone,
    WeaponCategory,
    Vitamin,
    VehicleGroup,
    Uncraft,
    Widget,
    StartLocation,
    MissionDefinition,
    Speech,
    #[serde(rename = "SPECIES")]
    Species,
    Snippet,
    Scenario,
    RotatableSymbol,
    Requirement,
    Trap,
    SpeedDescription,
    ScentType,
    VehiclePlacement,
    #[serde(rename = "MAGAZINE")]
    Magazine,
    #[serde(rename = "GUNMOD")]
    GunMod,
    #[serde(rename = "GUN")]
    Gun,
    #[serde(rename = "GENERIC")]
    Generic,
    #[serde(rename = "COMESTIBLE")]
    Comestible,
    #[serde(rename = "AMMO")]
    Ammo,
    #[serde(rename = "BOOK")]
    Book,
    #[serde(rename = "ARMOR")]
    Armor,
    #[serde(rename = "PET_ARMOR")]
    PetArmor,
    #[serde(rename = "TOOL_ARMOR")]
    ToolArmor,
    EffectType,
    #[serde(rename = "TOOL")]
    Tool,
    AmmunitionType,
    HitRange,
    Profession,
    HarvestDropType,
    Harvest,
    Gate,
    Recipe,
    EventStatistic,
    Technique,
    Skill,
    SkillDisplayType,
    Score,
    Fault,
    FaultFix,
    EndScreen,
    EffectOnCondition,
    Enchantment,
    Emit,
    Achievement,
    AddictionType,
    AmmoEffect,
    Anatomy,
    #[serde(rename = "SPELL")]
    Spell,
    RelicProcgenData,
    AsciiArt,
    AttackVector,
    Bionic,
    TerFurnTransform,
    JmathFunction,
    JsonFlag,
    VehiclePartCategory,
    ToolQuality,
    VehicleSpawn,
    RecipeCategory,
    Practice,
    NestedCategory,
    RecipeGroup,
    EventTransformation,
    Proficiency,
    ProficiencyCategory,
    ProfessionGroup,
    ProfessionItemSubstitutions,
    ActivityType,
    OterVision,
    OvermapLandUseCode,
    OvermapConnection,
    CityBuilding,
    MapExtra,
    #[serde(rename = "MIGRATION")]
    Migration,
    TrapMigration,
    #[serde(rename = "TRAIT_MIGRATION")]
    TraitMigration,
    OvermapSpecialMigration,
    OterIdMigration,
    BodyPart,
    NpcClass,
    TerFurnMigration,
    VehiclePartMigration,
    CampMigration,
    TemperatureRemovalBlacklist,
    #[serde(rename = "SCENARIO_BLACKLIST")]
    ScenarioBlacklist,
    ChargeRemovalBlacklist,
    TalkTopic,
    Mutation,
    Npc,
    TraitGroup,
    ShopkeeperConsumption,
    ShopkeeperBlacklist,
    ShopkeeperConsumptionRates,
    Behavior,
    VarMigration,
    Faction,
    MutationType,
    OverlayOrder,
    MutationCategory,
    MovementMode,
    MoraleType,
    MoodFace,
    WeakpointSet,
    #[serde(rename = "MONSTER_BLACKLIST")]
    MonsterBlacklist,
    MonsterAttack,
    #[serde(rename = "MONSTER_FACTION")]
    MonsterFaction,
    #[serde(rename = "MONSTER")]
    Monster,
    MartialArt,
    MonsterFlag,
    Material,
    LimbScore,
    #[serde(rename = "ITEM_CATEGORY")]
    ItemCategory,
    ItemAction,
    #[serde(rename = "WHEEL")]
    Wheel,
    #[serde(rename = "ENGINE")]
    Engine,
    #[serde(rename = "TOOLMOD")]
    ToolMod,
    #[serde(rename = "BIONIC_ITEM")]
    BionicItem,
    Dream,
    DiseaseType,
    Construction,
    DamageType,
    ConstructionGroup,
    ConstructionCategory,
    DamageInfoOrder,
    Conduct,
    ClothingMod,
    ClimbingAid,
    CharacterMod,
    ButcheryRequirement,
    SubBodyPart,
    BodyGraph,
    #[serde(rename = "ITEM")]
    Item,
    FaultGroup,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnownCataVariant {
    OvermapSpecialId,
    Palette,
    RegionSettings,
    Mapgen,
    ConnectGroup,
    #[serde(rename = "monstergroup")]
    MonsterGroup,
    #[serde(other)]
    Other,
}

pub trait GetIdentifier {
    type Error;

    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> Result<CDDAIdentifier, Self::Error>;
}
