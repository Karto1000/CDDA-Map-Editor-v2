pub(crate) mod furniture;
pub(crate) mod io;
pub(crate) mod item;
pub(crate) mod map_data;
pub(crate) mod monster;
pub(crate) mod palettes;
pub(crate) mod region_settings;
pub(crate) mod terrain;

use crate::cdda_data::furniture::CDDAFurnitureIntermediate;
use crate::cdda_data::item::CDDAItemGroupIntermediate;
use crate::cdda_data::map_data::CDDAMapDataIntermediate;
use crate::cdda_data::monster::CDDAMonsterGroup;
use crate::cdda_data::palettes::CDDAPaletteIntermediate;
use crate::cdda_data::region_settings::CDDARegionSettings;
use crate::cdda_data::terrain::CDDATerrainIntermediate;
use crate::util::{CDDAIdentifier, GetIdentifier, MeabyVec, MeabyWeighted, ParameterIdentifier};
use derive_more::Display;
use indexmap::IndexMap;
use num_traits::PrimInt;
use rand::distr::uniform::SampleUniform;
use rand::{rng, Rng};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

pub fn extract_comments<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
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

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum NumberOrRange<T: PrimInt + Clone + SampleUniform> {
    Number(T),
    Range((T, T)),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberOrArray<T: PrimInt + Clone + SampleUniform> {
    Number(T),
    Array(Vec<T>),
}

impl<'de, T: PrimInt + Clone + SampleUniform + Deserialize<'de>> Deserialize<'de>
    for NumberOrRange<T>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = NumberOrArray::<T>::deserialize(deserializer)?;
        match value {
            NumberOrArray::Number(n) => Ok(NumberOrRange::Number(n)),
            NumberOrArray::Array(arr) => match arr.len() {
                1 => Ok(NumberOrRange::Number(arr[0].clone())),
                2 => Ok(NumberOrRange::Range((arr[0].clone(), arr[1].clone()))),
                _ => Err(serde::de::Error::custom(
                    "Array must contain 1 or 2 elements",
                )),
            },
        }
    }
}

impl<T: PrimInt + Clone + SampleUniform> NumberOrRange<T> {
    pub fn rand_number(&self) -> T {
        match self.clone() {
            NumberOrRange::Number(n) => n,
            NumberOrRange::Range((from, to)) => {
                let mut rng = rng();
                //let mut rng = RANDOM.write().unwrap();
                let num = rng.random_range(from..to);
                num
            }
        }
    }

    pub fn is_random_hit(&self, default_upper_bound: T) -> bool {
        match self.clone() {
            NumberOrRange::Number(n) => {
                // This will always be true
                if n >= default_upper_bound {
                    return true;
                }

                let mut rng = rng();
                //let mut rng = RANDOM.write().unwrap();
                let num = rng.random_range(n..default_upper_bound);

                num == n
            }
            NumberOrRange::Range((from, to)) => {
                let mut rng = rng();
                //let mut rng = RANDOM.write().unwrap();
                let num = rng.random_range(from..to);

                num == from
            }
        }
    }

