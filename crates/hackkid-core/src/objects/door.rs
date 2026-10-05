#[derive(Debug, Clone, Copy)]
pub enum DoorKind {
    Regular,
    Main,
    Bathroom,
}

#[derive(Debug, Clone, Copy)]
pub enum KnobSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Door {
    pub tx: usize,
    pub ty: usize,
    pub kind: DoorKind,
    pub knob_color: (u8, u8, u8),
    pub knob_side: KnobSide,
    pub open_since_ms: Option<u64>,
    pub auto_close_ms: Option<u64>,
}

impl Door {
    pub fn new(tx: usize, ty: usize) -> Self {
        Self {
            tx, ty,
            kind: DoorKind::Regular,
            knob_color: (220, 190, 60),
            knob_side: KnobSide::Right,
            open_since_ms: None,
            auto_close_ms: None,
        }
    }

    pub fn main(tx: usize, ty: usize) -> Self {
        Self {
            tx, ty,
            kind: DoorKind::Main,
            knob_color: (210, 210, 220),
            knob_side: KnobSide::Right,
            open_since_ms: None,
            auto_close_ms: None,
        }
    }

    pub fn street(tx: usize, ty: usize) -> Self {
        Self {
            tx, ty,
            kind: DoorKind::Main,
            knob_color: (210, 210, 220),
            knob_side: KnobSide::Right,
            open_since_ms: None,
            auto_close_ms: Some(4000),
        }
    }

    pub fn bathroom(tx: usize, ty: usize) -> Self {
        Self {
            tx, ty,
            kind: DoorKind::Bathroom,
            knob_color: (120, 130, 140),
            knob_side: KnobSide::Right,
            open_since_ms: None,
            auto_close_ms: None,
        }
    }

    pub fn knob_u_range(&self) -> (f64, f64) {
        match (self.kind, self.knob_side) {
            (DoorKind::Regular, KnobSide::Left) => (0.10, 0.20),
            (DoorKind::Regular, KnobSide::Right) => (0.80, 0.90),
            (DoorKind::Bathroom, KnobSide::Left) => (0.10, 0.20),
            (DoorKind::Bathroom, KnobSide::Right) => (0.80, 0.90),
            (DoorKind::Main, KnobSide::Left) => (0.06, 0.22),
            (DoorKind::Main, KnobSide::Right) => (0.78, 0.94),
        }
    }

    pub fn knob_v_range(&self) -> (f64, f64) {
        match self.kind {
            DoorKind::Regular => (0.45, 0.58),
            DoorKind::Bathroom => (0.45, 0.58),
            DoorKind::Main => (0.40, 0.62),
        }
    }

    pub fn has_secondary_knob(&self) -> bool {
        matches!(self.kind, DoorKind::Main)
    }

    pub fn secondary_knob_u_range(&self) -> (f64, f64) {
        match self.knob_side {
            KnobSide::Right => (0.06, 0.22),
            KnobSide::Left => (0.78, 0.94),
        }
    }
}
