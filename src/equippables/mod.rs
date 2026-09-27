pub mod backpack;

pub use backpack::Backpack;

use crate::engine::sprites::SpriteShape;
use crate::game::items::Item;

pub enum Equippable {
    Backpack(Backpack),
}

impl Equippable {
    pub fn position(&self) -> (f64, f64) {
        match self {
            Equippable::Backpack(b) => (b.x, b.y),
        }
    }

    pub fn taken(&self) -> bool {
        match self {
            Equippable::Backpack(b) => b.taken,
        }
    }

    pub fn mark_taken(&mut self) {
        match self {
            Equippable::Backpack(b) => b.taken = true,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Equippable::Backpack(b) => b.label().into(),
        }
    }

    pub fn contents(&self) -> Vec<Item> {
        match self {
            Equippable::Backpack(b) => b.contents.clone(),
        }
    }

    pub fn sprite_shape(&self) -> SpriteShape {
        match self {
            Equippable::Backpack(_) => SpriteShape::Backpack,
        }
    }
}
