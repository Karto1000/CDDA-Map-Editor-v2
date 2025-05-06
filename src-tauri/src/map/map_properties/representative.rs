use crate::cdda_data::io::DeserializedCDDAJsonData;
use crate::cdda_data::item::{ItemEntry, ItemGroupSubtype};
use crate::cdda_data::map_data::{MapGenItem, ReferenceOrInPlace};
use crate::map::RepresentativeProperty;
use crate::util::CDDAIdentifier;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum DisplayItemGroup {
    Single {
        item: CDDAIdentifier,
        probability: f32,
    },
    Collection {
        name: Option<String>,
        items: Vec<DisplayItemGroup>,
        probability: f32,
    },
    Distribution {
        name: Option<String>,
        items: Vec<DisplayItemGroup>,
        probability: f32,
    },
}

impl DisplayItemGroup {
    pub fn probability(&self) -> f32 {
        match self {
            DisplayItemGroup::Single { probability, .. } => probability.clone(),
            DisplayItemGroup::Collection { probability, .. } => probability.clone(),
            DisplayItemGroup::Distribution { probability, .. } => probability.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemProperty {
    pub items: Vec<MapGenItem>,
}

impl ItemProperty {
    fn get_display_items_from_entries(
        &self,
        entries: &Vec<ItemEntry>,
        json_data: &DeserializedCDDAJsonData,
        group_probability: f32,
    ) -> Vec<DisplayItemGroup> {
        let mut display_item_groups: Vec<DisplayItemGroup> = Vec::new();

        let weight_sum = entries.iter().fold(0, |acc, v| match v {
            ItemEntry::Item(i) => acc + i.probability,
            ItemEntry::Group(g) => acc + g.probability,
            ItemEntry::Distribution { probability, .. } => acc + probability.unwrap_or(100),
            ItemEntry::Collection { probability, .. } => acc + probability.unwrap_or(100),
        });

        for entry in entries.iter() {
            match entry {
                ItemEntry::Item(i) => {
                    let display_item = DisplayItemGroup::Single {
                        item: i.item.clone(),
                        probability: i.probability as f32 / weight_sum as f32 * group_probability,
                    };
                    display_item_groups.push(display_item);
                }
                ItemEntry::Group(g) => {
                    let other_group = &json_data
                        .item_groups
                        .get(&g.group)
                        .expect(format!("Item Group {} to exist", &g.group).as_str());

                    let probability = g.probability as f32 / weight_sum as f32 * group_probability;

                    let display_items = self.get_display_items_from_entries(
                        &other_group.common.entries,
                        json_data,
                        probability,
                    );

                    match other_group.common.subtype {
                        ItemGroupSubtype::Collection => {
                            display_item_groups.push(DisplayItemGroup::Collection {
                                items: display_items,
                                name: Some(other_group.id.clone().0),
                                probability,
                            });
                        }
                        ItemGroupSubtype::Distribution => {
                            display_item_groups.push(DisplayItemGroup::Distribution {
                                items: display_items,
                                name: Some(other_group.id.clone().0),
                                probability,
                            });
                        }
                    }
                }
                ItemEntry::Distribution {
                    distribution,
                    probability,
                } => {
                    let probability = probability
                        .map(|p| p as f32 / weight_sum as f32 * group_probability)
                        .unwrap_or(group_probability / weight_sum as f32);

                    let display_items =
                        self.get_display_items_from_entries(distribution, json_data, probability);

                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some("In-Place".to_string()),
                        items: display_items,
                        probability,
                    });
                }
                ItemEntry::Collection {
                    collection,
                    probability,
                } => {
                    let probability = probability
                        .map(|p| p as f32 / weight_sum as f32 * group_probability)
                        .unwrap_or(group_probability / weight_sum as f32);

                    let display_items =
                        self.get_display_items_from_entries(collection, json_data, probability);

                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some("In-Place".to_string()),
                        items: display_items,
                        probability,
                    });
                }
            }
        }

        display_item_groups.sort_by(|v1, v2| {
            v2.probability()
                .partial_cmp(&v1.probability())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        display_item_groups
    }
}

impl RepresentativeProperty for ItemProperty {
    fn representation(&self, json_data: &DeserializedCDDAJsonData) -> Value {
        let mut display_item_groups: Vec<DisplayItemGroup> = Vec::new();

        for mapgen_item in self.items.iter() {
            let item_group_entries = match &mapgen_item.item {
                ReferenceOrInPlace::Reference(i) => {
                    if i == &"office_paper".into() {
                        dbg!(&mapgen_item.chance);
                        dbg!("NOW");
                    }

                    &json_data
                        .item_groups
                        .get(&i)
                        .expect("Item group to exist")
                        .common
                }
                ReferenceOrInPlace::InPlace(ip) => &ip.common,
            };

            let probability = mapgen_item
                .chance
                .clone()
                .map(|v| v.get_from_to().0)
                .unwrap_or(100) as f32
                // the default chance is 100, but we want to have a range from 0-1 so / 100
                / 100.;

            let items = self.get_display_items_from_entries(
                &item_group_entries.entries,
                json_data,
                probability,
            );

            match &item_group_entries.subtype {
                ItemGroupSubtype::Collection => {
                    display_item_groups.push(DisplayItemGroup::Collection {
                        name: Some(mapgen_item.item.ref_or("Unnamed Collection").0),
                        probability,
                        items,
                    });
                }
                ItemGroupSubtype::Distribution => {
                    display_item_groups.push(DisplayItemGroup::Distribution {
                        name: Some(mapgen_item.item.ref_or("Unnamed Distribution").0),
                        probability,
                        items,
                    });
                }
            }
        }

        display_item_groups.sort_by(|v1, v2| {
            v2.probability()
                .partial_cmp(&v1.probability())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        serde_json::to_value(display_item_groups).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::cdda_data::map_data::{MapGenItem, ReferenceOrInPlace};
    use crate::cdda_data::NumberOrRange;
    use crate::map::map_properties::representative::ItemProperty;
    use crate::map::RepresentativeProperty;
    use crate::TEST_CDDA_DATA;
    use serde_json::json;

    #[tokio::test]
    async fn test_item_property() {
        let cdda_data = TEST_CDDA_DATA.get().await;

        let items = vec![MapGenItem {
            item: ReferenceOrInPlace::Reference("test_itemgroup".into()),
            chance: Some(NumberOrRange::Number(50)),
            repeat: None,
            faction: None,
        }];

        let item_property = ItemProperty { items };

        let repr = item_property.representation(cdda_data);

        assert_eq!(
            repr,
            json! {
                [
                    {
                        "items": [
                            {
                                "item": "rock",
                                "probability": 0.10000000149011612,
                                "type": "Single"
                            },
                            {
                                "item": "wood_panel",
                                "probability": 0.10000000149011612,
                                "type": "Single"
                            },
                            {
                                "item": "nail",
                                "probability": 0.10000000149011612,
                                "type": "Single"
                            },
                            {
                                "item": "splinter",
                                "probability": 0.10000000149011612,
                                "type": "Single"
                            },
                            {
                                "items": [
                                    {
                                        "item": "dirt",
                                        "probability": 0.10000000149011612,
                                        "type": "Single"
                                    }
                                ],
                                "name": "test_itemgroup2",
                                "probability": 0.10000000149011612,
                                "type": "Collection"
                            }
                        ],
                        "name": "test_itemgroup",
                        "probability": 0.5,
                        "type": "Collection"
                    },
                ]
            }
        );
    }
}
