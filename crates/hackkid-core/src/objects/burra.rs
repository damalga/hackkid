#[derive(Debug, Clone)]
pub struct Burra {
    pub x: f64,
    pub y: f64,
}

impl Burra {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
