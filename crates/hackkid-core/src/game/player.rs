use serde::{Deserialize, Serialize};

use crate::engine::camera::{FOV_PLANE, MAX_PITCH};
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
    Sneakers,
    Boots,
}

impl Garment {
    pub fn label(self) -> &'static str {
        match self {
            Garment::HospitalGown => "Hospital Gown",
            Garment::Scrubs => "Medical Scrubs",
            Garment::HeavyJacket => "Heavy Jacket",
            Garment::HospitalTrousers => "Hospital Trousers",
            Garment::Socks => "Socks",
            Garment::Sneakers => "Sneakers",
            Garment::Boots => "Work Boots",
        }
    }

    pub fn slot(self) -> BodySlot {
        match self {
            Garment::HospitalGown | Garment::Scrubs | Garment::HeavyJacket => BodySlot::Body,
            Garment::HospitalTrousers => BodySlot::Legs,
            Garment::Socks | Garment::Sneakers | Garment::Boots => BodySlot::Feet,
        }
    }

    /// The inventory item this garment turns back into when taken off.
    pub fn item(self) -> Option<Item> {
        let kind = match self {
            Garment::HospitalGown => ItemKind::HospitalGown,
            Garment::Scrubs => ItemKind::Scrubs,
            Garment::HeavyJacket => ItemKind::Hoodie,
            Garment::HospitalTrousers => ItemKind::Pants,
            Garment::Sneakers => ItemKind::Sneakers,
            Garment::Boots => ItemKind::Boots,
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

/// Walking, running and sneaking, in metres per second.
pub const WALK_SPEED: f64 = 2.3;
pub const RUN_SPEED: f64 = 11.0;
pub const CROUCH_SPEED: f64 = 0.75;
/// Backwards and sideways are slower than straight ahead.
const BACK_FACTOR: f64 = 0.65;
const SIDE_FACTOR: f64 = 0.85;
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

    /// Puts an item away, stacking it with the same kind where that's allowed. Gives it
    /// back if there's no room.
    pub fn add(&mut self, mut item: Item) -> Result<(), Item> {
        let max = item.kind.stack_max();
        if max > 1 {
            for slot in self.slots.iter_mut().take(self.capacity).flatten() {
                if slot.kind == item.kind && slot.count() < max {
                    let room = max - slot.count();
                    let moved = item.count().min(room);
                    slot.units = slot.count() + moved;
                    if moved >= item.count() {
                        return Ok(());
                    }
                    item.units = item.count() - moved;
                }
            }
        }
        match self.slots.iter_mut().take(self.capacity).find(|s| s.is_none()) {
            Some(slot) => {
                *slot = Some(item);
                Ok(())
            }
            None => Err(item),
        }
    }

    /// How many of a kind are carried, counting stacks.
    pub fn count(&self, kind: ItemKind) -> u32 {
        self.slots.iter().flatten().filter(|it| it.kind == kind).map(Item::count).sum()
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
    /// How you feel against the comfort temperature: below 0 cold, above 0 hot.
    pub thermal: f64,
    pub hygiene: f64,
    /// Painkillers on an empty stomach: builds up with each pill, wears off with time.
    #[serde(default)]
    pub stomach: f64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            body: 80.0, mind: 70.0, stamina: MAX_STAMINA,
            thirst: 25.0, hunger: 20.0, sleep: 20.0,
            thermal: 0.0, hygiene: 80.0, stomach: 0.0,
        }
    }
}

/// Where the player is trying to go, set by the frontend and integrated over time.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Motion {
    /// -1 back, 0, 1 forward.
    pub forward: f64,
    /// -1 left, 0, 1 right.
    pub side: f64,
    pub run: bool,
    /// The frontend's clock when this intent lapses (a key held keeps extending it).
    pub until_ms: u64,
    /// Keyboard turning: -1 left, 1 right, with its own deadline.
    pub turn: f64,
    pub turn_until_ms: u64,
    /// When the current turn direction started: holding past a threshold accelerates it.
    pub turn_started_ms: u64,
    /// Radians still owed to a one-off animated spin (double-tap S for 180°).
    pub spin_remaining: f64,
    /// If set, pitch animates toward this angle instead of snapping (Z and X).
    pub pitch_target: Option<f64>,
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

