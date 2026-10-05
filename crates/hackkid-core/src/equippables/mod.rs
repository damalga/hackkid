pub mod backpack;

pub use backpack::Backpack;

use serde::{Deserialize, Serialize};

use crate::engine::sprites::SpriteShape;

/// Something lying in the world that the player can put on. Picking it up removes it
/// from the world's list; taking it off again puts a new one back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Equippable {
    Backpack(Backpack),
}

impl Equippable {
    pub fn position(&self) -> (f64, f64) {
        match self {
            Equippable::Backpack(b) => (b.x, b.y),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Equippable::Backpack(b) => b.label().into(),
        }
    }

    pub fn sprite_shape(&self) -> SpriteShape {
        match self {
            Equippable::Backpack(_) => SpriteShape::Backpack,
        }
    }
}
