use crate::util::{CDDAIdentifier, MeabyVec, Weighted};
use serde::{Deserialize, Serialize};

const fn default_probability() -> i32 {
    100
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Item {
    pub item: CDDAIdentifier,

    #[serde(default)]
    pub damage: Option<MeabyVec<i32>>,

    #[serde(default = "default_probability")]
    #[serde(rename = "prob")]
    pub probability: i32,

    #[serde(default)]
    pub count: Option<MeabyVec<i32>>,

    #[serde(default)]
    pub charges: Option<MeabyVec<i32>>,

    #[serde(default)]
    pub components: Option<Vec<String>>,

    #[serde(default)]
    pub contents_item: Option<MeabyVec<String>>,

    #[serde(default)]
    pub contents_group: Option<MeabyVec<String>>,

    #[serde(default)]
    pub ammo_item: Option<String>,

    #[serde(default)]
    pub ammo_group: Option<String>,

    #[serde(default)]
    pub container_group: Option<String>,

    #[serde(default)]
    pub entry_wrapper: Option<String>,

    #[serde(default)]
    pub sealed: Option<bool>,

    #[serde(default)]
    pub active: Option<bool>,

    #[serde(default)]
    pub custom_flags: Option<Vec<String>>,

    #[serde(default)]
    pub variant: Option<String>,

    #[serde(default)]
    pub event: Option<String>,

    #[serde(default)]
    pub snippets: Option<String>,
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

    #[serde(default)]
    pub count: Option<MeabyVec<i32>>,

    #[serde(default)]
    pub charges: Option<MeabyVec<i32>>,

    #[serde(default)]
    pub components: Option<Vec<String>>,

    #[serde(default)]
    pub contents_item: Option<MeabyVec<String>>,

    #[serde(default)]
    pub contents_group: Option<MeabyVec<String>>,

    #[serde(default)]
    pub ammo_item: Option<String>,

    #[serde(default)]
    pub ammo_group: Option<String>,

    #[serde(default)]
    pub container_group: Option<String>,

    #[serde(default)]
    pub entry_wrapper: Option<String>,

    #[serde(default)]
    pub sealed: Option<bool>,

    #[serde(default)]
    pub active: Option<bool>,

    #[serde(default)]
    pub custom_flags: Option<Vec<String>>,

    #[serde(default)]
    pub variant: Option<String>,

    #[serde(default)]
    pub event: Option<String>,

    #[serde(default)]
    pub snippets: Option<String>,
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
pub enum EntryItem {
    Item(Item),
    Group(Group),
    Distribution {
        distribution: Vec<EntryItem>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
    Collection {
        collection: Vec<EntryItem>,

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
        distribution: Vec<EntryItem>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
    Collection {
        collection: Vec<EntryItem>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
}

impl Into<EntryItem> for EntryItemShortcut {
    fn into(self) -> EntryItem {
        match self {
            EntryItemShortcut::NotWeighted(nw) => EntryItem::Item(Item::from(nw)),
            EntryItemShortcut::Weighted(w) => {
                let mut item = Item::from(w.data);
                item.probability = w.weight;
                EntryItem::Item(item)
            }
            EntryItemShortcut::Item(i) => EntryItem::Item(i),
            EntryItemShortcut::Group(g) => EntryItem::Group(g),
            EntryItemShortcut::Distribution {
                distribution,
                probability,
            } => EntryItem::Distribution {
                distribution: distribution.into_iter().map(|i| i.into()).collect(),
                probability,
            },
            EntryItemShortcut::Collection {
                collection,
                probability,
            } => EntryItem::Collection {
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
        distribution: Vec<EntryItem>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
    Collection {
        collection: Vec<EntryItem>,

        #[serde(rename = "prob")]
        probability: Option<i32>,
    },
}

impl Into<EntryItem> for EntryGroupShortcut {
    fn into(self) -> EntryItem {
        match self {
            EntryGroupShortcut::NotWeighted(nw) => EntryItem::Group(Group::from(nw)),
            EntryGroupShortcut::Weighted(w) => {
                let mut group = Group::from(w.data);
                group.probability = w.weight;
                EntryItem::Group(group)
            }
            EntryGroupShortcut::Item(i) => EntryItem::Item(i),
            EntryGroupShortcut::Group(g) => EntryItem::Group(g),
            EntryGroupShortcut::Distribution {
                distribution,
                probability,
            } => EntryItem::Distribution {
                distribution: distribution.into_iter().map(|i| i.into()).collect(),
                probability,
            },
            EntryGroupShortcut::Collection {
                collection,
                probability,
            } => EntryItem::Collection {
                collection: collection.into_iter().map(|i| i.into()).collect(),
                probability,
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntermediateItemGroup {
    pub id: CDDAIdentifier,

    #[serde(default)]
    pub subtype: ItemGroupSubtype,

    #[serde(default)]
    pub items: Vec<EntryItemShortcut>,

    #[serde(default)]
    pub groups: Vec<EntryGroupShortcut>,

    #[serde(default)]
    pub entries: Vec<EntryItem>,
}

impl Into<CDDDAItemGroup> for IntermediateItemGroup {
    fn into(self) -> CDDDAItemGroup {
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
            let entry_item: EntryItem = item.into();
            entries.push(entry_item);
        });

        // Turn groups into entires
        self.groups.into_iter().for_each(|group| {
            let entry_item: EntryItem = group.into();
            entries.push(entry_item);
        });

        entries.extend(self.entries);

        CDDDAItemGroup {
            id: self.id,
            subtype: self.subtype,
            entries,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CDDDAItemGroup {
    pub id: CDDAIdentifier,
    pub subtype: ItemGroupSubtype,
    pub entries: Vec<EntryItem>,
}
