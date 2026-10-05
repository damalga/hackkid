use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

use crate::engine::map::{Map, Tile};
use crate::game::items::{Item, ItemKind};
use crate::game::player::Player;
use crate::objects::{
    Bed, BedKind, Bench, Burra, Clothing, ClothingKind, CoatRack, Decor, DecorKind, Door, Fixture, FixtureKind, Fluorescent, FluorescentAxis, FluorescentState, Npc, Outlet, Reception, Sign, SignKind, Sofa, VendingKind, VendingMachine, Window as WinObj, private_bathroom, public_bathroom,
};
use crate::equippables::{Backpack, Equippable};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    Startup,
    InitialWake,
    Exploring,
    Sitting,
    Lying,
    Sleeping,
    Inventory,
    Laptop,
    Menu,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StaminaState {
    Idle,
    Draining,
    Recovering,
    Optimal,
}

pub const REAL_MS_PER_GAME_HOUR: f64 = 120_000.0; // 2 real sec = 1 game min → 2 min = 1 game hour ... actually 2 s = 1 min → 120 s = 1 h → 120000 ms
pub const STAMINA_GRACE_MS: u64 = 120_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Cloudy,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageStyle {
    Protagonist,
    Other,
    Neutral,
}

impl GameMode {
    pub fn shows_ceiling(&self) -> bool {
        matches!(self, GameMode::InitialWake | GameMode::Lying)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedItem {
    pub x: f64,
    pub y: f64,
    pub item: Item,
}

pub struct World {
    pub map: Map,
    pub player: Player,
    pub turn: u64,
    pub day: u32,
    pub hour: f64,
    pub mode: GameMode,
    pub doors: Vec<Door>,
    pub fluorescents: Vec<Fluorescent>,
    pub equippables: Vec<Equippable>,
    pub clothing: Vec<Clothing>,
    pub beds: Vec<Bed>,
    pub coat_racks: Vec<CoatRack>,
    pub benches: Vec<Bench>,
    pub sofas: Vec<Sofa>,
    pub outlets: Vec<Outlet>,
    pub fixtures: Vec<Fixture>,
    pub vending: Vec<VendingMachine>,
    pub receptions: Vec<Reception>,
    pub windows: Vec<WinObj>,
    pub burras: Vec<Burra>,
    pub decors: Vec<Decor>,
    pub npcs: Vec<Npc>,
    pub signs: Vec<Sign>,
    pub pending: Vec<(u64, String, u32)>,
    pub dropped: Vec<DroppedItem>,
    pub laptop_cursor: usize,
    pub laptop_battery: f64,
    pub laptop_playing_track: Option<usize>,
    pub samuel_stage: u8,
    pub samuel_line: usize,
    pub selenia_line: usize,
    pub selenia_met: bool,
    pub message: Option<String>,
    pub message_ttl: u32,
    pub message_expiry_ms: u64,
    pub message_style: MessageStyle,
    pub last_ms: u64,
    pub stamina_grace_ms: u64,
    pub stamina_state: StaminaState,
    pub walked: bool,
    pub sleep_end_ms: u64,
    pub sleep_pending_hours: u32,
    pub weather: Weather,
    pub weather_seen: bool,
    pub menu_cursor: usize,
    pub should_exit: bool,
    pub start_date: String,
    pub sink_drink_cooldown_ms: u64,
}

impl World {
    pub fn new() -> Self {
        let map = Map::hospital_ward();
        let player = Player::new(13.5, 4.5);

        let doors = vec![
            Door::new(13, 6),
            Door::new(19, 6),
            Door::new(25, 6),
            Door::new(31, 6),
            Door::bathroom(14, 3),
            Door::bathroom(20, 3),
            Door::bathroom(26, 3),
            Door::bathroom(32, 3),
            Door::new(36, 16),
            Door::new(41, 16),
            Door::main(22, 12),
            Door::bathroom(10, 19),
            Door::main(34, 19),
            Door::main(22, 23),
            Door::main(22, 44),
            Door::street(22, 59),
        ];

        let fluorescents = vec![
            Fluorescent::new(13.5, 4.0, 0.35, FluorescentAxis::Horizontal),
            Fluorescent::new(19.5, 4.0, 0.35, FluorescentAxis::Horizontal)
                .with_state(FluorescentState::Flicker { period_ms: 1400, duty: 0.72 }),
            Fluorescent::new(25.5, 4.0, 0.35, FluorescentAxis::Horizontal),
            Fluorescent::new(31.5, 4.0, 0.35, FluorescentAxis::Horizontal)
                .with_state(FluorescentState::Dead),
            Fluorescent::new(17.0, 9.0, 0.55, FluorescentAxis::Horizontal),
            Fluorescent::new(22.0, 9.0, 0.55, FluorescentAxis::Horizontal)
                .with_state(FluorescentState::Flicker { period_ms: 900, duty: 0.55 }),
            Fluorescent::new(27.0, 9.0, 0.55, FluorescentAxis::Horizontal),

            Fluorescent::new(14.0, 14.0, 0.60, FluorescentAxis::Horizontal),
            Fluorescent::new(22.0, 14.0, 0.60, FluorescentAxis::Horizontal),
            Fluorescent::new(30.0, 14.0, 0.60, FluorescentAxis::Horizontal)
                .with_state(FluorescentState::Flicker { period_ms: 700, duty: 0.60 }),
            Fluorescent::new(14.0, 18.0, 0.60, FluorescentAxis::Horizontal)
                .with_state(FluorescentState::Flicker { period_ms: 1600, duty: 0.80 }),
            Fluorescent::new(22.0, 18.0, 0.60, FluorescentAxis::Horizontal),
            Fluorescent::new(30.0, 18.0, 0.60, FluorescentAxis::Horizontal),
            Fluorescent::new(22.0, 21.5, 0.60, FluorescentAxis::Horizontal),
            Fluorescent::new(36.5, 14.0, 0.35, FluorescentAxis::Horizontal),
            Fluorescent::new(41.5, 14.0, 0.35, FluorescentAxis::Horizontal),
            Fluorescent::new(39.0, 19.5, 0.55, FluorescentAxis::Horizontal),
            Fluorescent::new(22.0, 30.0, 0.6, FluorescentAxis::Horizontal),
            Fluorescent::new(22.0, 40.0, 0.6, FluorescentAxis::Horizontal),
            Fluorescent::new(15.0, 46.5, 0.55, FluorescentAxis::Horizontal),
            Fluorescent::new(22.5, 46.5, 0.55, FluorescentAxis::Horizontal),
            Fluorescent::new(29.0, 46.5, 0.55, FluorescentAxis::Horizontal),
            Fluorescent::new(15.0, 56.5, 0.55, FluorescentAxis::Horizontal)
                .with_state(FluorescentState::Flicker { period_ms: 1300, duty: 0.70 }),
            Fluorescent::new(22.5, 56.5, 0.55, FluorescentAxis::Horizontal),
            Fluorescent::new(29.0, 56.5, 0.55, FluorescentAxis::Horizontal),
        ];

        let equippables = vec![
            Equippable::Backpack(Backpack::player_starter(14.5, 5.5)),
        ];

        let clothing = vec![
            Clothing::new(19.5, 4.7, ClothingKind::HospitalGown),
            Clothing::new(21.5, 10.5, ClothingKind::ScrubBundle),
            Clothing::new(22.5, 10.7, ClothingKind::HospitalGown),
            Clothing::new(14.0, 16.0, ClothingKind::HospitalGown),
            Clothing::new(19.5, 15.5, ClothingKind::ScrubBundle),
            Clothing::new(26.0, 17.5, ClothingKind::HospitalGown),
            Clothing::new(17.5, 19.5, ClothingKind::ScrubBundle),
            Clothing::new(28.5, 20.5, ClothingKind::HospitalGown),
            Clothing::new(30.5, 16.5, ClothingKind::HospitalGown),
            Clothing::new(15.5, 21.5, ClothingKind::ScrubBundle),
            Clothing::new(24.5, 21.8, ClothingKind::HospitalGown),
            Clothing::new(11.5, 3.55, ClothingKind::Hoodie).with_elevation(0.32),
            Clothing::new(11.2, 14.5, ClothingKind::Pants).with_elevation(0.32),
            // Outdoor ground clothing (elevation 0 = not pickable, just description)
            Clothing::new(12.0, 68.0, ClothingKind::HospitalGown),
            Clothing::new(20.5, 70.5, ClothingKind::ScrubBundle),
            Clothing::new(28.0, 66.5, ClothingKind::HospitalGown),
            Clothing::new(34.5, 74.5, ClothingKind::ScrubBundle),
            Clothing::new(48.0, 27.5, ClothingKind::HospitalGown),
            Clothing::new(51.0, 40.0, ClothingKind::ScrubBundle),
            Clothing::new(53.5, 62.5, ClothingKind::HospitalGown),
            Clothing::new(46.5, 71.5, ClothingKind::ScrubBundle),
        ];

        let beds = vec![
            Bed::new(11.5, 5.5),
            Bed::new(17.5, 5.5),
            Bed::new(23.5, 5.5),
            Bed::new(29.5, 5.5),
            Bed::operating(36.5, 14.0),
        ];

        let coat_racks = vec![
            CoatRack::new(11.5, 3.5),
            CoatRack::new(17.5, 3.5),
            CoatRack::new(23.5, 3.5),
            CoatRack::new(29.5, 3.5),
        ];

        let benches = vec![
            Bench::new(16.0, 12.7),
            Bench::new(11.15, 14.5),
            Bench::new(32.85, 14.5),
            Bench::new(28.0, 22.3),
        ];

        let sofas = vec![
            Sofa::new(14.5, 5.55),
            Sofa::new(20.5, 5.55),
            Sofa::new(26.5, 5.55),
            Sofa::new(32.5, 5.55),
            Sofa::new(43.15, 14.5),
        ];

        let mut fixtures: Vec<Fixture> = Vec::new();
        fixtures.extend(private_bathroom(13.5, 1.5));
        fixtures.extend(private_bathroom(19.5, 1.5));
        fixtures.extend(private_bathroom(25.5, 1.5));
        fixtures.extend(private_bathroom(31.5, 1.5));
        fixtures.extend(public_bathroom(2.5, 18.3));
        fixtures.extend(public_bathroom(6.5, 18.3));

        let outlets = vec![
            // Origin rooms: outlet next to bed on south wall
            Outlet::new(11.5, 5.98),
            Outlet::new(17.5, 5.98),
            Outlet::new(23.5, 5.98),
            Outlet::new(29.5, 5.98),
            // Origin rooms: outlet next to sofa on south wall
            Outlet::new(14.5, 5.98),
            Outlet::new(20.5, 5.98),
            Outlet::new(26.5, 5.98),
            Outlet::new(32.5, 5.98),
            // Operating room outlet on top wall next to bed head
            Outlet::new(36.5, 13.02),
            // Waiting room: outlet on east wall next to sofa
            Outlet::new(43.98, 14.5),
            // Hub benches
            Outlet::new(16.8, 13.02),
            Outlet::new(11.02, 14.5),
            Outlet::new(33.98, 14.5),
            Outlet::new(28.7, 22.98),
        ];

        let signs = vec![
            Sign::new(14.3, 7.02, SignKind::RoomSmall),
            Sign::new(20.3, 7.02, SignKind::RoomSmall),
            Sign::new(26.3, 7.02, SignKind::RoomSmall),
            Sign::new(32.3, 7.02, SignKind::RoomSmall),
            Sign::new(37.3, 17.02, SignKind::RoomSmall),
            Sign::new(42.3, 17.02, SignKind::RoomSmall),
            Sign::new(23.3, 13.02, SignKind::MainSquare),
            Sign::new(11.02, 20.3, SignKind::Bathroom),
            Sign::new(33.98, 20.3, SignKind::MainSquare),
            Sign::new(23.3, 22.98, SignKind::HallwayLong),
            // Big HOSPITAL plaque outside street door
            Sign::new(24.5, 59.15, SignKind::HospitalPlaque),
        ];

        let npcs = vec![
            Npc::julian(17.5, 17.5),
        ];

        let vending = vec![
            VendingMachine::new(15.5, 58.85, VendingKind::Snacks, 3),
            VendingMachine::new(17.2, 58.85, VendingKind::Drinks, 2),
        ];

        let receptions = vec![
            Reception::new(27.5, 56.6),
        ];

        let windows = vec![
            WinObj::new(11.02, 50.0),
            WinObj::new(33.98, 50.0),
        ];

        let burras = vec![
            // Operating room: burra against east wall (side)
            Burra::new(37.85, 15.4),
        ];

        let mut decors = vec![
            // Operating room aluminum tables against north wall
            Decor::new(35.5, 13.35, DecorKind::AluminumTable),
            Decor::new(37.5, 13.35, DecorKind::AluminumTable),
            // Surgical cabinet on west wall (elevated - wall-mounted)
            Decor::new(35.15, 14.5, DecorKind::SurgicalCabinet).with_elevation(0.35),
            // Waiting room wooden table
            Decor::new(40.7, 13.7, DecorKind::WoodenTable),

            // Bus stop and mailbox on the sidewalk at y=63.5
            Decor::new(18.5, 63.4, DecorKind::BusStop),
            Decor::new(25.0, 63.4, DecorKind::Mailbox),
            Decor::new(46.5, 40.0, DecorKind::Mailbox),

            // South frontage trees + bushes (garden between hospital wall y=59 and sidewalk y=60)
            Decor::new(6.0, 62.0, DecorKind::TreeOak),
            Decor::new(13.5, 68.0, DecorKind::TreePine),
            Decor::new(30.0, 68.0, DecorKind::TreeOak),
            Decor::new(38.5, 68.5, DecorKind::TreePine),
            Decor::new(43.5, 68.0, DecorKind::TreeOak),
            Decor::new(3.5, 68.5, DecorKind::TreeOak),
            Decor::new(3.5, 75.0, DecorKind::TreePine),

            // South garden bushes (below parking lot y=65..66)
            Decor::new(7.5, 72.0, DecorKind::Bush),
            Decor::new(11.5, 72.5, DecorKind::Bush),
            Decor::new(17.5, 73.0, DecorKind::Bush),
            Decor::new(21.5, 72.0, DecorKind::Bush),
            Decor::new(26.5, 74.0, DecorKind::TreeOak),
            Decor::new(32.5, 72.5, DecorKind::Bush),
            Decor::new(36.5, 74.0, DecorKind::Bush),
            Decor::new(41.5, 72.5, DecorKind::Bush),
            Decor::new(45.5, 74.0, DecorKind::Bush),
            Decor::new(49.5, 72.5, DecorKind::TreePine),
            Decor::new(53.5, 71.5, DecorKind::Bush),
            Decor::new(56.5, 73.5, DecorKind::TreeOak),

            // East side trees, bushes (garden between hospital wall x=45 and sidewalk x=46)
            Decor::new(4.5, 30.0, DecorKind::TreeOak),
            Decor::new(3.5, 42.0, DecorKind::TreePine),
            Decor::new(4.5, 55.0, DecorKind::TreeOak),
            Decor::new(50.0, 8.0, DecorKind::TreeOak),
            Decor::new(53.5, 5.5, DecorKind::TreePine),
            Decor::new(54.5, 33.0, DecorKind::TreePine),
            Decor::new(48.5, 45.0, DecorKind::Bush),
            Decor::new(54.5, 45.0, DecorKind::TreeOak),
            Decor::new(56.5, 55.0, DecorKind::TreeOak),
            Decor::new(54.5, 62.0, DecorKind::TreePine),
            Decor::new(58.0, 40.0, DecorKind::Bush),
            Decor::new(58.0, 25.0, DecorKind::Bush),
        ];

        // Scatter weeds across garden tiles
        let weed_positions: &[(f64, f64)] = &[
            (2.5, 3.5), (5.5, 6.5), (7.5, 10.5), (2.5, 15.5), (5.5, 22.5),
            (2.5, 30.5), (4.5, 36.5), (2.5, 44.5), (5.5, 50.5), (2.5, 58.5),
            (6.5, 61.5), (16.5, 61.5), (28.5, 61.5), (40.5, 61.5), (5.5, 68.5),
            (9.5, 68.5), (15.5, 71.5), (19.5, 74.5), (24.5, 68.5), (34.5, 71.5),
            (44.5, 68.5), (52.5, 40.5), (58.5, 15.5), (56.5, 3.5), (58.5, 55.5),
            (52.5, 72.5), (48.5, 74.5), (7.5, 76.5), (14.5, 78.5), (22.5, 78.5),
            (30.5, 78.5), (38.5, 78.5), (46.5, 78.5), (55.5, 78.5),
        ];
        for (wx, wy) in weed_positions {
            decors.push(Decor::new(*wx, *wy, DecorKind::Weed));
        }

        // Cars in south parking lot (rows y=65..66, stalls 2 tiles wide centered at x=8.5,10.5,...)
        let car_colors: &[(u8, u8, u8)] = &[
            (185, 40, 40),   // red
            (30, 90, 190),   // blue
            (24, 24, 28),    // black
            (225, 225, 228), // white
            (200, 180, 60),  // yellow
            (40, 100, 55),   // green
            (110, 60, 30),   // brown
            (140, 145, 155), // silver
        ];
        let south_stall_xs = [8.5_f64, 10.5, 12.5, 14.5, 16.5, 22.5, 26.5, 28.5, 32.5, 34.5, 40.5, 44.5];
        for (i, sx) in south_stall_xs.iter().enumerate() {
            let color = car_colors[i % car_colors.len()];
            decors.push(Decor::new(*sx, 65.6, DecorKind::Car { body: color }));
        }
        // East parking column: cars stacked vertically (stalls 2 tiles tall each)
        let east_stall_ys = [7.5_f64, 9.5, 12.5, 14.5, 17.5, 19.5, 22.5, 25.5, 28.5];
        for (i, sy) in east_stall_ys.iter().enumerate() {
            let color = car_colors[(i + 3) % car_colors.len()];
            decors.push(Decor::new(51.6, *sy, DecorKind::Car { body: color }));
        }
        // A couple of cars parked at the road shoulder
        decors.push(Decor::new(20.5, 62.4, DecorKind::Car { body: (30, 90, 190) }));
        decors.push(Decor::new(36.5, 62.4, DecorKind::Car { body: (24, 24, 28) }));

        let door_centers: Vec<(f64, f64)> = doors.iter()
            .map(|d| (d.tx as f64 + 0.5, d.ty as f64 + 0.5))
            .collect();
        let mut signs = signs;
        let mut outlets = outlets;
        let mut coat_racks = coat_racks;
        let mut benches = benches;
        let mut sofas = sofas;
        let mut beds = beds;
        let mut fixtures = fixtures;
        let mut clothing = clothing;
        let mut burras = burras;
        let mut vending = vending;
        let mut receptions = receptions;
        let mut windows = windows;
        let mut decors = decors;
        for s in &mut signs { push_away_from_doors(&mut s.x, &mut s.y, &door_centers, 1.6); }
        for o in &mut outlets { push_away_from_doors(&mut o.x, &mut o.y, &door_centers, 1.3); }
        for c in &mut coat_racks { push_away_from_doors(&mut c.x, &mut c.y, &door_centers, 1.4); }
        for b in &mut benches { push_away_from_doors(&mut b.x, &mut b.y, &door_centers, 1.6); }
        for s in &mut sofas { push_away_from_doors(&mut s.x, &mut s.y, &door_centers, 1.2); }
        for b in &mut beds { push_away_from_doors(&mut b.x, &mut b.y, &door_centers, 1.8); }
        for f in &mut fixtures { push_away_from_doors(&mut f.x, &mut f.y, &door_centers, 1.3); }
        for c in &mut clothing { push_away_from_doors(&mut c.x, &mut c.y, &door_centers, 1.4); }
        for b in &mut burras { push_away_from_doors(&mut b.x, &mut b.y, &door_centers, 1.6); }
        for v in &mut vending { push_away_from_doors(&mut v.x, &mut v.y, &door_centers, 1.3); }
        for r in &mut receptions { push_away_from_doors(&mut r.x, &mut r.y, &door_centers, 1.5); }
        for w in &mut windows { push_away_from_doors(&mut w.x, &mut w.y, &door_centers, 1.2); }
        for d in &mut decors { push_away_from_doors(&mut d.x, &mut d.y, &door_centers, 1.6); }

        Self {
            map, player,
            turn: 0, day: 1, hour: 8.0,
            mode: GameMode::Startup,
            doors,
            fluorescents,
            equippables,
            clothing,
            beds,
            coat_racks,
            benches,
            sofas,
            outlets,
            fixtures,
            vending,
            receptions,
            windows,
            burras,
            decors,
            npcs,
            signs,
            pending: Vec::new(),
            dropped: Vec::new(),
            laptop_cursor: 0,
            laptop_battery: 62.0,
            laptop_playing_track: None,
            samuel_stage: 0,
            samuel_line: 0,
            selenia_line: 0,
            selenia_met: false,
            message: None,
            message_ttl: 0,
            message_expiry_ms: 0,
            message_style: MessageStyle::Neutral,
            last_ms: 0,
            stamina_grace_ms: STAMINA_GRACE_MS,
            stamina_state: StaminaState::Optimal,
            walked: false,
            sleep_end_ms: 0,
            sleep_pending_hours: 0,
            weather: Weather::Clear,
            weather_seen: false,
            menu_cursor: 0,
            should_exit: false,
            start_date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            sink_drink_cooldown_ms: 0,
        }
    }

    pub fn set_message(&mut self, msg: String, ttl: u32) {
        self.message_style = detect_message_style(&msg);
        self.message = Some(msg);
        self.message_ttl = ttl;
        self.message_expiry_ms = self.last_ms + (ttl as u64) * 400;
    }

    pub fn decay_message(&mut self) {
        if self.message_ttl > 0 {
            self.message_ttl -= 1;
        }
        if self.message_ttl == 0 {
            self.message = None;
        }
    }

    pub fn tick(&mut self) {
        self.turn += 1;
    }

    pub fn advance_realtime(&mut self, now_ms: u64) {
        if matches!(self.mode, GameMode::Startup | GameMode::InitialWake | GameMode::Menu | GameMode::Inventory | GameMode::Dead) {
            self.last_ms = now_ms;
            return;
        }
        if self.mode == GameMode::Sleeping {
            self.last_ms = now_ms;
            if now_ms >= self.sleep_end_ms {
                self.finish_sleep();
            }
            return;
        }
        let dt_ms = now_ms.saturating_sub(self.last_ms);
        self.last_ms = now_ms;
        if dt_ms == 0 { return; }
        let dt_game_h = (dt_ms as f64) / REAL_MS_PER_GAME_HOUR;
        let dt_real_s = (dt_ms as f64) / 1000.0;

        self.hour += dt_game_h;
        while self.hour >= 24.0 {
            self.hour -= 24.0;
            self.day += 1;
        }

        let stamina_drains = self.stamina_grace_ms == 0
            && self.mode == GameMode::Exploring;
        let hygiene_before;
        {
            let s = &mut self.player.stats;
            s.body = (s.body - 0.20 * dt_game_h).max(0.0);
            s.mind = (s.mind - 0.20 * dt_game_h).max(0.0);
            s.sleep = (s.sleep + 1.0 * dt_game_h).min(100.0);
            s.thirst = (s.thirst + 5.0 * dt_game_h).min(100.0);
            s.hunger = (s.hunger + 3.0 * dt_game_h).min(100.0);
            hygiene_before = s.hygiene;
            s.hygiene = s.hygiene - 1.0 * dt_game_h;
            if stamina_drains {
                s.stamina = (s.stamina - 3.0 * dt_game_h).max(0.0);
            }
            // Extra drain from critical needs
            if s.thirst >= 90.0 {
                s.body = (s.body - 1.0 * dt_game_h).max(0.0);
            }
            if s.hunger >= 90.0 {
                s.body = (s.body - 0.5 * dt_game_h).max(0.0);
            }
            if s.sleep >= 90.0 {
                s.mind = (s.mind - 1.0 * dt_game_h).max(0.0);
            }
            s.thermal *= (0.995_f64).powf(dt_ms as f64 / 400.0);
        }
        self.sink_drink_cooldown_ms = self.sink_drink_cooldown_ms.saturating_sub(dt_ms);
        // Auto-consume paper roll when hygiene would go negative
        if self.player.stats.hygiene < 0.0 {
            if self.consume_paper_unit() {
                self.player.stats.hygiene = 25.0;
                self.set_message("You use paper automatically to wash.".into(), 12);
            } else {
                self.player.stats.hygiene = 0.0;
            }
        }
        let _ = hygiene_before;

        // Death check
        if self.player.stats.body <= 0.0 || self.player.stats.mind <= 0.0 {
            self.mode = GameMode::Dead;
            self.set_message("Has muerto.".into(), 30);
        }

        // Paper rolls are finite - no refill.

        // Rest regen when Sitting/Lying (sleep only fixed by proper sleep)
        if matches!(self.mode, GameMode::Sitting | GameMode::Lying) {
            let s = &mut self.player.stats;
            let (dbody, dmind) = if self.mode == GameMode::Lying {
                (0.12, 0.12)
            } else {
                (0.05, 0.05)
            };
            s.body = (s.body + dbody * dt_real_s).min(100.0);
            s.mind = (s.mind + dmind * dt_real_s).min(100.0);
        }

        self.update_stamina(dt_ms, dt_real_s);
        self.sync_inventory_capacity();
        if self.map.get(self.player.x as usize, self.player.y as usize).is_outdoor() {
            self.weather_seen = true;
        }
        if self.message.is_some() && self.last_ms >= self.message_expiry_ms {
            self.message = None;
            self.message_ttl = 0;
        }
        self.auto_close_doors();
    }

    fn update_stamina(&mut self, dt_ms: u64, dt_real_s: f64) {
        let prev = self.player.stats.stamina;
        let walked = self.walked;
        self.walked = false;

        if self.stamina_grace_ms > 0 {
            self.player.stats.stamina = 100.0;
            self.stamina_grace_ms = self.stamina_grace_ms.saturating_sub(dt_ms);
            self.stamina_state = StaminaState::Optimal;
            if self.stamina_grace_ms == 0 {
                self.stamina_state = StaminaState::Idle;
            }
            return;
        }

        let regen = match self.mode {
            GameMode::Lying => 3.5,
            GameMode::Sitting => 1.5,
            GameMode::Exploring if !walked => 0.3,
            _ => 0.0,
        };
        let new_val = (self.player.stats.stamina + regen * dt_real_s).min(100.0);
        if new_val > prev + 0.01 {
            self.player.stats.stamina = new_val;
            self.stamina_state = StaminaState::Recovering;
        } else if new_val < prev {
            self.player.stats.stamina = new_val;
        } else {
            self.player.stats.stamina = new_val;
            if self.stamina_state != StaminaState::Draining {
                self.stamina_state = StaminaState::Idle;
            }
        }

        if self.player.stats.stamina >= 100.0 {
            self.stamina_state = StaminaState::Optimal;
        }
    }

    pub fn player_move_forward(&mut self, n: u8) {
        let want_sprint = n >= 2;
        let can_sprint = self.player.stats.stamina > 0.0;
        if want_sprint && can_sprint {
            self.player.sprinting = true;
            for _ in 0..n { self.player.move_forward(&self.map); }
            self.player.sprinting = false;
            if self.stamina_grace_ms == 0 {
                let weight = self.player.inventory.weight_ratio();
                let cost = 0.35 * n as f64 * (1.0 + weight);
                self.player.stats.stamina = (self.player.stats.stamina - cost).max(0.0);
                self.stamina_state = StaminaState::Draining;
            }
        } else {
            self.player.move_forward(&self.map);
        }
        self.walked = true;
        self.decay_message();
    }

    pub fn player_move_backward(&mut self, _n: u8) {
        // No sprinting backward — walk only.
        self.player.move_backward(&self.map);
        self.walked = true;
        self.decay_message();
    }

    pub fn laptop_idle_tick(&mut self) {
        let near_outlet = self.outlets.iter().any(|o| {
            let dx = o.x - self.player.x;
            let dy = o.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.5
        });
        if near_outlet {
            self.laptop_battery = (self.laptop_battery + 0.10).min(100.0);
        } else {
            self.laptop_battery = (self.laptop_battery - 0.05).max(0.0);
            if self.laptop_battery <= 0.0 {
                self.mode = GameMode::Exploring;
                self.set_message("The emergency terminal powers down.".into(), 10);
            }
        }
    }

    fn sleep_hours_by_debt(sleep_debt: f64) -> u32 {
        if sleep_debt < 10.0 { 1 }
        else if sleep_debt < 20.0 { 2 }
        else if sleep_debt < 30.0 { 3 }
        else if sleep_debt < 40.0 { 4 }
        else if sleep_debt < 50.0 { 5 }
        else if sleep_debt < 70.0 { 6 }
        else if sleep_debt < 90.0 { 7 }
        else { 8 }
    }

    pub fn sleep(&mut self) {
        if !matches!(self.mode, GameMode::Lying) { return; }
        self.begin_sleep();
    }

    pub fn sleep_at_nearby_bed(&mut self) -> bool {
        if self.mode != GameMode::Exploring { return false; }
        if self.nearby_bed_idx().is_none() { return false; }
        // Move onto bed like lie_on_nearby_bed
        if let Some(idx) = self.nearby_bed_idx() {
            let b = &self.beds[idx];
            self.player.x = b.x;
            self.player.y = b.y;
        }
        self.begin_sleep();
        true
    }

    fn begin_sleep(&mut self) {
        let hours = Self::sleep_hours_by_debt(self.player.stats.sleep);
        self.sleep_pending_hours = hours;
        self.sleep_end_ms = self.last_ms + 2500;
        self.mode = GameMode::Sleeping;
        self.set_message(format!("Durmiendo {}h...", hours), 30);
    }

    fn finish_sleep(&mut self) {
        let dgh = self.sleep_pending_hours as f64;
        self.hour += dgh;
        while self.hour >= 24.0 {
            self.hour -= 24.0;
            self.day += 1;
        }
        {
            let s = &mut self.player.stats;
            s.body = (s.body - 0.20 * dgh).max(0.0);
            s.mind = (s.mind - 0.20 * dgh).max(0.0);
            s.sleep = 0.0;
            s.thirst = (s.thirst + 5.0 * dgh).min(100.0);
            s.hunger = (s.hunger + 3.0 * dgh).min(100.0);
            s.hygiene = s.hygiene - 1.0 * dgh;
            s.stamina = 100.0;
        }
        if self.player.stats.hygiene < 0.0 {
            if self.consume_paper_unit() {
                self.player.stats.hygiene = 25.0;
            } else {
                self.player.stats.hygiene = 0.0;
            }
        }
        if self.player.stats.body <= 0.0 || self.player.stats.mind <= 0.0 {
            self.mode = GameMode::Dead;
            self.set_message("You died in your sleep.".into(), 30);
            return;
        }
        self.stamina_grace_ms = STAMINA_GRACE_MS;
        self.stamina_state = StaminaState::Optimal;
        self.mode = GameMode::Exploring;
        let hours = self.sleep_pending_hours;
        self.sleep_pending_hours = 0;
        self.set_message(format!("You wake up after {}h. Energy at 100%.", hours), 14);
    }

    pub fn get_up(&mut self) {
        let was_initial = matches!(self.mode, GameMode::InitialWake);
        self.mode = GameMode::Exploring;
        if was_initial {
            self.player.dir_x = 0.0;
            self.player.dir_y = 1.0;
            self.player.plane_x = -0.66;
            self.player.plane_y = 0.0;
            self.set_message("You wake up in ICU Ward 104. The monitors are beeping rhythmically; everyone else vanished during The Fanfare.".into(), 16);
        }
        self.tick();
    }

    pub fn nearby_dropped_idx(&self) -> Option<usize> {
        self.dropped.iter().position(|d| {
            let dx = d.x - self.player.x;
            let dy = d.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.2
        })
    }

    pub fn pickup_nearby_dropped(&mut self) -> bool {
        if let Some(idx) = self.nearby_dropped_idx() {
            if !self.player.has_backpack {
                self.set_message("Necesitas la mochila para guardar objetos.".into(), 10);
                return true;
            }
            let item = self.dropped[idx].item.clone();
            let label = item.label();
            if self.player.inventory.add(item.clone()) {
                self.dropped.remove(idx);
                self.set_message(format!("Recoges {}", label), 12);
                self.tick();
                return true;
            } else {
                self.set_message("Mochila llena.".into(), 8);
                return true;
            }
        }
        false
    }

    pub fn drop_inventory_slot(&mut self, idx: usize) -> bool {
        if idx >= self.player.inventory.slots.len() { return false; }
        if let Some(item) = self.player.inventory.slots[idx].take() {
            let label = item.label();
            self.set_message(format!("Tiras {}", label), 10);
            self.dropped.push(DroppedItem {
                x: self.player.x + self.player.dir_x * 0.5,
                y: self.player.y + self.player.dir_y * 0.5,
                item,
            });
            return true;
        }
        false
    }

    pub fn use_inventory_slot(&mut self, idx: usize) -> bool {
        if idx >= self.player.inventory.slots.len() { return false; }
        let item = match &self.player.inventory.slots[idx] {
            Some(it) => it.clone(),
            None => return false,
        };
        match item.kind {
            ItemKind::Laptop => {
                self.mode = GameMode::Laptop;
                self.laptop_cursor = 0;
                self.set_message("You power on the emergency terminal.".into(), 8);
            }
            ItemKind::Canteen => {
                if self.player.canteen_fill > 0.0 {
                    let sip = self.player.canteen_fill.min(40.0);
                    self.player.canteen_fill -= sip;
                    let s = &mut self.player.stats;
                    s.thirst = (s.thirst - sip).max(0.0);
                    self.set_message(
                        format!("You drink from the water flask. Remaining: {:.0}%", self.player.canteen_fill),
                        8,
                    );
                } else {
                    self.set_message("Water flask empty. Fill it at a sink.".into(), 10);
                }
            }
            ItemKind::Refresco => {
                let s = &mut self.player.stats;
                s.thirst = (s.thirst - 35.0).max(0.0);
                s.stamina = (s.stamina + 15.0).min(100.0);
                s.hunger = (s.hunger - 5.0).max(0.0);
                self.player.inventory.slots[idx] = Some(Item::new(ItemKind::RefrescoEnvase));
                self.set_message("You drink the energy beverage.".into(), 8);
            }
            ItemKind::Cafe => {
                let s = &mut self.player.stats;
                s.thirst = (s.thirst - 20.0).max(0.0);
                s.sleep = (s.sleep - 50.0).max(0.0);
                self.player.inventory.slots[idx] = Some(Item::new(ItemKind::CafeEnvase));
                self.set_message("You drink the coffee. Drowsiness dissipates.".into(), 8);
            }
            ItemKind::Snack => {
                let s = &mut self.player.stats;
                s.hunger = (s.hunger - 25.0).max(0.0);
                s.thirst = (s.thirst + 5.0).min(100.0);
                self.player.inventory.slots[idx] = Some(Item::new(ItemKind::SnackEnvase));
                self.set_message("Te comes la barrita energética.".into(), 8);
            }
            ItemKind::RefrescoEnvase | ItemKind::CafeEnvase | ItemKind::SnackEnvase => {
                self.set_message("Empty container. Drop it (T).".into(), 8);
            }
            ItemKind::Hoodie | ItemKind::Pants | ItemKind::HospitalGown | ItemKind::Scrubs => {
                let label = match item.kind {
                    ItemKind::Hoodie => "Heavy Jacket",
                    ItemKind::Pants => "Hospital Trousers",
                    ItemKind::HospitalGown => "Hospital Gown",
                    ItemKind::Scrubs => "Medical Scrubs",
                    _ => "Garment",
                };
                if let Some(slot) = item.body_slot() {
                    self.player.wardrobe.set(slot, label.into());
                    self.player.inventory.slots[idx] = None;
                    self.set_message(format!("You put on {}.", label), 10);
                }
            }
            _ => {
                self.set_message(format!("You don't know what to do with {}", item.label()), 8);
            }
        }
        true
    }

    // Dialog impls moved to crate::game::dialogs.

    // Save/load impls moved to crate::game::save.
}

fn detect_message_style(msg: &str) -> MessageStyle {
    let trimmed = msg.trim_start();
    if trimmed.starts_with("Tú:") || trimmed.starts_with("Tu:") {
        MessageStyle::Protagonist
    } else if trimmed.contains(":") && (trimmed.contains("'") || trimmed.contains("«")) {
        MessageStyle::Other
    } else {
        MessageStyle::Neutral
    }
}

fn push_away_from_doors(x: &mut f64, y: &mut f64, doors: &[(f64, f64)], min_d: f64) {
    for (dx, dy) in doors {
        let vx = *x - dx;
        let vy = *y - dy;
        let d2 = vx * vx + vy * vy;
        if d2 < 0.0001 { continue; }
        let d = d2.sqrt();
        if d < min_d {
            let push = (min_d - d) * 0.6;
            *x += vx / d * push;
            *y += vy / d * push;
        }
    }
}

fn clothing_to_item(kind: ClothingKind) -> Item {
    let ikind = match kind {
        ClothingKind::Hoodie => ItemKind::Hoodie,
        ClothingKind::Pants => ItemKind::Pants,
        ClothingKind::HospitalGown => ItemKind::HospitalGown,
        ClothingKind::ScrubBundle => ItemKind::Scrubs,
    };
    Item::new(ikind)
}

// SaveState struct moved to crate::game::save.

impl World {
    pub fn process_pending(&mut self, now_ms: u64) {
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].0 <= now_ms {
                let (_, text, ttl) = self.pending.remove(i);
                self.set_message(text, ttl);
            } else {
                i += 1;
            }
        }
    }

