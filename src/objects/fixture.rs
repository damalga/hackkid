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

pub fn private_bathroom(base_x: f64, base_y: f64) -> Vec<Fixture> {
    vec![
        Fixture::new(base_x, base_y, FixtureKind::Toilet).with_paper(10),
        Fixture::new(base_x + 1.0, base_y, FixtureKind::Sink),
        Fixture::new(base_x + 2.0, base_y, FixtureKind::Shower),
    ]
}

pub fn public_bathroom(base_x: f64, base_y: f64) -> Vec<Fixture> {
    vec![
        Fixture::new(base_x, base_y, FixtureKind::Sink),
        Fixture::new(base_x + 0.8, base_y, FixtureKind::Sink),
        Fixture::new(base_x + 1.8, base_y, FixtureKind::Toilet),
        Fixture::new(base_x + 2.8, base_y, FixtureKind::Urinal),
    ]
}