/// A solid thing the player can't walk through: its footprint on the floor.
#[derive(Debug, Clone, Copy)]
pub struct Obstacle {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Obstacle {
    /// A square footprint `r` either side of a point.
    pub fn around(x: f64, y: f64, r: f64) -> Self {
        Self { x0: x - r, y0: y - r, x1: x + r, y1: y + r }
    }

    /// How far a point is from the footprint (0 inside it).
    pub fn distance(&self, x: f64, y: f64) -> f64 {
        let dx = (self.x0 - x).max(0.0).max(x - self.x1);
        let dy = (self.y0 - y).max(0.0).max(y - self.y1);
        (dx * dx + dy * dy).sqrt()
    }
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
    pub has_backpack: bool,
    pub inventory_cursor: usize,
    pub wardrobe: Wardrobe,
    pub has_map: bool,
    pub has_city_map: bool,
    pub canteen_fill: f64,
    /// Looking up (positive) or down, radians.
    #[serde(default)]
    pub pitch: f64,
    /// Metres walked, for the stride of the feet seen when looking down.
    #[serde(skip)]
    pub stride: f64,
    #[serde(default)]
    pub flashlight_on: bool,
    /// Charge of the flashlight's batteries, 0..100.
    #[serde(default = "full_charge")]
    pub flashlight_charge: f64,
}

fn full_charge() -> f64 {
    100.0
}

impl Player {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            dir_x: 1.0,
            dir_y: 0.0,
            plane_x: 0.0,
            plane_y: FOV_PLANE,
            stats: Stats::new(),
            inventory: Inventory::backpack(),
            hidden: false,
            has_backpack: false,
            inventory_cursor: 0,
            wardrobe: Wardrobe::starter(),
            has_map: false,
            has_city_map: false,
            canteen_fill: 0.0,
            flashlight_on: false,
            flashlight_charge: 100.0,
            pitch: 0.0,
            stride: 0.0,
        }
    }

    /// Tilts the view by `angle` radians, within what the neck allows.
    pub fn pitch_by(&mut self, angle: f64) {
        self.pitch = (self.pitch + angle).clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Snaps the view to an absolute pitch in radians, clamped to what the neck allows.
    pub fn pitch_to(&mut self, angle: f64) {
        self.pitch = angle.clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Toggles the view between level and the given pitch stop: if already at the stop
    /// (within a small tolerance), levels back to zero. Otherwise snaps to the stop.
    pub fn toggle_pitch(&mut self, angle: f64) {
        let target = angle.clamp(-MAX_PITCH, MAX_PITCH);
        self.pitch = if (self.pitch - target).abs() < 0.05 { 0.0 } else { target };
    }

    /// How well what you wear keeps the cold out, 0 (a gown and socks) to about 1.
    pub fn insulation(&self) -> f64 {
        let w = &self.wardrobe;
        let body = match w.body {
            Some(Garment::HeavyJacket) => 0.7,
            Some(Garment::Scrubs) => 0.3,
            Some(Garment::HospitalGown) => 0.12,
            _ => 0.0,
        };
        let legs = if w.wearing_pants() { 0.22 } else { 0.0 };
        let feet = match w.feet {
            Some(Garment::Boots) => 0.18,
            Some(Garment::Sneakers) => 0.08,
            Some(_) => 0.05,
            None => 0.0,
        };
        body + legs + feet
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

    /// Faces a direction (unit vector), keeping the field of view.
    pub fn face(&mut self, dir_x: f64, dir_y: f64) {
        self.dir_x = dir_x;
        self.dir_y = dir_y;
        self.plane_x = -dir_y * FOV_PLANE;
        self.plane_y = dir_x * FOV_PLANE;
    }

    /// Walks for `dt_s` seconds in the direction of `m` (forward/side are -1..1), at
    /// walking, running or crouching pace. Returns how far the player went.
    pub fn walk(&mut self, map: &Map, obstacles: &[Obstacle], m: Motion, dt_s: f64) -> f64 {
        if m.forward == 0.0 && m.side == 0.0 {
            return 0.0;
        }
        let speed = self.speed(m.run);
        let fwd = m.forward * if m.forward < 0.0 { BACK_FACTOR } else { 1.0 };
        let side = m.side * SIDE_FACTOR;
        // to the right of where you face (the floor plan has y pointing south)
        let (rx, ry) = (-self.dir_y, self.dir_x);
        let (mut vx, mut vy) = (self.dir_x * fwd + rx * side, self.dir_y * fwd + ry * side);
        let len = (vx * vx + vy * vy).sqrt();
        if len > 1.0 {
            vx /= len;
            vy /= len;
        }
        let before = (self.x, self.y);
        self.step(map, obstacles, vx * speed * dt_s, vy * speed * dt_s);
        ((self.x - before.0).powi(2) + (self.y - before.1).powi(2)).sqrt()
    }

    /// Metres per second right now. A loaded backpack slows you (more so running), and
    /// so does being out of breath.
    pub fn speed(&self, run: bool) -> f64 {
        let load = self.inventory.weight_ratio();
        if self.hidden {
            return CROUCH_SPEED;
        }
        let base = if run && self.can_run() {
            RUN_SPEED * (if self.has_backpack { 0.95 } else { 1.0 } - 0.25 * load)
        } else {
            WALK_SPEED * (1.0 - 0.15 * load)
        };
        // sneakers are made for it; socks on vinyl aren't
        let shoes = match self.wardrobe.feet {
            Some(Garment::Sneakers) => 1.06,
            Some(Garment::Boots) => 0.97,
            _ => 0.94,
        };
        let base = base * shoes;
        if self.stats.stamina < 15.0 { base * 0.7 } else { base }
    }

    /// Running needs breath left.
    pub fn can_run(&self) -> bool {
        self.stats.stamina > 5.0
    }

    /// Moves one axis at a time so the player slides along walls. A move is refused
    /// only if it pushes further into something: anyone who ends up overlapping a wall
    /// or a sofa (after standing up, or from an old save) can always step back out.
    fn step(&mut self, map: &Map, obstacles: &[Obstacle], dx: f64, dy: f64) {
        slide(map, obstacles, &mut self.x, &mut self.y, dx, dy);
    }

    /// True while any part of the player's body is inside tile (tx, ty).
    pub fn overlaps_tile(&self, tx: usize, ty: usize) -> bool {
        let (x0, y0) = (tx as f64, ty as f64);
        self.x + PLAYER_RADIUS > x0 && self.x - PLAYER_RADIUS < x0 + 1.0
            && self.y + PLAYER_RADIUS > y0 && self.y - PLAYER_RADIUS < y0 + 1.0
    }
}

/// Moves a point (something the size of a person) by (dx, dy), sliding along walls and
/// round furniture, the same way the player moves.
pub fn slide(map: &Map, obstacles: &[Obstacle], x: &mut f64, y: &mut f64, dx: f64, dy: f64) {
    if dx != 0.0 && !blocked_x(map, *x, *y, dx) && !obstructed(obstacles, *x, *y, *x + dx, *y) {
        *x += dx;
    }
    if dy != 0.0 && !blocked_y(map, *x, *y, dy) && !obstructed(obstacles, *x, *y, *x, *y + dy) {
        *y += dy;
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
        let d_new = o.distance(nx, ny);
        d_new < PLAYER_RADIUS && d_new < o.distance(x, y)
    })
}