    pub fn nearby_bench_idx(&self) -> Option<usize> {
        self.benches.iter().position(|b| {
            let dx = b.x - self.player.x;
            let dy = b.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 0.9
        })
    }

    pub fn nearby_sofa_idx(&self) -> Option<usize> {
        self.sofas.iter().position(|s| {
            let dx = s.x - self.player.x;
            let dy = s.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.0
        })
    }

    pub fn sit_on_nearby_sofa(&mut self) -> bool {
        if let Some(idx) = self.nearby_sofa_idx() {
            let s = &self.sofas[idx];
            self.player.x = s.x;
            self.player.y = s.y;
            self.mode = GameMode::Sitting;
            self.set_message("Te sientas en el sofá.".into(), 6);
            return true;
        }
        false
    }

    pub fn nearby_bed_idx(&self) -> Option<usize> {
        self.beds.iter().position(|b| {
            let dx = b.x - self.player.x;
            let dy = b.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.0
        })
    }

    pub fn sit_on_nearby_bench(&mut self) -> bool {
        if let Some(idx) = self.nearby_bench_idx() {
            let b = &self.benches[idx];
            self.player.x = b.x;
            self.player.y = b.y;
            self.mode = GameMode::Sitting;
            self.set_message("Te sientas en el banco.".into(), 6);
            return true;
        }
        false
    }

