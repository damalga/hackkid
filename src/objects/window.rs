#[derive(Debug, Clone)]
pub struct Window {
    pub x: f64,
    pub y: f64,
}

impl Window {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