    pub fn get_from_to(&self) -> (T, T) {
        match self.clone() {
            NumberOrRange::Number(n) => (n, n),
            NumberOrRange::Range((from, to)) => (from, to),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAExtendOp {
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDADeleteOp {
    pub flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TileLayer {
    Terrain = 0,
    Furniture = 1,
    Trap = 2,
    Monster = 3,
    Field = 4,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CDDAString {
    String(String),
    StringMap { str: String },
}

#[derive(Debug, Clone, Deserialize)]
pub enum IdOrAbstract {
    #[serde(rename = "id")]
    Id(CDDAIdentifier),
    #[serde(rename = "abstract")]
    Abstract(CDDAIdentifier),
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnknownEntry {
    #[serde(flatten)]
    identifier: IdOrAbstract,

    #[serde(rename = "type")]
    ty: KnownCataVariant,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectGroup {
    pub id: CDDAIdentifier,
}

#[derive(Debug, Clone, Deserialize)]
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
    MonsterGroup(CDDAMonsterGroup),

    // -- UNUSED
    WeatherType,
    FieldType,
    #[serde(rename = "LOOT_ZONE")]
    LootZone,
    WeaponCategory,
    Vitamin,
    VehicleGroup,
    Vehicle,
    VehiclePart,
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
    OvermapLocation,
    OvermapTerrain,
    OvermapSpecial,
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Switch {
    pub param: ParameterIdentifier,
    pub fallback: CDDAIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Distribution {
    pub distribution: MeabyVec<MeabyWeighted<CDDAIdentifier>>,
}

// TODO: Kind of a hacky solution to a Stack Overflow problem that i experienced when using
// a self-referencing MapGenValue enum
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CDDADistributionInner {
    String(CDDAIdentifier),
    Param {
        param: ParameterIdentifier,
        fallback: Option<CDDAIdentifier>,
    },
    Switch {
        switch: Switch,
        cases: HashMap<CDDAIdentifier, CDDAIdentifier>,
    },
    // The distribution inside another distribution must have the 'distribution'
    // property. This is why we're using the Distribution struct here
    //                 --- Here ---
    // "palettes": [ { "distribution": [ [ "cabin_palette", 1 ], [ "cabin_palette_abandoned", 1 ] ] } ],
    Distribution(Distribution),
}

impl From<&str> for CDDADistributionInner {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl GetIdentifier for CDDADistributionInner {
    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        match self {
            CDDADistributionInner::String(s) => s.clone(),
            CDDADistributionInner::Distribution(d) => d.distribution.get(calculated_parameters),
            CDDADistributionInner::Param { param, fallback } => calculated_parameters
                .get(param)
                .map(|p| p.clone())
                .unwrap_or_else(|| fallback.clone().expect("Fallback to exist")),
            CDDADistributionInner::Switch { switch, cases } => {
                let id = calculated_parameters
                    .get(&switch.param)
                    .map(|p| p.clone())
                    .unwrap_or_else(|| switch.fallback.clone());

                cases.get(&id).expect("MapTo to exist").clone()
            }
        }
    }
}

// https://github.com/CleverRaven/Cataclysm-DDA/blob/master/doc/JSON/MAPGEN.md#mapgen-values
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapGenValue {
    String(CDDAIdentifier),
    Param {
        param: ParameterIdentifier,
        fallback: Option<CDDAIdentifier>,
    },
    Switch {
        switch: Switch,
        cases: HashMap<CDDAIdentifier, CDDAIdentifier>,
    },
    // TODO: We could probably use a MapGenValue instead of this DistributionInner type, but that would
    // require a Box<> since we don't know the size. I tried this but for some reason it causes a Stack Overflow
    // because serde keeps infinitely calling the Deserialize function even though it should deserialize to the String variant.
    // I'm not sure if this is a bug with my logic or if this is some sort of oversight in serde.
    Distribution(MeabyVec<MeabyWeighted<CDDADistributionInner>>),
}

impl GetIdentifier for MapGenValue {
    fn get_identifier(
        &self,
        calculated_parameters: &IndexMap<ParameterIdentifier, CDDAIdentifier>,
    ) -> CDDAIdentifier {
        match self {
            MapGenValue::String(s) => s.clone(),
            MapGenValue::Distribution(d) => d.get(calculated_parameters),
            MapGenValue::Param { param, fallback } => calculated_parameters
                .get(param)
                .map(|p| p.clone())
                .unwrap_or_else(|| fallback.clone().expect("Fallback to exist")),
            MapGenValue::Switch { switch, cases } => {
                let id = calculated_parameters
                    .get(&switch.param)
                    .map(|p| p.clone())
                    .unwrap_or_else(|| switch.fallback.clone());

                cases.get(&id).expect("case MapTo to exist").clone()
            }
        }
    }
}