    pub fn lie_on_nearby_bed(&mut self) -> bool {
        if let Some(idx) = self.nearby_bed_idx() {
            let b = &self.beds[idx];
            self.player.x = b.x;
            self.player.y = b.y;
            self.mode = GameMode::Lying;
            self.set_message("Te tumbas en la cama.".into(), 6);
            return true;
        }
        false
    }

    pub fn rest_tick(&mut self) {
        // Real-time regen handled by advance_realtime.
    }

    pub fn nearby_fixture_idx(&self) -> Option<usize> {
        let mut best: Option<(usize, f64)> = None;
        for (i, f) in self.fixtures.iter().enumerate() {
            let dx = f.x - self.player.x;
            let dy = f.y - self.player.y;
            let d = (dx * dx + dy * dy).sqrt();
            if d < 1.0 {
                match best {
                    Some((_, bd)) if bd <= d => {}
                    _ => best = Some((i, d)),
                }
            }
        }
        best.map(|(i, _)| i)
    }

    pub fn nearby_window_idx(&self) -> Option<usize> {
        self.windows.iter().position(|w| {
            let dx = w.x - self.player.x;
            let dy = w.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.2
        })
    }

    pub fn observe_window_weather(&mut self) -> bool {
        if self.nearby_window_idx().is_none() { return false; }
        self.weather_seen = true;
        let label = match self.weather { Weather::Clear => "despejado", Weather::Cloudy => "nublado" };
        self.set_message(format!("Miras por la ventana. Fuera está {}.", label), 8);
        self.tick();
        true
    }

