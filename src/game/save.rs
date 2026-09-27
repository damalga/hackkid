use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

use crate::engine::map::Tile;
use crate::game::items::Item;
use crate::game::world::{DroppedItem, GameMode, World};

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    #[serde(default)]
    pub start_date: String,
    pub player_x: f64,
    pub player_y: f64,
    pub player_dir_x: f64,
    pub player_dir_y: f64,
    pub player_plane_x: f64,
    pub player_plane_y: f64,
    pub turn: u64,
    pub day: u32,
    pub hour: f64,
    pub samuel_stage: u8,
    pub samuel_line: usize,
    #[serde(default)]
    pub selenia_line: usize,
    #[serde(default)]
    pub selenia_met: bool,
    pub has_map: bool,
    #[serde(default)]
    pub has_city_map: bool,
    pub has_backpack: bool,
    pub hidden: bool,
    pub wardrobe: crate::game::player::Wardrobe,
    pub inventory: Vec<Option<Item>>,
    pub dropped: Vec<DroppedItem>,
    pub doors_open: Vec<(usize, usize, bool)>,
    pub equippables_taken: Vec<bool>,
    pub clothing_taken: Vec<bool>,
    pub laptop_battery: f64,
    pub laptop_cursor: usize,
    pub mode: GameMode,
    pub stats: crate::game::player::Stats,
    #[serde(default)]
    pub canteen_fill: f64,
}

impl World {
    pub fn save_to_disk(&mut self, _path: &str) -> io::Result<()> {
        let state = self.build_save_state();
        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let path = format!("save-{}.json", self.start_date);
        fs::write(&path, json)?;
        self.set_message(format!("Partida guardada como {}", path), 12);
        Ok(())
    }

    pub fn load_latest_save(&mut self) -> io::Result<()> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let path = format!("save-{}.json", today);
        self.load_from_disk(&path)
    }

    pub fn load_from_disk(&mut self, path: &str) -> io::Result<()> {
        let raw = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                self.set_message(format!("No hay partida guardada ({})", e), 12);
                return Err(e);
            }
        };
        let state: SaveState = match serde_json::from_str(&raw) {
            Ok(s) => s,
            Err(e) => {
                self.set_message(format!("Save corrupto: {}", e), 14);
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
        };
        self.apply_save_state(state);
        self.set_message("Partida cargada.".into(), 10);
        Ok(())
    }

    pub(crate) fn build_save_state(&self) -> SaveState {
        let doors_open: Vec<(usize, usize, bool)> = self.doors.iter().map(|d| {
            let open = match self.map.get(d.tx, d.ty) {
                Tile::Door { open } => open,
                Tile::MainDoor { open } => open,
                Tile::BathroomDoor { open } => open,
                Tile::OperatingDoor { open } => open,
                _ => false,
            };
            (d.tx, d.ty, open)
        }).collect();
        SaveState {
            start_date: self.start_date.clone(),
            player_x: self.player.x,
            player_y: self.player.y,
            player_dir_x: self.player.dir_x,
            player_dir_y: self.player.dir_y,
            player_plane_x: self.player.plane_x,
            player_plane_y: self.player.plane_y,
            turn: self.turn,
            day: self.day,
            hour: self.hour,
            samuel_stage: self.samuel_stage,
            samuel_line: self.samuel_line,
            selenia_line: self.selenia_line,
            selenia_met: self.selenia_met,
            has_map: self.player.has_map,
            has_city_map: self.player.has_city_map,
            has_backpack: self.player.has_backpack,
            hidden: self.player.hidden,
            wardrobe: self.player.wardrobe.clone(),
            inventory: self.player.inventory.slots.clone(),
            dropped: self.dropped.clone(),
            doors_open,
            equippables_taken: self.equippables.iter().map(|e| e.taken()).collect(),
            clothing_taken: self.clothing.iter().map(|c| c.taken).collect(),
            laptop_battery: self.laptop_battery,
            laptop_cursor: self.laptop_cursor,
            mode: self.mode,
            stats: self.player.stats.clone(),
            canteen_fill: self.player.canteen_fill,
        }
    }

    pub(crate) fn apply_save_state(&mut self, state: SaveState) {
        self.player.x = state.player_x;
        self.player.y = state.player_y;
        self.player.dir_x = state.player_dir_x;
        self.player.dir_y = state.player_dir_y;
        self.player.plane_x = state.player_plane_x;
        self.player.plane_y = state.player_plane_y;
        self.turn = state.turn;
        self.day = state.day;
        self.hour = state.hour;
        self.samuel_stage = state.samuel_stage;
        self.samuel_line = state.samuel_line;
        self.selenia_line = state.selenia_line;
        self.selenia_met = state.selenia_met;
        self.player.has_map = state.has_map;
        self.player.has_city_map = state.has_city_map;
        self.player.has_backpack = state.has_backpack;
        self.player.hidden = state.hidden;
        self.player.wardrobe = state.wardrobe;
        self.player.inventory.slots = state.inventory;
        self.dropped = state.dropped;
        self.laptop_battery = state.laptop_battery;
        self.laptop_cursor = state.laptop_cursor;
        self.mode = state.mode;
        self.player.stats = state.stats;
        self.player.canteen_fill = state.canteen_fill;
        if !state.start_date.is_empty() {
            self.start_date = state.start_date;
        }

        for (tx, ty, open) in state.doors_open {
            let tile = self.map.get(tx, ty);
            let new_tile = match tile {
                Tile::Door { .. } => Tile::Door { open },
                Tile::MainDoor { .. } => Tile::MainDoor { open },
                Tile::BathroomDoor { .. } => Tile::BathroomDoor { open },
                Tile::OperatingDoor { .. } => Tile::OperatingDoor { open },
                other => other,
            };
            self.map.set(tx, ty, new_tile);
        }
        for (i, taken) in state.equippables_taken.iter().enumerate() {
            if let Some(e) = self.equippables.get_mut(i) {
                if *taken { e.mark_taken(); }
            }
        }
        for (i, taken) in state.clothing_taken.iter().enumerate() {
            if let Some(c) = self.clothing.get_mut(i) {
                c.taken = *taken;
            }
        }
        self.message = None;
        self.message_ttl = 0;
        self.pending.clear();
    }
}
