use serde::{Deserialize, Serialize};

use crate::game::items::{Item, ItemKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backpack {
    pub x: f64,
    pub y: f64,
    pub contents: Vec<Item>,
}

impl Backpack {
    pub fn player_starter(x: f64, y: f64) -> Self {
        Self {
            x, y,
            contents: vec![
                Item::new(ItemKind::Laptop),
                Item::new(ItemKind::PowerSupply),
                Item::new(ItemKind::Canteen),
            ],
        }
    }

    pub fn label(&self) -> &str {
        "backpack"
    }
}
