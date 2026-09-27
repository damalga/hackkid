#[derive(Debug, Clone)]
pub struct CoatRack {
    pub x: f64,
    pub y: f64,
}

impl CoatRack {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
