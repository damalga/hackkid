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
    // food and drink found around the hospital
    WaterBottle,
    Crackers,
    ChocolateBar,
    Apple,
    Sandwich,
    JuiceBox,
    // first aid
    Ibuprofen,
    Paracetamol,
    Bandage,
    Gauze,
    Peroxide,
    // light
    Flashlight,
    Batteries,
    // the research annex
    LabAccess,
    Adrenaline,
    VitaminShot,
    Antibiotics,
    SutureKit,
    // on your feet
    Sneakers,
    Boots,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    #[serde(default)]
    pub units: u32,
}

impl ItemKind {
    /// How many of these share one slot: food, drink, pills (doses), paper (uses),
    /// first aid, batteries. 1 means one per slot.
    pub fn stack_max(self) -> u32 {
        use ItemKind as I;
        match self {
            I::Ibuprofen | I::Paracetamol | I::ToiletPaper => 24,
            I::WaterBottle | I::JuiceBox | I::Crackers | I::ChocolateBar | I::Apple | I::Sandwich => 5,
            I::Refresco | I::Cafe | I::Snack | I::RefrescoEnvase | I::CafeEnvase | I::SnackEnvase => 5,
            I::Bandage | I::Gauze | I::Peroxide | I::Batteries => 6,
            I::Antibiotics => 24,
            I::Adrenaline | I::VitaminShot | I::SutureKit => 4,
            _ => 1,
        }
    }
}

impl Item {
    pub fn new(kind: ItemKind) -> Self {
        Self { kind, units: 0 }
    }

    /// How many are in the slot (an item with no count is one).
    pub fn count(&self) -> u32 {
        self.units.max(1)
    }

    /// Uses one up; true if the slot is now empty.
    pub fn take_one(&mut self) -> bool {
        self.units = self.count() - 1;
        self.units == 0
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
            ItemKind::WaterBottle => "Water Bottle",
            ItemKind::Crackers => "Crackers",
            ItemKind::ChocolateBar => "Chocolate Bar",
            ItemKind::Apple => "Apple",
            ItemKind::Sandwich => "Sandwich",
            ItemKind::JuiceBox => "Juice Box",
            ItemKind::Ibuprofen => "Ibuprofen",
            ItemKind::Paracetamol => "Paracetamol",
            ItemKind::Bandage => "Bandage",
            ItemKind::Gauze => "Sterile Gauze",
            ItemKind::Peroxide => "Hydrogen Peroxide",
            ItemKind::Flashlight => "Flashlight",
            ItemKind::Batteries => "Batteries",
            ItemKind::LabAccess => "Lab Access Card",
            ItemKind::Adrenaline => "Adrenaline Shot",
            ItemKind::VitaminShot => "Vitamin Shot",
            ItemKind::Antibiotics => "Antibiotics",
            ItemKind::SutureKit => "Suture Kit",
            ItemKind::Sneakers => "Sneakers",
            ItemKind::Boots => "Work Boots",
        }
    }

    pub fn dropped_scale(&self) -> f64 {
        match self.kind {
            ItemKind::Refresco | ItemKind::RefrescoEnvase | ItemKind::Cafe | ItemKind::CafeEnvase | ItemKind::Snack | ItemKind::SnackEnvase | ItemKind::ToiletPaper
            | ItemKind::TriageKeycard | ItemKind::ArchiveClearance | ItemKind::CassetteTape
            | ItemKind::WaterBottle | ItemKind::Crackers | ItemKind::ChocolateBar | ItemKind::Apple | ItemKind::Sandwich
            | ItemKind::JuiceBox | ItemKind::Ibuprofen | ItemKind::Paracetamol | ItemKind::Bandage | ItemKind::Gauze
            | ItemKind::Peroxide | ItemKind::Flashlight | ItemKind::Batteries | ItemKind::LabAccess | ItemKind::Adrenaline
            | ItemKind::VitaminShot | ItemKind::Antibiotics | ItemKind::SutureKit => 0.35,
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
            ItemKind::Sneakers => Garment::Sneakers,
            ItemKind::Boots => Garment::Boots,
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
            ItemKind::WaterBottle => (150, 200, 230),
            ItemKind::Crackers => (210, 170, 90),
            ItemKind::ChocolateBar => (110, 60, 40),
            ItemKind::Apple => (200, 40, 40),
            ItemKind::Sandwich => (220, 200, 150),
            ItemKind::JuiceBox => (240, 150, 40),
            ItemKind::Ibuprofen | ItemKind::Paracetamol => (236, 236, 236),
            ItemKind::Bandage | ItemKind::Gauze => (240, 236, 226),
            ItemKind::Peroxide => (120, 70, 40),
            ItemKind::Flashlight => (60, 60, 64),
            ItemKind::Batteries => (210, 160, 40),
            ItemKind::LabAccess => (60, 120, 200),
            ItemKind::Adrenaline => (230, 90, 90),
            ItemKind::VitaminShot => (240, 190, 70),
            ItemKind::Antibiotics => (236, 236, 236),
            ItemKind::SutureKit => (90, 150, 140),
            ItemKind::Sneakers => (230, 230, 226),
            ItemKind::Boots => (110, 72, 40),
        }
    }
}
