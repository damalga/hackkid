#[derive(Debug, Clone)]
pub struct Sofa {
    pub x: f64,
    pub y: f64,
}

impl Sofa {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