    pub fn nearby_vending_idx(&self) -> Option<usize> {
        self.vending.iter().position(|v| {
            let dx = v.x - self.player.x;
            let dy = v.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.4
        })
    }

    pub fn use_nearby_vending(&mut self, secondary: bool) -> bool {
        let idx = match self.nearby_vending_idx() { Some(i) => i, None => return false };
        if !self.player.has_backpack {
            self.set_message("Necesitas la mochila.".into(), 8);
            return true;
        }
        let vkind = self.vending[idx].kind;
        let (ikind, label) = match (vkind, secondary) {
            (VendingKind::Drinks, false) => (ItemKind::Refresco, "un refresco"),
            (VendingKind::Drinks, true) => (ItemKind::Cafe, "un café"),
            (VendingKind::Snacks, false) => (ItemKind::Snack, "una barrita"),
            (VendingKind::Snacks, true) => {
                self.set_message("Sólo hay snacks aquí.".into(), 6);
                return true;
            }
        };
        if self.vending[idx].stock == 0 {
            self.set_message("Vacía.".into(), 6);
            return true;
        }
        let item = Item::new(ikind);
        if !self.player.inventory.add(item) {
            self.set_message("Mochila llena.".into(), 8);
            return true;
        }
        self.vending[idx].stock -= 1;
        self.set_message(format!("Coges {}. Quedan {}.", label, self.vending[idx].stock), 8);
        self.tick();
        true
    }

