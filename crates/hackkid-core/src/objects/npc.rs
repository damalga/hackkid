use crate::engine::sprites::{Hair, PersonLook, Top};

/// The other survivors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Who {
    /// Julian ("Jules"): a biology student in his early twenties. He woke up a week ago,
    /// in ICU 106, and has been checking on you every day since.
    Julian,
    /// Selenia: in her late fifties, the first to wake, twenty-three days ago. She has
    /// made a camp in the on-call room and knows where everything is.
    Selenia,
}

#[derive(Debug, Clone)]
pub struct Npc {
    pub x: f64,
    pub y: f64,
    pub who: Who,
    pub name: String,
    /// The room they keep to: (min x, min y, max x, max y). They wander inside it by day
    /// and stand still at night.
    pub home: (f64, f64, f64, f64),
    /// Where they're walking to, if anywhere.
    pub target: Option<(f64, f64)>,
    /// When to pick a new target (the frontend's clock, ms).
    pub next_move_ms: u64,
}

impl Npc {
    pub fn julian(x: f64, y: f64, home: (f64, f64, f64, f64)) -> Self {
        Self { x, y, who: Who::Julian, name: "Julian (Jules)".into(), home, target: None, next_move_ms: 0 }
    }

    pub fn selenia(x: f64, y: f64, home: (f64, f64, f64, f64)) -> Self {
        Self { x, y, who: Who::Selenia, name: "Selenia".into(), home, target: None, next_move_ms: 0 }
    }

    /// Metres per second when wandering: an unhurried walk.
    pub fn pace(&self) -> f64 {
        match self.who {
            Who::Julian => 0.8,
            Who::Selenia => 0.55,
        }
    }

    pub fn look(&self) -> PersonLook {
        match self.who {
            // a week without shaving, glasses, a hoodie from a staff locker over scrub
            // trousers, and the hospital wristband he never cut off
            Who::Julian => PersonLook {
                height: 1.8,
                build: 0.95,
                skin: (222, 184, 152),
                hair: (62, 42, 30),
                hair_style: Hair::Messy,
                top: (92, 110, 72),
                top_style: Top::Hoodie,
                under: (210, 210, 200),
                legs: (82, 108, 116),
                shoes: (228, 228, 222),
                glasses: true,
                stubble: true,
                lined: false,
                bag: None,
                wristband: true,
            },
            // grey hair pinned up, a long cardigan over a blouse, and the tote bag she
            // has carried round the hospital for three weeks
            Who::Selenia => PersonLook {
                height: 1.63,
                build: 1.08,
                skin: (204, 162, 130),
                hair: (184, 182, 176),
                hair_style: Hair::Bun,
                top: (122, 44, 56),
                top_style: Top::Cardigan,
                under: (228, 220, 198),
                legs: (44, 50, 74),
                shoes: (96, 62, 40),
                glasses: false,
                stubble: false,
                lined: true,
                bag: Some((198, 182, 142)),
                wristband: true,
            },
        }
    }
}
