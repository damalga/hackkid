#[derive(Debug, Clone, Copy)]
pub enum SignKind {
    RoomSmall,
    MainSquare,
    HallwayLong,
    Bathroom,
    HospitalPlaque,
}

#[derive(Debug, Clone)]
pub struct Sign {
    pub x: f64,
    pub y: f64,
    pub kind: SignKind,
}

impl Sign {
    pub fn new(x: f64, y: f64, kind: SignKind) -> Self {
        Self { x, y, kind }
    }
}