    pub fn toggle_backpack(&mut self) -> bool {
        if self.player.has_backpack {
            // Drop backpack with current contents at player position
            let backpack_contents: Vec<Item> = self.player.inventory.slots.iter()
                .take(6)
                .filter_map(|s| s.clone())
                .collect();
            for slot in self.player.inventory.slots.iter_mut().take(6) {
                *slot = None;
            }
            let mut bp = crate::equippables::Backpack::player_starter(self.player.x, self.player.y);
            bp.contents = backpack_contents;
            bp.taken = false;
            self.equippables.push(crate::equippables::Equippable::Backpack(bp));
            self.player.has_backpack = false;
            self.set_message("Te quitas la mochila.".into(), 8);
            self.tick();
            true
        } else {
            self.set_message("No llevas mochila.".into(), 6);
            false
        }
    }

    fn sync_inventory_capacity(&mut self) {
        let cap = if self.player.wardrobe.wearing_pants() { 8 } else { 6 };
        self.player.inventory.set_capacity(cap);
    }

    fn add_to_inventory_effective(&mut self, item: Item) -> bool {
        self.sync_inventory_capacity();
        self.player.inventory.add(item)
    }

    pub fn take_paper_from_nearby_toilet(&mut self) -> bool {
        let idx = match self.nearby_fixture_idx() { Some(i) => i, None => return false };
        if self.fixtures[idx].kind != FixtureKind::Toilet { return false; }
        let units = self.fixtures[idx].paper_units;
        if units == 0 { return false; }
        if !self.player.has_backpack {
            self.set_message("Necesitas la mochila.".into(), 8);
            return true;
        }
        // Try to stack with existing roll in inventory
        for slot in self.player.inventory.slots.iter_mut() {
            if let Some(it) = slot.as_mut() {
                if it.kind == ItemKind::ToiletPaper {
                    it.units += units;
                    self.fixtures[idx].paper_units = 0;
                    self.set_message(format!("Coges rollo (+{} usos).", units), 8);
                    self.tick();
                    return true;
                }
            }
        }
        let item = Item::new_with_units(ItemKind::ToiletPaper, units);
        if !self.add_to_inventory_effective(item) {
            self.set_message("Inventario lleno.".into(), 8);
            return true;
        }
        self.fixtures[idx].paper_units = 0;
        self.set_message(format!("Coges rollo ({} usos).", units), 8);
        self.tick();
        true
    }

