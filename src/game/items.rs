use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    Laptop,
    PowerSupply,
    Canteen,
    Hoodie,
    Pants,
    HospitalGown,
    Scrubs,
    Refresco,
    RefrescoEnvase,
    Cafe,
    CafeEnvase,
    ToiletPaper,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    #[serde(default)]
    pub units: u32,
}

impl Item {
    pub fn new(kind: ItemKind) -> Self {
        Self { kind, units: 0 }
    }

    pub fn new_with_units(kind: ItemKind, units: u32) -> Self {
        Self { kind, units }
    }

    pub fn label(&self) -> &'static str {
        match self.kind {
            ItemKind::Laptop => "Portátil",
            ItemKind::PowerSupply => "Fuente de alimentación",
            ItemKind::Canteen => "Cantimplora",
            ItemKind::Hoodie => "Sudadera",
            ItemKind::Pants => "Pantalones",
            ItemKind::HospitalGown => "Bata de hospital",
            ItemKind::Scrubs => "Ropa de enfermería",
            ItemKind::Refresco => "Refresco",
            ItemKind::RefrescoEnvase => "Lata vacía",
            ItemKind::Cafe => "Café",
            ItemKind::CafeEnvase => "Vaso de café vacío",
            ItemKind::ToiletPaper => "Rollo de papel",
        }
    }

    pub fn dropped_scale(&self) -> f64 {
        match self.kind {
            ItemKind::Refresco | ItemKind::RefrescoEnvase | ItemKind::Cafe | ItemKind::CafeEnvase | ItemKind::ToiletPaper => 0.35,
            _ => 1.0,
        }
    }

    pub fn body_slot(&self) -> Option<crate::game::player::BodySlot> {
        use crate::game::player::BodySlot;
        Some(match self.kind {
            ItemKind::Hoodie => BodySlot::Body,
            ItemKind::Pants => BodySlot::Legs,
            ItemKind::HospitalGown => BodySlot::Body,
            ItemKind::Scrubs => BodySlot::Body,
            _ => return None,
        })
    }

    pub fn dropped_sprite_color(&self) -> (u8, u8, u8) {
        match self.kind {
            ItemKind::Laptop => (60, 60, 80),
            ItemKind::PowerSupply => (110, 90, 60),
            ItemKind::Canteen => (140, 130, 100),
            ItemKind::Hoodie => (110, 90, 140),
            ItemKind::Pants => (60, 65, 90),
            ItemKind::HospitalGown => (200, 210, 220),
            ItemKind::Scrubs => (60, 130, 150),
            ItemKind::Refresco => (200, 60, 60),
            ItemKind::RefrescoEnvase => (110, 90, 80),
            ItemKind::Cafe => (95, 60, 40),
            ItemKind::CafeEnvase => (150, 130, 110),
            ItemKind::ToiletPaper => (240, 232, 218),
        }
    }
}
