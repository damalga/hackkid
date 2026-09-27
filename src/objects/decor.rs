#[derive(Debug, Clone, Copy)]
pub enum DecorKind {
    AluminumTable,
    SurgicalCabinet,
    WoodenTable,
    TreeOak,
    TreePine,
    Bush,
    Weed,
    Car { body: (u8, u8, u8) },
    BusStop,
    Mailbox,
}

#[derive(Debug, Clone)]
pub struct Decor {
    pub x: f64,
    pub y: f64,
    pub kind: DecorKind,
    pub elevation: f64,
}

impl Decor {
    pub fn new(x: f64, y: f64, kind: DecorKind) -> Self {
        Self { x, y, kind, elevation: 0.0 }
    }

    pub fn with_elevation(mut self, e: f64) -> Self {
        self.elevation = e;
        self
    }
}