    pub fn player_has_paper(&self) -> bool {
        self.player.inventory.slots.iter().any(|s| matches!(s, Some(it) if it.kind == ItemKind::ToiletPaper && it.units > 0))
    }

    fn consume_paper_unit(&mut self) -> bool {
        // Prefer player roll first
        for slot in self.player.inventory.slots.iter_mut() {
            if let Some(it) = slot.as_mut() {
                if it.kind == ItemKind::ToiletPaper && it.units > 0 {
                    it.units -= 1;
                    if it.units == 0 { *slot = None; }
                    return true;
                }
            }
        }
        false
    }

    pub fn has_canteen_in_inventory(&self) -> bool {
        self.player.inventory.slots.iter()
            .any(|slot| matches!(slot, Some(it) if it.kind == ItemKind::Canteen))
    }

    /// Z near sink: priority = fill canteen if present & not full, else wash.
    pub fn sink_secondary_action(&mut self) -> bool {
        let idx = match self.nearby_fixture_idx() { Some(i) => i, None => return false };
        if self.fixtures[idx].kind != FixtureKind::Sink { return false; }
        if self.has_canteen_in_inventory() && self.player.canteen_fill < 100.0 {
            self.player.canteen_fill = 100.0;
            self.set_message("Rellenas la cantimplora.".into(), 8);
            self.tick();
            return true;
        }
        self.player.stats.hygiene = (self.player.stats.hygiene + 20.0).min(100.0);
        self.set_message("Te aseas en el lavabo.".into(), 8);
        self.tick();
        true
    }

