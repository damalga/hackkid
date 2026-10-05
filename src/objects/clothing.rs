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

    pub fn wardrobe_label(&self) -> &'static str {
        match self.kind {
            ClothingKind::HospitalGown => "Hospital Gown",
            ClothingKind::ScrubBundle => "Medical Scrubs",
            ClothingKind::Hoodie => "Heavy Jacket",
            ClothingKind::Pants => "Hospital Trousers",
        }
    }

    pub fn body_slot(&self) -> crate::game::player::BodySlot {
        use crate::game::player::BodySlot;
        match self.kind {
            ClothingKind::HospitalGown => BodySlot::Body,
            ClothingKind::ScrubBundle => BodySlot::Body,
            ClothingKind::Hoodie => BodySlot::Body,
            ClothingKind::Pants => BodySlot::Legs,
        }
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
