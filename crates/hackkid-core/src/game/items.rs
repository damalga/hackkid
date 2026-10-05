use serde::{Deserialize, Serialize};

use crate::game::player::Garment;

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
    Snack,
    SnackEnvase,
    ToiletPaper,
    TriageKeycard,
    ArchiveClearance,
    CassetteTape,
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
            ItemKind::Laptop => "Emergency Terminal",
            ItemKind::PowerSupply => "Battery Pack",
            ItemKind::Canteen => "Water Flask",
            ItemKind::Hoodie => "Heavy Jacket",
            ItemKind::Pants => "Hospital Trousers",
            ItemKind::HospitalGown => "Hospital Gown",
            ItemKind::Scrubs => "Medical Scrubs",
            ItemKind::Refresco => "Energy Drink",
            ItemKind::RefrescoEnvase => "Empty Can",
            ItemKind::Cafe => "Coffee",
            ItemKind::CafeEnvase => "Empty Cup",
            ItemKind::Snack => "Energy Bar",
            ItemKind::SnackEnvase => "Empty Wrapper",
            ItemKind::ToiletPaper => "Paper Roll",
            ItemKind::TriageKeycard => "Triage Keycard",
            ItemKind::ArchiveClearance => "Archive Clearance",
            ItemKind::CassetteTape => "Cassette Tape 01",
        }
    }

    pub fn dropped_scale(&self) -> f64 {
        match self.kind {
            ItemKind::Refresco | ItemKind::RefrescoEnvase | ItemKind::Cafe | ItemKind::CafeEnvase | ItemKind::Snack | ItemKind::SnackEnvase | ItemKind::ToiletPaper
            | ItemKind::TriageKeycard | ItemKind::ArchiveClearance | ItemKind::CassetteTape => 0.35,
            _ => 1.0,
        }
    }

    /// The garment this item is, if it can be worn.
    pub fn garment(&self) -> Option<Garment> {
        Some(match self.kind {
            ItemKind::Hoodie => Garment::HeavyJacket,
            ItemKind::Pants => Garment::HospitalTrousers,
            ItemKind::HospitalGown => Garment::HospitalGown,
            ItemKind::Scrubs => Garment::Scrubs,
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
            ItemKind::Snack => (210, 160, 70),
            ItemKind::SnackEnvase => (130, 120, 100),
            ItemKind::ToiletPaper => (240, 232, 218),
            ItemKind::TriageKeycard => (220, 200, 60),
            ItemKind::ArchiveClearance => (60, 200, 220),
            ItemKind::CassetteTape => (180, 100, 60),
        }
    }
}
