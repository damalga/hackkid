#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FixtureKind {
    Toilet,
    Sink,
    Shower,
    Urinal,
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub x: f64,
    pub y: f64,
    pub kind: FixtureKind,
    pub paper_units: u32,
}

impl Fixture {
    pub fn new(x: f64, y: f64, kind: FixtureKind) -> Self {
        Self { x, y, kind, paper_units: 0 }
    }

    pub fn with_paper(mut self, units: u32) -> Self {
        self.paper_units = units;
        self
    }
}
