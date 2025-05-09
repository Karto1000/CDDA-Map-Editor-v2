use crate::cdda_data::NumberOrRange;
use crate::util::{CDDAIdentifier, MeabyVec, Weighted};
use serde::{Deserialize, Serialize};

const fn default_probability() -> i32 {
    100
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Item {
    pub item: CDDAIdentifier,

    #[serde(default = "default_probability")]
    #[serde(rename = "prob")]
    pub probability: i32,

    #[serde(default)]
    pub count: Option<NumberOrRange<i32>>,
}

impl From<CDDAIdentifier> for Item {
    fn from(value: CDDAIdentifier) -> Self {
        Self {
            item: value,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Group {
    pub group: CDDAIdentifier,

    #[serde(default)]
    pub damage: Option<MeabyVec<i32>>,

    #[serde(default = "default_probability")]
    #[serde(rename = "prob")]
    pub probability: i32,
}

impl From<CDDAIdentifier> for Group {
    fn from(value: CDDAIdentifier) -> Self {
        Self {
            group: value,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemGroupSubtype {
    Collection,
    #[default]
    Distribution,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ItemEntry {
    Item(Item),
    Group(Group),
    Distribution {
        distribution: Vec<ItemEntry>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
    Collection {
        collection: Vec<ItemEntry>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EntryItemShortcut {
    NotWeighted(CDDAIdentifier),
    Weighted(Weighted<CDDAIdentifier>),
    Item(Item),
    Group(Group),
    Distribution {
        distribution: Vec<ItemEntry>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
    Collection {
        collection: Vec<ItemEntry>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
}

impl Into<ItemEntry> for EntryItemShortcut {
    fn into(self) -> ItemEntry {
        match self {
            EntryItemShortcut::NotWeighted(nw) => ItemEntry::Item(Item::from(nw)),
            EntryItemShortcut::Weighted(w) => {
                let mut item = Item::from(w.data);
                item.probability = w.weight;
                ItemEntry::Item(item)
            }
            EntryItemShortcut::Item(i) => ItemEntry::Item(i),
            EntryItemShortcut::Group(g) => ItemEntry::Group(g),
            EntryItemShortcut::Distribution {
                distribution,
                probability,
            } => ItemEntry::Distribution {
                distribution: distribution.into_iter().map(|i| i.into()).collect(),
                probability,
            },
            EntryItemShortcut::Collection {
                collection,
                probability,
            } => ItemEntry::Collection {
                collection: collection.into_iter().map(|i| i.into()).collect(),
                probability,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EntryGroupShortcut {
    NotWeighted(CDDAIdentifier),
    Weighted(Weighted<CDDAIdentifier>),
    Item(Item),
    Group(Group),
    Distribution {
        distribution: Vec<ItemEntry>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
    Collection {
        collection: Vec<ItemEntry>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
}

impl Into<ItemEntry> for EntryGroupShortcut {
    fn into(self) -> ItemEntry {
        match self {
            EntryGroupShortcut::NotWeighted(nw) => ItemEntry::Group(Group::from(nw)),
            EntryGroupShortcut::Weighted(w) => {
                let mut group = Group::from(w.data);
                group.probability = w.weight;
                ItemEntry::Group(group)
            }
            EntryGroupShortcut::Item(i) => ItemEntry::Item(i),
            EntryGroupShortcut::Group(g) => ItemEntry::Group(g),
            EntryGroupShortcut::Distribution {
                distribution,
                probability,
            } => ItemEntry::Distribution {
                distribution: distribution.into_iter().map(|i| i.into()).collect(),
                probability,
            },
            EntryGroupShortcut::Collection {
                collection,
                probability,
            } => ItemEntry::Collection {
                collection: collection.into_iter().map(|i| i.into()).collect(),
                probability,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAItemGroupIntermediate {
    pub id: CDDAIdentifier,

    #[serde(default)]
    pub subtype: ItemGroupSubtype,

    #[serde(default)]
    pub items: Vec<EntryItemShortcut>,

    #[serde(default)]
    pub groups: Vec<EntryGroupShortcut>,

    #[serde(default)]
    pub entries: Vec<ItemEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAItemGroupInPlace {
    #[serde(flatten)]
    pub common: CDDAItemGroupCommon,

    #[serde(default)]
    pub items: Vec<EntryItemShortcut>,

    #[serde(default)]
    pub groups: Vec<EntryGroupShortcut>,
}

impl Into<CDDAItemGroup> for CDDAItemGroupIntermediate {
    fn into(self) -> CDDAItemGroup {
        // This:
        //
        // "items": [ "<id-1>", [ "<id-2>", 10 ] ]
        //
        // means the same as:
        //
        // "entries": [ { "item": "<id-1>" }, { "item": "<id-2>", "prob": 10 } ]
        //
        // In other words: a single string denotes an item id; an array (which must contain a string and a number) denotes an item id and a probability.
        //
        // This is true for groups as well:
        //
        // "groups": [ "<id-1>", [ "<id-2>", 10 ] ]

        let mut entries = vec![];

        // Turn items into entries
        self.items.into_iter().for_each(|item| {
            let entry_item: ItemEntry = item.into();
            entries.push(entry_item);
        });

        // Turn groups into entries
        self.groups.into_iter().for_each(|group| {
            let entry_item: ItemEntry = group.into();
            entries.push(entry_item);
        });

        entries.extend(self.entries);

        CDDAItemGroup {
            id: self.id,
            common: CDDAItemGroupCommon {
                entries,
                subtype: self.subtype,
            },
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CDDAItemGroupCommon {
    pub entries: Vec<ItemEntry>,
    pub subtype: ItemGroupSubtype,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDAItemGroup {
    pub id: CDDAIdentifier,

    #[serde(flatten)]
    pub common: CDDAItemGroupCommon,
}
