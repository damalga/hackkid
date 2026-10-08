//! Saving and loading, as a JSON string. Where it's stored is the frontend's business.
//!
//! A save holds everything that changes during play: the player, the clock, which doors
//! are open, what has been picked up, eaten or moved. The hospital itself comes from
//! [`crate::game::level`], so a save only loads into the same version of the level.

use serde::{Deserialize, Serialize};

use crate::equippables::Equippable;
use crate::game::items::Item;
use crate::game::player::Player;
use crate::game::world::{DroppedItem, GameMode, Laptop, StaminaState, World, weather_at};

/// Bump when the save layout or the level changes in a way old saves can't follow.
pub const SAVE_VERSION: u32 = 3;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub version: u32,
    pub player: Player,
    pub day: u32,
    pub hour: f64,
    pub mode: GameMode,
    pub rest_return: Option<(f64, f64)>,
    pub doors_open: Vec<(usize, usize, bool)>,
    pub equippables: Vec<Equippable>,
    pub clothing_taken: Vec<bool>,
    pub vending_stock: Vec<u32>,
    pub paper_units: Vec<u32>,
    /// What's left in each container, and whether it has been searched.
    pub containers: Vec<(Vec<Item>, bool)>,
    pub seed: u64,
    pub dropped: Vec<DroppedItem>,
    pub laptop: Laptop,
    pub julian_line: usize,
    pub weather_seen: bool,
    pub stamina_grace_ms: u64,
    pub stamina_state: StaminaState,
    pub sink_drink_cooldown_ms: u64,
}

#[derive(Deserialize)]
struct Header {
    #[serde(default)]
    version: u32,
}

impl World {
    /// Saving is possible while playing, not on the title, while asleep or when dead.
    pub fn can_save(&self) -> bool {
        matches!(
            self.mode,
            GameMode::Exploring | GameMode::Sitting | GameMode::Lying | GameMode::Inventory | GameMode::Laptop | GameMode::Menu
        )
    }

    pub fn save_json(&self) -> String {
        let state = SaveState {
            version: SAVE_VERSION,
            player: self.player.clone(),
            day: self.day,
            hour: self.hour,
            // a save made from the pause menu resumes into the game
            mode: if self.mode == GameMode::Menu { GameMode::Exploring } else { self.mode },
            rest_return: self.rest_return,
            doors_open: self
                .doors
                .iter()
                .map(|d| (d.tx, d.ty, self.map.get(d.tx, d.ty).door_open().unwrap_or(false)))
                .collect(),
            equippables: self.equippables.clone(),
            clothing_taken: self.clothing.iter().map(|c| c.taken).collect(),
            vending_stock: self.vending.iter().map(|v| v.stock).collect(),
            paper_units: self.fixtures.iter().map(|f| f.paper_units).collect(),
            containers: self.containers.iter().map(|c| (c.items.clone(), c.searched)).collect(),
            seed: self.seed,
            dropped: self.dropped.clone(),
            laptop: self.laptop.clone(),
            julian_line: self.julian_line,
            weather_seen: self.weather_seen,
            stamina_grace_ms: self.stamina_grace_ms,
            stamina_state: self.stamina_state,
            sink_drink_cooldown_ms: self.sink_drink_cooldown_ms,
        };
        serde_json::to_string_pretty(&state).expect("save state always serializes")
    }

    /// Replaces the whole world with the saved one. On error nothing changes and the
    /// message says why.
    pub fn load_json(&mut self, json: &str) -> Result<(), String> {
        let header: Header = serde_json::from_str(json).map_err(|e| format!("The save file is damaged ({e})."))?;
        if header.version != SAVE_VERSION {
            return Err("That save is from another version of the game and can't be loaded.".into());
        }
        let s: SaveState = serde_json::from_str(json).map_err(|e| format!("The save file is damaged ({e})."))?;

        let mut w = World::with_seed(s.seed);
        if s.clothing_taken.len() != w.clothing.len()
            || s.vending_stock.len() != w.vending.len()
            || s.paper_units.len() != w.fixtures.len()
            || s.containers.len() != w.containers.len()
        {
            return Err("That save doesn't match this version of the hospital.".into());
        }
        w.now_ms = self.now_ms;
        w.player = s.player;
        w.day = s.day;
        w.hour = s.hour.rem_euclid(24.0);
        w.weather = weather_at(w.day, w.hour);
        w.mode = match s.mode {
            GameMode::Sitting | GameMode::Lying if s.rest_return.is_some() => s.mode,
            GameMode::Inventory if w.player.has_backpack => GameMode::Inventory,
            GameMode::Laptop => GameMode::Laptop,
            _ => GameMode::Exploring,
        };
        w.rest_return = if matches!(w.mode, GameMode::Sitting | GameMode::Lying) { s.rest_return } else { None };
        for (tx, ty, open) in s.doors_open {
            let tile = w.map.get(tx, ty);
            if tile.is_any_door() {
                w.map.set(tx, ty, tile.with_open(open));
            }
            // self-closing doors that were open start their countdown again
            if let Some(d) = w.doors.iter_mut().find(|d| d.tx == tx && d.ty == ty) {
                d.open_since_ms = open.then_some(w.now_ms);
            }
        }
        w.equippables = s.equippables;
        for (c, taken) in w.clothing.iter_mut().zip(s.clothing_taken) {
            c.taken = taken;
        }
        for (v, stock) in w.vending.iter_mut().zip(s.vending_stock) {
            v.stock = stock;
        }
        for (f, units) in w.fixtures.iter_mut().zip(s.paper_units) {
            f.paper_units = units;
        }
        for (c, (items, searched)) in w.containers.iter_mut().zip(s.containers) {
            c.items = items;
            c.searched = searched;
        }
        w.dropped = s.dropped;
        w.laptop = s.laptop;
        w.laptop.cursor = w.laptop.cursor.min(crate::game::tracks::TRACKS.len() - 1);
        w.julian_line = s.julian_line;
        w.weather_seen = s.weather_seen;
        w.stamina_grace_ms = s.stamina_grace_ms;
        w.stamina_state = s.stamina_state;
        w.sink_drink_cooldown_ms = s.sink_drink_cooldown_ms;
        *self = w;
        Ok(())
    }
}
