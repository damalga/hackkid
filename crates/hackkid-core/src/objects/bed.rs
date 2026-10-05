#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BedKind {
    Hospital,
    Operating,
}

#[derive(Debug, Clone)]
pub struct Bed {
    pub x: f64,
    pub y: f64,
    pub kind: BedKind,
}

impl Bed {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, kind: BedKind::Hospital }
    }

    pub fn operating(x: f64, y: f64) -> Self {
        Self { x, y, kind: BedKind::Operating }
    }
}
