use serde::{Deserialize, Serialize};

use crate::engine::map::Map;
use crate::game::items::{Item, ItemKind};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BodySlot {
    Head,
    Body,
    Hands,
    Legs,
    Feet,
}

/// Something worn. Most come from an [`Item`]; socks are only ever worn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Garment {
    HospitalGown,
    Scrubs,
    HeavyJacket,
    HospitalTrousers,
    Socks,
}

impl Garment {
    pub fn label(self) -> &'static str {
        match self {
            Garment::HospitalGown => "Hospital Gown",
            Garment::Scrubs => "Medical Scrubs",
            Garment::HeavyJacket => "Heavy Jacket",
            Garment::HospitalTrousers => "Hospital Trousers",
            Garment::Socks => "Socks",
        }
    }

    pub fn slot(self) -> BodySlot {
        match self {
            Garment::HospitalGown | Garment::Scrubs | Garment::HeavyJacket => BodySlot::Body,
            Garment::HospitalTrousers => BodySlot::Legs,
            Garment::Socks => BodySlot::Feet,
        }
    }

    /// The inventory item this garment turns back into when taken off.
    pub fn item(self) -> Option<Item> {
        let kind = match self {
            Garment::HospitalGown => ItemKind::HospitalGown,
            Garment::Scrubs => ItemKind::Scrubs,
            Garment::HeavyJacket => ItemKind::Hoodie,
            Garment::HospitalTrousers => ItemKind::Pants,
            Garment::Socks => return None,
        };
        Some(Item::new(kind))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wardrobe {
    pub head: Option<Garment>,
    pub body: Option<Garment>,
    pub hands: Option<Garment>,
    pub legs: Option<Garment>,
    pub feet: Option<Garment>,
}

impl Wardrobe {
    pub fn starter() -> Self {
        Self {
            head: None,
            body: Some(Garment::HospitalGown),
            hands: None,
            legs: None,
            feet: Some(Garment::Socks),
        }
    }

    /// Trousers add two pocket slots to the inventory.
    pub fn wearing_pants(&self) -> bool {
        self.legs == Some(Garment::HospitalTrousers)
    }

    fn slot_mut(&mut self, slot: BodySlot) -> &mut Option<Garment> {
        match slot {
            BodySlot::Head => &mut self.head,
            BodySlot::Body => &mut self.body,
            BodySlot::Hands => &mut self.hands,
            BodySlot::Legs => &mut self.legs,
            BodySlot::Feet => &mut self.feet,
        }
    }

    /// Puts `garment` on and returns whatever was worn in that slot before.
    pub fn wear(&mut self, garment: Garment) -> Option<Garment> {
        self.slot_mut(garment.slot()).replace(garment)
    }

    pub fn display_rows(&self) -> [(&'static str, String); 5] {
        let name = |g: Option<Garment>| g.map_or_else(|| "Nothing".to_string(), |g| g.label().to_string());
        let legs = match self.legs {
            Some(g) if self.wearing_pants() => format!("{} *", g.label()),
            Some(g) => g.label().to_string(),
            None => "Underwear".into(),
        };
        [
            ("Head:", name(self.head)),
            ("Body:", name(self.body)),
            ("Hands:", name(self.hands)),
            ("Legs:", legs),
            ("Feet:", name(self.feet)),
        ]
    }
}

const MOVE_SPEED: f64 = 0.16;
const SPRINT_SPEED: f64 = 0.28;
const MAX_STAMINA: f64 = 100.0;

/// How close the player's centre can get to a wall or a piece of furniture.
pub const PLAYER_RADIUS: f64 = 0.2;

/// Slots 0..6 are the backpack, 6..8 the trouser pockets.
pub const BACKPACK_SLOTS: usize = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<Item>>,
    pub capacity: usize,
}

impl Inventory {
    pub fn backpack() -> Self {
        Self { slots: vec![None; 8], capacity: BACKPACK_SLOTS }
    }

    pub fn add(&mut self, item: Item) -> Result<(), Item> {
        match self.slots.iter_mut().take(self.capacity).find(|s| s.is_none()) {
            Some(slot) => {
                *slot = Some(item);
                Ok(())
            }
            None => Err(item),
        }
    }

    pub fn has(&self, kind: ItemKind) -> bool {
        self.slots.iter().flatten().any(|it| it.kind == kind)
    }

    pub fn set_capacity(&mut self, cap: usize) {
        self.capacity = cap.min(self.slots.len());
    }

    pub fn weight_ratio(&self) -> f64 {
        let cap = self.capacity.max(1);
        let used = self.slots.iter().take(cap).filter(|s| s.is_some()).count();
        used as f64 / cap as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub body: f64,
    pub mind: f64,
    pub stamina: f64,
    pub thirst: f64,
    pub hunger: f64,
    pub sleep: f64,
    pub thermal: f64,
    pub hygiene: f64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            body: 80.0, mind: 70.0, stamina: MAX_STAMINA,
            thirst: 25.0, hunger: 20.0, sleep: 20.0,
            thermal: 0.0, hygiene: 80.0,
        }
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

/// A solid thing the player can't walk through, as a circle on the floor.
#[derive(Debug, Clone, Copy)]
pub struct Obstacle {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub x: f64,
    pub y: f64,
    pub dir_x: f64,
    pub dir_y: f64,
    pub plane_x: f64, // camera plane (perpendicular to dir, len = FOV factor)
    pub plane_y: f64,
    pub stats: Stats,
    pub inventory: Inventory,
    pub hidden: bool,
    #[serde(skip)]
    pub sprinting: bool,
    pub has_backpack: bool,
    pub inventory_cursor: usize,
    pub wardrobe: Wardrobe,
    pub has_map: bool,
    pub has_city_map: bool,
    pub canteen_fill: f64,
}

impl Player {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            dir_x: 1.0,
            dir_y: 0.0,
            plane_x: 0.0,
            plane_y: 0.66, // ~66 degree FOV
            stats: Stats::new(),
            inventory: Inventory::backpack(),
            hidden: false,
            sprinting: false,
            has_backpack: false,
            inventory_cursor: 0,
            wardrobe: Wardrobe::starter(),
            has_map: false,
            has_city_map: false,
            canteen_fill: 0.0,
        }
    }

    pub fn rotate(&mut self, angle: f64) {
        let cos = angle.cos();
        let sin = angle.sin();
        let old_dir_x = self.dir_x;
        self.dir_x = self.dir_x * cos - self.dir_y * sin;
        self.dir_y = old_dir_x * sin + self.dir_y * cos;
        let old_plane_x = self.plane_x;
        self.plane_x = self.plane_x * cos - self.plane_y * sin;
        self.plane_y = old_plane_x * sin + self.plane_y * cos;
    }

    /// Faces a compass direction (unit vector), keeping the ~66° field of view.
    pub fn face(&mut self, dir_x: f64, dir_y: f64) {
        self.dir_x = dir_x;
        self.dir_y = dir_y;
        self.plane_x = -dir_y * 0.66;
        self.plane_y = dir_x * 0.66;
    }

    pub fn move_forward(&mut self, map: &Map, obstacles: &[Obstacle]) {
        let speed = self.effective_speed();
        self.step(map, obstacles, self.dir_x * speed, self.dir_y * speed);
    }

    pub fn move_backward(&mut self, map: &Map, obstacles: &[Obstacle]) {
        let speed = self.effective_speed();
        self.step(map, obstacles, -self.dir_x * speed, -self.dir_y * speed);
    }

    /// Moves one axis at a time so the player slides along walls. A move is refused
    /// only if it pushes further into something: anyone who ends up overlapping a wall
    /// or a sofa (after standing up, or from an old save) can always step back out.
    fn step(&mut self, map: &Map, obstacles: &[Obstacle], dx: f64, dy: f64) {
        if dx != 0.0 && !blocked_x(map, self.x, self.y, dx) && !obstructed(obstacles, self.x, self.y, self.x + dx, self.y) {
            self.x += dx;
        }
        if dy != 0.0 && !blocked_y(map, self.x, self.y, dy) && !obstructed(obstacles, self.x, self.y, self.x, self.y + dy) {
            self.y += dy;
        }
    }

    fn effective_speed(&self) -> f64 {
        let base = if self.sprinting && self.stats.stamina > 0.0 {
            SPRINT_SPEED
        } else {
            MOVE_SPEED
        };
        base * (1.0 - self.inventory.weight_ratio() * 0.3)
    }

    /// True while any part of the player's body is inside tile (tx, ty).
    pub fn overlaps_tile(&self, tx: usize, ty: usize) -> bool {
        let (x0, y0) = (tx as f64, ty as f64);
        self.x + PLAYER_RADIUS > x0 && self.x - PLAYER_RADIUS < x0 + 1.0
            && self.y + PLAYER_RADIUS > y0 && self.y - PLAYER_RADIUS < y0 + 1.0
    }
}

/// Moving along x: the leading edge, at both ends of the body, must stay out of solid tiles.
fn blocked_x(map: &Map, x: f64, y: f64, dx: f64) -> bool {
    let edge = x + dx + PLAYER_RADIUS * dx.signum();
    [y - PLAYER_RADIUS * 0.99, y + PLAYER_RADIUS * 0.99]
        .iter()
        .any(|&py| map.at(edge, py).blocks_movement())
}

fn blocked_y(map: &Map, x: f64, y: f64, dy: f64) -> bool {
    let edge = y + dy + PLAYER_RADIUS * dy.signum();
    [x - PLAYER_RADIUS * 0.99, x + PLAYER_RADIUS * 0.99]
        .iter()
        .any(|&px| map.at(px, edge).blocks_movement())
}

fn obstructed(obstacles: &[Obstacle], x: f64, y: f64, nx: f64, ny: f64) -> bool {
    obstacles.iter().any(|o| {
        let reach = o.r + PLAYER_RADIUS;
        let d_new = (nx - o.x).powi(2) + (ny - o.y).powi(2);
        let d_old = (x - o.x).powi(2) + (y - o.y).powi(2);
        d_new < reach * reach && d_new < d_old
    })
}
