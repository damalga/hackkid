use crate::engine::boxes::Facing;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BedKind {
    Hospital,
    Operating,
    /// A plain bed in the on-call room.
    Cot,
}

/// A bed you can lie on. `facing` is where its foot end points.
#[derive(Debug, Clone)]
pub struct Bed {
    pub x: f64,
    pub y: f64,
    pub kind: BedKind,
    pub facing: Facing,
}

impl Bed {
    pub fn new(x: f64, y: f64, facing: Facing) -> Self {
        Self { x, y, kind: BedKind::Hospital, facing }
    }

    pub fn operating(x: f64, y: f64, facing: Facing) -> Self {
        Self { x, y, kind: BedKind::Operating, facing }
    }
}
