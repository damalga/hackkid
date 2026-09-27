#[derive(Debug, Clone)]
pub struct Reception {
    pub x: f64,
    pub y: f64,
}

impl Reception {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
