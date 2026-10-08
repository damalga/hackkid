use crate::engine::boxes::{BoxKind, Facing};
use crate::engine::sprites::SpriteShape;

/// Something that's only there to be seen (and walked round).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropKind {
    NurseCounter,
    Reception,
    KitchenCounter,
    ServingCounter,
    Stove,
    CafeTable,
    WoodTable,
    Desk,
    Generator,
    Car(u32),
    Ambulance,
    Dumpster,
    Gurney,
    Headboard,
    IvStand,
    Monitor,
    Wheelchair,
    Chair(u32),
    TrashBin,
    StreetLamp,
    Plant,
    Curtain,
    SurgicalLight,
    CoatRack,
    GownRack,
    InstrumentTable,
    TreeOak,
    TreePine,
    Bush,
    Weed,
    BusStop,
    Mailbox,
}

/// How a prop or container is drawn.
pub enum Visual {
    Box(BoxKind),
    Sprite(SpriteShape),
}

fn tuple(c: u32) -> (u8, u8, u8) {
    ((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

impl PropKind {
    pub fn visual(self) -> Visual {
        use PropKind as P;
        match self {
            P::NurseCounter => Visual::Box(BoxKind::NurseCounter),
            P::Reception => Visual::Box(BoxKind::Reception),
            P::KitchenCounter => Visual::Box(BoxKind::KitchenCounter),
            P::ServingCounter => Visual::Box(BoxKind::ServingCounter),
            P::Stove => Visual::Box(BoxKind::Stove),
            P::CafeTable => Visual::Box(BoxKind::CafeTable),
            P::WoodTable => Visual::Box(BoxKind::WoodTable),
            P::Desk => Visual::Box(BoxKind::Desk),
            P::Generator => Visual::Box(BoxKind::Generator),
            P::Car(c) => Visual::Box(BoxKind::Car(c)),
            P::Ambulance => Visual::Box(BoxKind::Ambulance),
            P::Dumpster => Visual::Box(BoxKind::Dumpster),
            P::Gurney => Visual::Box(BoxKind::Gurney),
            P::Headboard => Visual::Box(BoxKind::Headboard),
            P::IvStand => Visual::Sprite(SpriteShape::IvStand),
            P::Monitor => Visual::Sprite(SpriteShape::Monitor),
            P::Wheelchair => Visual::Sprite(SpriteShape::Wheelchair),
            P::Chair(c) => Visual::Sprite(SpriteShape::Chair { color: tuple(c) }),
            P::TrashBin => Visual::Sprite(SpriteShape::TrashBin),
            P::StreetLamp => Visual::Sprite(SpriteShape::StreetLamp),
            P::Plant => Visual::Sprite(SpriteShape::Plant),
            P::Curtain => Visual::Sprite(SpriteShape::Curtain),
            P::SurgicalLight => Visual::Sprite(SpriteShape::SurgicalLight),
            P::CoatRack => Visual::Sprite(SpriteShape::CoatRack),
            P::GownRack => Visual::Sprite(SpriteShape::GownRack),
            P::InstrumentTable => Visual::Sprite(SpriteShape::InstrumentTable),
            P::TreeOak => Visual::Sprite(SpriteShape::TreeOak),
            P::TreePine => Visual::Sprite(SpriteShape::TreePine),
            P::Bush => Visual::Sprite(SpriteShape::Bush),
            P::Weed => Visual::Sprite(SpriteShape::Weed),
            P::BusStop => Visual::Sprite(SpriteShape::BusStop),
            P::Mailbox => Visual::Sprite(SpriteShape::Mailbox),
        }
    }

    /// Half the width of a sprite's footprint, if you bump into it.
    pub fn solid_radius(self) -> Option<f64> {
        use PropKind as P;
        match self {
            P::IvStand | P::CoatRack | P::StreetLamp => Some(0.14),
            P::Monitor | P::TrashBin | P::Mailbox => Some(0.18),
            P::Chair(_) => Some(0.22),
            P::Wheelchair | P::Plant | P::InstrumentTable => Some(0.26),
            P::GownRack => Some(0.3),
            P::TreeOak | P::TreePine => Some(0.25),
            P::Bush => Some(0.45),
            P::BusStop => Some(0.35),
            P::Curtain | P::SurgicalLight | P::Weed => None,
            _ => None, // boxes collide by their own bounds
        }
    }
}

#[derive(Debug, Clone)]
pub struct Prop {
    pub x: f64,
    pub y: f64,
    pub kind: PropKind,
    pub facing: Facing,
    pub elevation: f64,
}

impl Prop {
    pub fn new(x: f64, y: f64, kind: PropKind, facing: Facing) -> Self {
        Self { x, y, kind, facing, elevation: 0.0 }
    }

    pub fn raised(mut self, e: f64) -> Self {
        self.elevation = e;
        self
    }
}
