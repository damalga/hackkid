use crate::game::items::{Item, ItemKind};
use crate::game::player::Garment;

#[derive(Debug, Clone, Copy)]
pub enum ClothingKind {
    HospitalGown,
    ScrubBundle,
    Hoodie,
    Pants,
}

#[derive(Debug, Clone)]
pub struct Clothing {
    pub x: f64,
    pub y: f64,
    pub kind: ClothingKind,
    pub taken: bool,
    pub elevation: f64,
}

impl Clothing {
    pub fn new(x: f64, y: f64, kind: ClothingKind) -> Self {
        Self { x, y, kind, taken: false, elevation: 0.0 }
    }

    pub fn with_elevation(mut self, elev: f64) -> Self {
        self.elevation = elev;
        self
    }

    pub fn label(&self) -> &'static str {
        match self.kind {
            ClothingKind::HospitalGown => "hospital gown",
            ClothingKind::ScrubBundle => "medical scrubs",
            ClothingKind::Hoodie => "heavy jacket",
            ClothingKind::Pants => "hospital trousers",
        }
    }

    pub fn description(&self) -> &'static str {
        match self.kind {
            ClothingKind::HospitalGown =>
                "A hospital gown draped over the bed as if the body inside simply vanished.",
            ClothingKind::ScrubBundle =>
                "Medical scrubs collapsed on the floor. Nobody was wearing them when they fell.",
            ClothingKind::Hoodie =>
                "A heavy jacket hanging on the rack. Fits your frame.",
            ClothingKind::Pants =>
                "Hospital trousers neatly folded on the bench.",
        }
    }

    pub fn is_pickable(&self) -> bool {
        self.elevation > 0.0
    }

    pub fn garment(&self) -> Garment {
        match self.kind {
            ClothingKind::HospitalGown => Garment::HospitalGown,
            ClothingKind::ScrubBundle => Garment::Scrubs,
            ClothingKind::Hoodie => Garment::HeavyJacket,
            ClothingKind::Pants => Garment::HospitalTrousers,
        }
    }

    pub fn item(&self) -> Item {
        Item::new(match self.kind {
            ClothingKind::HospitalGown => ItemKind::HospitalGown,
            ClothingKind::ScrubBundle => ItemKind::Scrubs,
            ClothingKind::Hoodie => ItemKind::Hoodie,
            ClothingKind::Pants => ItemKind::Pants,
        })
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self.kind {
            ClothingKind::HospitalGown => (200, 210, 220),
            ClothingKind::ScrubBundle => (60, 130, 150),
            ClothingKind::Hoodie => (110, 90, 140),
            ClothingKind::Pants => (60, 65, 90),
        }
    }
}