    pub fn drink_from_canteen(&mut self) -> bool {
        if !self.has_canteen_in_inventory() {
            self.set_message("No llevas cantimplora.".into(), 8);
            return false;
        }
        if self.player.canteen_fill <= 0.0 {
            self.set_message("Cantimplora vacía. Rellénala en un grifo.".into(), 10);
            return false;
        }
        let sip = self.player.canteen_fill.min(40.0);
        self.player.canteen_fill -= sip;
        let s = &mut self.player.stats;
        s.thirst = (s.thirst - sip).max(0.0);
        self.set_message(
            format!("Bebes de la cantimplora. Queda: {:.0}%", self.player.canteen_fill),
            8,
        );
        self.tick();
        true
    }

    pub fn use_nearby_fixture(&mut self) -> bool {
        let idx = match self.nearby_fixture_idx() { Some(i) => i, None => return false };
        let kind = self.fixtures[idx].kind;
        match kind {
            FixtureKind::Sink => {
                if self.sink_drink_cooldown_ms > 0 {
                    self.set_message("Aún estás saciado.".into(), 6);
                    return true;
                }
                let s = &mut self.player.stats;
                s.thirst = (s.thirst - 45.0).max(0.0);
                self.sink_drink_cooldown_ms = 60_000;
                self.set_message("Bebes agua del grifo.".into(), 8);
            }
            FixtureKind::Shower => {
                let s = &mut self.player.stats;
                s.hygiene = 100.0;
                s.thermal = (s.thermal + 6.0).min(100.0);
                self.set_message("Te duchas. Como nuevo.".into(), 8);
            }
            FixtureKind::Toilet => {
                let has_own = self.player_has_paper();
                let fixture_units = self.fixtures[idx].paper_units;
                if has_own {
                    self.consume_paper_unit();
                    self.player.stats.hygiene = (self.player.stats.hygiene + 8.0).min(100.0);
                    self.set_message("Usas el water con tu rollo.".into(), 8);
                } else if fixture_units > 0 {
                    self.fixtures[idx].paper_units -= 1;
                    self.player.stats.hygiene = (self.player.stats.hygiene + 8.0).min(100.0);
                    self.set_message("Usas el water. Papel usado.".into(), 8);
                } else {
                    self.player.stats.hygiene = (self.player.stats.hygiene - 10.0).max(0.0);
                    self.set_message("Usas el water. Sin papel, aseo parcial.".into(), 10);
                }
            }
            FixtureKind::Urinal => {
                self.player.stats.hygiene = (self.player.stats.hygiene - 6.0).max(0.0);
                self.set_message("Usas el urinario.".into(), 6);
            }
        }
        self.tick();
        true
    }

