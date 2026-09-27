#[derive(Debug, Clone)]
pub struct Bench {
    pub x: f64,
    pub y: f64,
}

impl Bench {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
