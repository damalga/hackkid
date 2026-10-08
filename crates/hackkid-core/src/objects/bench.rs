use crate::engine::boxes::Facing;

/// A bench you can sit on: the corridor's waiting benches, the chapel's pews.
#[derive(Debug, Clone)]
pub struct Bench {
    pub x: f64,
    pub y: f64,
    pub facing: Facing,
    pub pew: bool,
}

impl Bench {
    pub fn new(x: f64, y: f64, facing: Facing) -> Self {
        Self { x, y, facing, pew: false }
    }

    pub fn pew(x: f64, y: f64, facing: Facing) -> Self {
        Self { x, y, facing, pew: true }
    }
}