    pub fn nearby_equippable_idx(&self) -> Option<usize> {
        self.equippables.iter().position(|e| {
            if e.taken() { return false; }
            let (x, y) = e.position();
            let dx = x - self.player.x;
            let dy = y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.4
        })
    }

    pub fn nearby_clothing_idx(&self) -> Option<usize> {
        self.clothing.iter().position(|c| {
            if c.taken { return false; }
            let dx = c.x - self.player.x;
            let dy = c.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.6
        })
    }

    pub fn nearby_pickable_clothing_idx(&self) -> Option<usize> {
        self.clothing.iter().position(|c| {
            if c.taken || !c.is_pickable() { return false; }
            let dx = c.x - self.player.x;
            let dy = c.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 1.4
        })
    }

    pub fn nearby_npc_idx(&self) -> Option<usize> {
        self.npcs.iter().position(|n| {
            let dx = n.x - self.player.x;
            let dy = n.y - self.player.y;
            (dx * dx + dy * dy).sqrt() < 2.2
        })
    }

    pub fn front_door_pos(&self) -> Option<(usize, usize)> {
        let fx = (self.player.x + self.player.dir_x * 0.6) as i32;
        let fy = (self.player.y + self.player.dir_y * 0.6) as i32;
        if fx < 0 || fy < 0 { return None; }
        let (fx, fy) = (fx as usize, fy as usize);
        if self.map.get(fx, fy).is_any_door() { Some((fx, fy)) } else { None }
    }

    pub fn toggle_front_door(&mut self) -> bool {
        if let Some((fx, fy)) = self.front_door_pos() {
            let now = self.last_ms;
            match self.map.get(fx, fy) {
                Tile::Door { open } => {
                    let new_open = !open;
                    self.map.set(fx, fy, Tile::Door { open: new_open });
                    self.set_door_timer(fx, fy, new_open, now);
                    let msg = if open { "You close the door." } else { "You open the door." };
                    self.set_message(msg.into(), 4);
                    self.tick();
                    return true;
                }
                Tile::MainDoor { open } => {
                    let new_open = !open;
                    self.map.set(fx, fy, Tile::MainDoor { open: new_open });
                    self.set_door_timer(fx, fy, new_open, now);
                    let msg = if open { "You close the main door." } else { "You open the main door." };
                    self.set_message(msg.into(), 4);
                    self.tick();
                    return true;
                }
                Tile::BathroomDoor { open } => {
                    let new_open = !open;
                    self.map.set(fx, fy, Tile::BathroomDoor { open: new_open });
                    self.set_door_timer(fx, fy, new_open, now);
                    let msg = if open { "You close the restroom door." } else { "You open the restroom door." };
                    self.set_message(msg.into(), 4);
                    self.tick();
                    return true;
                }
                Tile::OperatingDoor { open } => {
                    let has_clearance = self.player.inventory.slots.iter().any(|s| {
                        matches!(s.as_ref().map(|it| it.kind), Some(ItemKind::ArchiveClearance | ItemKind::TriageKeycard))
                    });
                    if !open && !has_clearance {
                        self.set_message("[SECTOR SEALED: BACKUP RELAY OFFLINE]".into(), 16);
                        return true;
                    }
                    let new_open = !open;
                    self.map.set(fx, fy, Tile::OperatingDoor { open: new_open });
                    self.set_door_timer(fx, fy, new_open, now);
                    let msg = if open { "You close the secure sector door." } else { "You open the secure sector door." };
                    self.set_message(msg.into(), 4);
                    self.tick();
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn set_door_timer(&mut self, tx: usize, ty: usize, open: bool, now: u64) {
        if let Some(d) = self.doors.iter_mut().find(|d| d.tx == tx && d.ty == ty) {
            d.open_since_ms = if open { Some(now) } else { None };
        }
    }

    fn auto_close_doors(&mut self) {
        let now = self.last_ms;
        let mut to_close: Vec<(usize, usize)> = Vec::new();
        for d in &self.doors {
            let (Some(t), Some(threshold)) = (d.open_since_ms, d.auto_close_ms) else { continue };
            if now.saturating_sub(t) >= threshold {
                to_close.push((d.tx, d.ty));
            }
        }
        for (tx, ty) in to_close {
            match self.map.get(tx, ty) {
                Tile::Door { .. } => self.map.set(tx, ty, Tile::Door { open: false }),
                Tile::MainDoor { .. } => self.map.set(tx, ty, Tile::MainDoor { open: false }),
                Tile::BathroomDoor { .. } => self.map.set(tx, ty, Tile::BathroomDoor { open: false }),
                Tile::OperatingDoor { .. } => self.map.set(tx, ty, Tile::OperatingDoor { open: false }),
                _ => {}
            }
            if let Some(d) = self.doors.iter_mut().find(|d| d.tx == tx && d.ty == ty) {
                d.open_since_ms = None;
            }
        }
    }

    pub fn pick_nearby(&mut self) -> bool {
        if self.pickup_nearby_dropped() { return true; }
        if let Some(idx) = self.nearby_equippable_idx() {
            let label = self.equippables[idx].label().to_string();
            let contents = self.equippables[idx].contents();
            let is_backpack = matches!(self.equippables[idx], Equippable::Backpack(_));
            self.equippables[idx].mark_taken();
            for item in contents {
                self.player.inventory.add(item);
            }
            if is_backpack {
                self.player.has_backpack = true;
                self.set_message("You pick up your backpack. You can now carry up to 6 items...".into(), 20);
            } else {
                self.set_message(format!("You pick up {}", label), 14);
            }
            self.tick();
            return true;
        }
        if let Some(idx) = self.nearby_pickable_clothing_idx() {
            let kind = self.clothing[idx].kind;
            if self.player.has_backpack {
                let item = clothing_to_item(kind);
                let label = item.label();
                if !self.player.inventory.add(item) {
                    self.set_message("Backpack full.".into(), 8);
                    return true;
                }
                self.clothing[idx].taken = true;
                self.set_message(format!("You store {} in the backpack.", label), 12);
            } else {
                let label = self.clothing[idx].wardrobe_label().to_string();
                let slot = self.clothing[idx].body_slot();
                self.player.wardrobe.set(slot, label.clone());
                self.clothing[idx].taken = true;
                self.set_message(format!("You put on {}.", label), 12);
            }
            self.tick();
            return true;
        }
        false
    }

    pub fn active_message(&self) -> Option<String> {
        if let Some(m) = &self.message {
            return Some(m.clone());
        }
        if let Some(idx) = self.nearby_clothing_idx() {
            return Some(self.clothing[idx].description().into());
        }
        None
    }
}
