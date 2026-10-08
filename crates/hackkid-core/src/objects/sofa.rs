use crate::engine::boxes::Facing;

#[derive(Debug, Clone)]
pub struct Sofa {
    pub x: f64,
    pub y: f64,
    pub facing: Facing,
}

impl Sofa {
    pub fn new(x: f64, y: f64, facing: Facing) -> Self {
        Self { x, y, facing }
    }
}
