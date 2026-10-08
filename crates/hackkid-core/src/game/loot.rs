//! What's in each container, rolled once when the world is made (Project Zomboid style:
//! some cupboards hold what you need, most hold little or nothing).

use crate::game::items::{Item, ItemKind};
use crate::objects::ContainerKind;

/// A small, fast, seedable random generator (splitmix64).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        (self.next_u64() % 10_000) as f64 / 10_000.0 < p
    }

    pub fn range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next_u64() % (hi - lo + 1) as u64) as u32
    }
}

/// Each entry: the item, the chance it's there, and how many uses a pack holds.
fn table(kind: ContainerKind) -> &'static [(ItemKind, f64, u32)] {
    use ItemKind as I;
    match kind {
        // the bathroom cabinets: sometimes a painkiller, some gauze, little else
        ContainerKind::MedCabinet => &[(I::Ibuprofen, 0.30, 8), (I::Paracetamol, 0.22, 8), (I::Gauze, 0.25, 0), (I::Bandage, 0.18, 0), (I::Peroxide, 0.12, 0), (I::ToiletPaper, 0.15, 6)],
        ContainerKind::BedsideTable => &[(I::WaterBottle, 0.35, 0), (I::Crackers, 0.2, 0), (I::ChocolateBar, 0.15, 0), (I::Paracetamol, 0.08, 8)],
        ContainerKind::Fridge => &[(I::Sandwich, 0.45, 0), (I::JuiceBox, 0.5, 0), (I::Apple, 0.4, 0), (I::WaterBottle, 0.5, 0), (I::JuiceBox, 0.2, 0)],
        ContainerKind::Locker => &[(I::Scrubs, 0.3, 0), (I::Hoodie, 0.12, 0), (I::Pants, 0.2, 0), (I::ChocolateBar, 0.3, 0), (I::Ibuprofen, 0.15, 8)],
        ContainerKind::Shelf => &[(I::Bandage, 0.5, 0), (I::Gauze, 0.55, 0), (I::Peroxide, 0.35, 0), (I::ToiletPaper, 0.4, 10), (I::PowerSupply, 0.1, 0)],
        ContainerKind::StoreShelf => &[(I::ChocolateBar, 0.6, 0), (I::Crackers, 0.5, 0), (I::WaterBottle, 0.55, 0), (I::Paracetamol, 0.3, 8), (I::Ibuprofen, 0.3, 8)],
        ContainerKind::Desk => &[(I::ChocolateBar, 0.25, 0), (I::Ibuprofen, 0.15, 8), (I::PowerSupply, 0.08, 0)],
        ContainerKind::CrashCart => &[(I::Bandage, 0.7, 0), (I::Gauze, 0.7, 0), (I::Peroxide, 0.4, 0)],
        ContainerKind::FileCabinet => &[(I::ChocolateBar, 0.06, 0)],
        ContainerKind::GlassCabinet => &[(I::Peroxide, 0.5, 0), (I::Gauze, 0.5, 0), (I::Paracetamol, 0.3, 8), (I::Ibuprofen, 0.3, 8)],
        ContainerKind::KitchenCupboard => &[(I::Crackers, 0.5, 0), (I::WaterBottle, 0.4, 0), (I::Apple, 0.3, 0)],
    }
}

pub fn roll(kind: ContainerKind, rng: &mut Rng) -> Vec<Item> {
    let mut out = Vec::new();
    for &(item, p, units) in table(kind) {
        if rng.chance(p) {
            out.push(if units > 0 { Item::new_with_units(item, rng.range(units / 2, units)) } else { Item::new(item) });
        }
    }
    out
}
