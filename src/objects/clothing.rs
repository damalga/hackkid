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
            ClothingKind::HospitalGown => "bata de hospital",
            ClothingKind::ScrubBundle => "ropa de enfermería",
            ClothingKind::Hoodie => "sudadera",
            ClothingKind::Pants => "pantalones",
        }
    }

    pub fn description(&self) -> &'static str {
        match self.kind {
            ClothingKind::HospitalGown =>
                "Una bata de hospital caída como si el cuerpo se hubiera esfumado dentro.",
            ClothingKind::ScrubBundle =>
                "Ropa de enfermería sobre el suelo. Nadie la llevaba cuando cayó.",
            ClothingKind::Hoodie =>
                "Una sudadera colgada del perchero. Parece de tu talla.",
            ClothingKind::Pants =>
                "Unos pantalones doblados sobre el banco.",
        }
    }

    pub fn is_pickable(&self) -> bool {
        self.elevation > 0.0
    }

    pub fn wardrobe_label(&self) -> &'static str {
        match self.kind {
            ClothingKind::HospitalGown => "Bata de hospital",
            ClothingKind::ScrubBundle => "Ropa de enfermería",
            ClothingKind::Hoodie => "Sudadera",
            ClothingKind::Pants => "Pantalones",
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
