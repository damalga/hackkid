use serde::{Deserialize, Serialize};

use crate::game::items::Item;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BodySlot {
    Head,
    Body,
    Hands,
    Legs,
    Feet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wardrobe {
    pub head: Option<String>,
    pub body: Option<String>,
    pub hands: Option<String>,
    pub legs: Option<String>,
    pub feet: Option<String>,
}

impl Wardrobe {
    pub fn starter() -> Self {
        Self {
            head: None,
            body: Some("Hospital Gown".into()),
            hands: None,
            legs: None,
            feet: Some("Socks".into()),
        }
    }

    pub fn wearing_pants(&self) -> bool {
        matches!(&self.legs, Some(l) if l.to_lowercase().contains("trousers") || l.to_lowercase().contains("pantalones"))
    }

    pub fn slot(&self, slot: BodySlot) -> Option<&String> {
        match slot {
            BodySlot::Head => self.head.as_ref(),
            BodySlot::Body => self.body.as_ref(),
            BodySlot::Hands => self.hands.as_ref(),
            BodySlot::Legs => self.legs.as_ref(),
            BodySlot::Feet => self.feet.as_ref(),
        }
    }

    pub fn set(&mut self, slot: BodySlot, label: String) {
        match slot {
            BodySlot::Head => self.head = Some(label),
            BodySlot::Body => self.body = Some(label),
            BodySlot::Hands => self.hands = Some(label),
            BodySlot::Legs => self.legs = Some(label),
            BodySlot::Feet => self.feet = Some(label),
        }
    }

    pub fn display_rows(&self) -> [(&'static str, String); 5] {
        let nothing = || "Nada".to_string();
        let legs_display = match &self.legs {
            Some(l) => {
                let mark = if self.wearing_pants() { " *" } else { "" };
                format!("{}{} + Calzoncillos", l, mark)
            }
            None => "Calzoncillos".into(),
        };
        [
            ("Cabeza:", self.head.clone().unwrap_or_else(nothing)),
            ("Cuerpo:", self.body.clone().unwrap_or_else(nothing)),
            ("Manos:", self.hands.clone().unwrap_or_else(nothing)),
            ("Piernas:", legs_display),
            ("Pies:", self.feet.clone().unwrap_or_else(nothing)),
        ]
    }
}

const MOVE_SPEED: f64 = 0.16;
const SPRINT_SPEED: f64 = 0.28;
const MAX_STAMINA: f64 = 100.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<Item>>,
    pub capacity: usize,
}

impl Inventory {
    pub fn backpack() -> Self {
        Self { slots: vec![None; 8], capacity: 6 }
    }

    pub fn add(&mut self, item: Item) -> bool {
        for slot in self.slots.iter_mut().take(self.capacity) {
            if slot.is_none() {
                *slot = Some(item);
                return true;
            }
        }
        false
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
    #[serde(default = "default_hygiene")]
    pub hygiene: f64,
}

fn default_hygiene() -> f64 { 80.0 }

impl Stats {
    pub fn new() -> Self {
        Self {
            body: 80.0, mind: 70.0, stamina: MAX_STAMINA,
            thirst: 25.0, hunger: 20.0, sleep: 20.0,
            thermal: 0.0, hygiene: 80.0,
        }
    }
}

#[derive(Debug, Clone)]
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

    pub fn move_forward(&mut self, map: &crate::engine::map::Map) {
        let speed = self.effective_speed();
        let nx = self.x + self.dir_x * speed;
        let ny = self.y + self.dir_y * speed;
        if !map.get(nx as usize, self.y as usize).blocks_movement() {
            self.x = nx;
        }
        if !map.get(self.x as usize, ny as usize).blocks_movement() {
            self.y = ny;
        }
    }

    pub fn move_backward(&mut self, map: &crate::engine::map::Map) {
        let speed = self.effective_speed();
        let nx = self.x - self.dir_x * speed;
        let ny = self.y - self.dir_y * speed;
        if !map.get(nx as usize, self.y as usize).blocks_movement() {
            self.x = nx;
        }
        if !map.get(self.x as usize, ny as usize).blocks_movement() {
            self.y = ny;
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
}
