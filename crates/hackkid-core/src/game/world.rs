use std::cell::OnceCell;

use serde::{Deserialize, Serialize};

use crate::engine::boxes::{BoxInput, BoxKind};
use crate::engine::decals::Decals;
use crate::engine::lighting::{LightKind, LightMap, LightSource};
use crate::engine::map::{Map, Tile};
use crate::equippables::{Backpack, Equippable};
use crate::game::items::{Item, ItemKind};
use crate::game::level;
use crate::game::loot::{self, Rng};
use crate::game::player::{BACKPACK_SLOTS, Garment, Obstacle, Player};
use crate::game::tracks::TRACKS;
use crate::objects::{
    Bed, BedKind, Bench, Clothing, Container, Door, Fixture, FixtureKind, Fluorescent, FluorescentState, Npc, Outlet,
    Prop, Sofa, VendingKind, VendingMachine, Visual,
};

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

impl GameMode {
    pub fn shows_ceiling(&self) -> bool {
        matches!(self, GameMode::InitialWake | GameMode::Lying)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum StaminaState {
    Idle,
    Draining,
    Recovering,
    Optimal,
}

/// 2 real minutes are one game hour.
pub const REAL_MS_PER_GAME_HOUR: f64 = 120_000.0;
pub const STAMINA_GRACE_MS: u64 = 120_000;
/// The longest stretch of time one update may simulate, so a frontend that stalls
/// (a hidden browser tab, a suspended terminal) doesn't come back to a dead player.
const MAX_STEP_MS: u64 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Weather {
    Clear,
    Cloudy,
}

impl Weather {
    pub fn label(self) -> &'static str {
        match self {
            Weather::Clear => "Clear",
            Weather::Cloudy => "Overcast",
        }
    }
}

/// The weather changes every 6 game hours, the same way in every playthrough.
pub fn weather_at(day: u32, hour: f64) -> Weather {
    let block = u64::from(day) * 4 + (hour / 6.0) as u64;
    // splitmix64: a well-mixed hash of the block number
    let mut z = block.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    if z % 100 < 35 { Weather::Cloudy } else { Weather::Clear }
}

/// Who is talking, which decides how a message is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageStyle {
    /// Olivia, the player.
    Protagonist,
    /// Anyone else.
    Other,
    /// Narration and system messages.
    Neutral,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub style: MessageStyle,
    pub expires_ms: u64,
}

/// Something the world needs the frontend to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    /// Write [`World::save_json`] somewhere and report back with a message.
    Save,
    /// Read the save back and pass it to [`World::load_json`].
    Load,
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Laptop {
    pub cursor: usize,
    pub battery: f64,
    /// Index into [`TRACKS`] of the transmission being played, if any.
    pub playing: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroppedItem {
    pub x: f64,
    pub y: f64,
    pub item: Item,
}

pub const TITLE_ITEMS: [&str; 3] = ["New game", "Load game", "Quit"];
pub const MENU_ITEMS: [&str; 5] = ["Resume", "Save game", "Load game", "Quit to title", "Quit"];

/// Battery use and charge, in % per real second.
const CHARGE_RATE: f64 = 0.25;
const SCREEN_DRAIN: f64 = 0.125;
const BROADCAST_DRAIN: f64 = 0.05;
const OUTLET_REACH: f64 = 1.5;

pub struct World {
    pub map: Map,
    pub player: Player,
    pub day: u32,
    pub hour: f64,
    pub mode: GameMode,
    pub doors: Vec<Door>,
    pub fluorescents: Vec<Fluorescent>,
    pub equippables: Vec<Equippable>,
    pub clothing: Vec<Clothing>,
    pub beds: Vec<Bed>,
    pub benches: Vec<Bench>,
    pub sofas: Vec<Sofa>,
    pub outlets: Vec<Outlet>,
    pub fixtures: Vec<Fixture>,
    pub vending: Vec<VendingMachine>,
    pub npcs: Vec<Npc>,
    pub props: Vec<Prop>,
    pub containers: Vec<Container>,
    pub decals: Decals,
    pub dropped: Vec<DroppedItem>,
    /// Picks what's in every container; a new game gets a new seed.
    pub seed: u64,
    pub laptop: Laptop,
    pub julian_line: usize,
    pub message: Option<Message>,
    /// The frontend's clock at the last [`World::update`], in ms.
    pub now_ms: u64,
    pub stamina_grace_ms: u64,
    pub stamina_state: StaminaState,
    pub walked: bool,
    pub sleep_end_ms: u64,
    pub sleep_hours: u32,
    pub weather: Weather,
    pub weather_seen: bool,
    pub menu_cursor: usize,
    pub sink_drink_cooldown_ms: u64,
    /// Where the player stood before sitting or lying down; they get up there again.
    pub rest_return: Option<(f64, f64)>,
    requests: Vec<Request>,
    obstacles: Vec<Obstacle>,
    lights: Vec<LightSource>,
    lightmap: OnceCell<LightMap>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// The hospital with the standard loot (seed 1). See [`World::with_seed`].
    pub fn new() -> Self {
        Self::with_seed(1)
    }

    /// A new game: the same hospital, with what's in the cupboards rolled from `seed`.
    pub fn with_seed(seed: u64) -> Self {
        let mut lvl = level::hospital();
        let mut rng = Rng::new(seed);
        for c in &mut lvl.containers {
            c.items = loot::roll(c.kind, &mut rng);
        }
        for (i, item) in lvl.guaranteed {
            lvl.containers[i].items.push(item);
        }
        let (bx, by) = level::PLAYER_BED;
        let mut player = Player::new(bx, by);
        player.face(0.0, 1.0);
        let mut world = Self {
            map: lvl.map,
            player,
            day: 1,
            hour: 8.0,
            mode: GameMode::Startup,
            doors: lvl.doors,
            fluorescents: lvl.fluorescents,
            equippables: lvl.equippables,
            clothing: lvl.clothing,
            beds: lvl.beds,
            benches: lvl.benches,
            sofas: lvl.sofas,
            outlets: lvl.outlets,
            fixtures: lvl.fixtures,
            vending: lvl.vending,
            npcs: lvl.npcs,
            props: lvl.props,
            containers: lvl.containers,
            decals: lvl.decals,
            dropped: lvl.items,
            seed,
            laptop: Laptop { cursor: 0, battery: 62.0, playing: None },
            julian_line: 0,
            message: None,
            now_ms: 0,
            stamina_grace_ms: STAMINA_GRACE_MS,
            stamina_state: StaminaState::Optimal,
            walked: false,
            sleep_end_ms: 0,
            sleep_hours: 0,
            weather: Weather::Clear,
            weather_seen: false,
            menu_cursor: 0,
            sink_drink_cooldown_ms: 0,
            // you wake lying in bed; getting up puts you beside it
            rest_return: Some(level::PLAYER_START),
            requests: Vec::new(),
            obstacles: Vec::new(),
            lights: lvl.extra_lights,
            lightmap: OnceCell::new(),
        };
        world.weather = weather_at(world.day, world.hour);
        world.obstacles = world.build_obstacles();
        world
    }

    /// Back to the title screen with a fresh hospital (and fresh loot).
    pub fn reset(&mut self) {
        let now = self.now_ms;
        let seed = self.seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        *self = World::with_seed(seed);
        self.now_ms = now;
    }

    /// Everything box-shaped in the world, as drawn and as walked round.
    pub fn boxes(&self) -> Vec<BoxInput> {
        let mut out = Vec::new();
        for b in &self.beds {
            let kind = if b.kind == BedKind::Operating { BoxKind::OpTable } else { BoxKind::Bed };
            out.push(BoxInput::new(kind, b.x, b.y, b.facing));
        }
        for s in &self.sofas {
            out.extend(seat_and_back(BoxKind::SofaSeat, BoxKind::SofaBack, s.x, s.y, s.facing));
        }
        for b in &self.benches {
            let (seat, back) = if b.pew { (BoxKind::PewSeat, BoxKind::PewBack) } else { (BoxKind::BenchSeat, BoxKind::BenchBack) };
            out.extend(seat_and_back(seat, back, b.x, b.y, b.facing));
        }
        for v in &self.vending {
            let kind = match v.kind {
                VendingKind::Snacks => BoxKind::VendingSnacks,
                VendingKind::Drinks => BoxKind::VendingDrinks,
            };
            out.push(BoxInput::new(kind, v.x, v.y, v.facing));
        }
        for c in &self.containers {
            if let Some(kind) = c.kind.box_kind() {
                out.push(BoxInput::new(kind, c.x, c.y, c.facing));
            }
        }
        for p in &self.props {
            if let Visual::Box(kind) = p.kind.visual() {
                let mut b = BoxInput::new(kind, p.x, p.y, p.facing);
                b.z0 = p.elevation;
                out.push(b);
            }
        }
        out
    }

    /// What you can't walk through: furniture, fixtures, people, trees.
    fn build_obstacles(&self) -> Vec<Obstacle> {
        let mut o: Vec<Obstacle> = self
            .boxes()
            .iter()
            .map(|b| {
                let (x0, y0, x1, y1) = b.bounds();
                Obstacle { x0, y0, x1, y1 }
            })
            .collect();
        for p in &self.props {
            if let Some(r) = p.kind.solid_radius() {
                o.push(Obstacle::around(p.x, p.y, r));
            }
        }
        for f in &self.fixtures {
            o.push(Obstacle::around(f.x, f.y, 0.2));
        }
        for n in &self.npcs {
            o.push(Obstacle::around(n.x, n.y, 0.22));
        }
        o
    }

    pub fn obstacles(&self) -> &[Obstacle] {
        &self.obstacles
    }

    // ------------------------------------------------------------ messages

    /// Narration or a system message, shown for as long as it takes to read.
    pub fn say(&mut self, text: impl Into<String>) {
        self.speak(MessageStyle::Neutral, text);
    }

    pub fn speak(&mut self, style: MessageStyle, text: impl Into<String>) {
        let text = text.into();
        let read_ms = (1_800 + 55 * text.chars().count() as u64).clamp(2_500, 14_000);
        self.message = Some(Message { text, style, expires_ms: self.now_ms + read_ms });
    }

    /// What to show in the message box: the current message, or a description of
    /// clothing lying nearby.
    pub fn active_message(&self) -> Option<(&str, MessageStyle)> {
        if let Some(m) = &self.message {
            return Some((&m.text, m.style));
        }
        self.nearby_clothing_idx()
            .map(|i| (self.clothing[i].description(), MessageStyle::Neutral))
    }

    // ------------------------------------------------------------ frontend contract

    pub(crate) fn request(&mut self, r: Request) {
        self.requests.push(r);
    }

    /// Everything asked of the frontend since the last call.
    pub fn take_requests(&mut self) -> Vec<Request> {
        std::mem::take(&mut self.requests)
    }

    /// The transmission that should be audible right now.
    pub fn music(&self) -> Option<usize> {
        self.laptop.playing
    }

    /// The frontend reached the end of track `idx`: carry on with the next one.
    pub fn music_finished(&mut self, idx: usize) {
        if self.laptop.playing != Some(idx) {
            return;
        }
        if idx + 1 < TRACKS.len() {
            self.laptop.playing = Some(idx + 1);
        } else {
            self.laptop.playing = None;
            self.say("Transmission complete. The carrier wave falls silent.");
        }
    }

    /// The frontend can't play track `idx` (no audio device, no file...).
    pub fn music_unavailable(&mut self, idx: usize, why: &str) {
        if self.laptop.playing == Some(idx) {
            self.laptop.playing = None;
        }
        self.say(format!("[NO CARRIER] {why}"));
    }

    // ------------------------------------------------------------ time

    /// Advances the world to the frontend's clock `now_ms` (any monotonic milliseconds).
    pub fn update(&mut self, now_ms: u64) {
        let dt_ms = now_ms.saturating_sub(self.now_ms).min(MAX_STEP_MS);
        self.now_ms = now_ms;
        if self.message.as_ref().is_some_and(|m| now_ms >= m.expires_ms) {
            self.message = None;
        }
        match self.mode {
            GameMode::Startup | GameMode::InitialWake | GameMode::Menu | GameMode::Inventory | GameMode::Dead => return,
            GameMode::Sleeping => {
                if now_ms >= self.sleep_end_ms {
                    self.finish_sleep();
                }
                return;
            }
            _ => {}
        }
        if dt_ms == 0 {
            return;
        }
        let dt_game_h = (dt_ms as f64) / REAL_MS_PER_GAME_HOUR;
        let dt_real_s = (dt_ms as f64) / 1000.0;

        self.advance_clock(dt_game_h);
        let stamina_drains = self.stamina_grace_ms == 0 && self.mode == GameMode::Exploring;
        self.drain_needs(dt_game_h);
        let s = &mut self.player.stats;
        if stamina_drains {
            s.stamina = (s.stamina - 3.0 * dt_game_h).max(0.0);
        }
        s.thermal *= 0.995_f64.powf(dt_ms as f64 / 400.0);
        self.sink_drink_cooldown_ms = self.sink_drink_cooldown_ms.saturating_sub(dt_ms);
        // a paper roll is used up automatically before hygiene would go negative
        if self.player.stats.hygiene < 0.0 {
            if self.consume_paper_unit() {
                self.player.stats.hygiene = 25.0;
                self.say("You use some paper to freshen up.");
            } else {
                self.player.stats.hygiene = 0.0;
            }
        }

        if self.player.stats.body <= 0.0 || self.player.stats.mind <= 0.0 {
            self.die("You have died.");
            return;
        }

        // resting regenerates body and mind (sleep debt only goes away by sleeping)
        if matches!(self.mode, GameMode::Sitting | GameMode::Lying) {
            let k = if self.mode == GameMode::Lying { 0.12 } else { 0.05 };
            let s = &mut self.player.stats;
            s.body = (s.body + k * dt_real_s).min(100.0);
            s.mind = (s.mind + k * dt_real_s).min(100.0);
        }

        self.update_stamina(dt_ms, dt_real_s);
        self.sync_inventory_capacity();
        self.tick_laptop(dt_real_s);
        if self.player_outdoors() {
            self.weather_seen = true;
        }
        self.auto_close_doors();
    }

    fn advance_clock(&mut self, dt_game_h: f64) {
        self.hour += dt_game_h;
        while self.hour >= 24.0 {
            self.hour -= 24.0;
            self.day += 1;
        }
        let weather = weather_at(self.day, self.hour);
        if weather != self.weather {
            self.weather = weather;
            if self.player_outdoors() && self.message.is_none() {
                self.say(match weather {
                    Weather::Cloudy => "Clouds roll in over the city.",
                    Weather::Clear => "The clouds break up. The sky clears.",
                });
            }
        }
    }

    fn drain_needs(&mut self, dt_game_h: f64) {
        let s = &mut self.player.stats;
        s.body = (s.body - 0.20 * dt_game_h).max(0.0);
        s.mind = (s.mind - 0.20 * dt_game_h).max(0.0);
        s.sleep = (s.sleep + 1.0 * dt_game_h).min(100.0);
        s.thirst = (s.thirst + 5.0 * dt_game_h).min(100.0);
        s.hunger = (s.hunger + 3.0 * dt_game_h).min(100.0);
        s.hygiene -= 1.0 * dt_game_h;
        // critical needs wear you down faster
        if s.thirst >= 90.0 {
            s.body = (s.body - 1.0 * dt_game_h).max(0.0);
        }
        if s.hunger >= 90.0 {
            s.body = (s.body - 0.5 * dt_game_h).max(0.0);
        }
        if s.sleep >= 90.0 {
            s.mind = (s.mind - 1.0 * dt_game_h).max(0.0);
        }
    }

    fn die(&mut self, how: &str) {
        self.mode = GameMode::Dead;
        self.laptop.playing = None;
        self.say(how.to_string());
    }

    fn update_stamina(&mut self, dt_ms: u64, dt_real_s: f64) {
        let walked = std::mem::take(&mut self.walked);

        if self.stamina_grace_ms > 0 {
            self.player.stats.stamina = 100.0;
            self.stamina_grace_ms = self.stamina_grace_ms.saturating_sub(dt_ms);
            self.stamina_state = if self.stamina_grace_ms == 0 { StaminaState::Idle } else { StaminaState::Optimal };
            return;
        }

        let regen = match self.mode {
            GameMode::Lying => 3.5,
            GameMode::Sitting => 1.5,
            GameMode::Exploring if !walked => 0.3,
            _ => 0.0,
        };
        let prev = self.player.stats.stamina;
        let new_val = (prev + regen * dt_real_s).min(100.0);
        self.player.stats.stamina = new_val;
        if new_val > prev + 0.01 {
            self.stamina_state = StaminaState::Recovering;
        } else if new_val >= prev && self.stamina_state != StaminaState::Draining {
            self.stamina_state = StaminaState::Idle;
        }
        if new_val >= 100.0 {
            self.stamina_state = StaminaState::Optimal;
        }
    }

    pub fn player_outdoors(&self) -> bool {
        self.map.at(self.player.x, self.player.y).is_outdoor()
    }

    // ------------------------------------------------------------ the emergency terminal

    pub fn near_outlet(&self) -> bool {
        let (x, y) = (self.player.x, self.player.y);
        self.outlets.iter().any(|o| dist(o.x, o.y, x, y) < OUTLET_REACH)
    }

    fn tick_laptop(&mut self, dt_s: f64) {
        if !self.player.inventory.has(ItemKind::Laptop) {
            if self.laptop.playing.take().is_some() {
                self.say("The broadcast cuts out: the terminal isn't with you any more.");
            }
            if self.mode == GameMode::Laptop {
                self.mode = GameMode::Exploring;
            }
            return;
        }
        let on = self.mode == GameMode::Laptop;
        let mut rate = if self.near_outlet() { CHARGE_RATE } else { 0.0 };
        if on {
            rate -= SCREEN_DRAIN;
        } else if self.laptop.playing.is_some() {
            rate -= BROADCAST_DRAIN;
        }
        self.laptop.battery = (self.laptop.battery + rate * dt_s).clamp(0.0, 100.0);
        if self.laptop.battery <= 0.0 && (on || self.laptop.playing.is_some()) {
            self.laptop.playing = None;
            if on {
                self.mode = GameMode::Exploring;
            }
            self.say("The emergency terminal's battery dies. Find an outlet or a battery pack.");
        }
    }

    pub(crate) fn open_laptop(&mut self) {
        if self.laptop.battery <= 0.0 && !self.near_outlet() {
            self.say("The terminal's battery is dead. Stand by an outlet to charge it.");
            return;
        }
        self.mode = GameMode::Laptop;
        self.say("You power on the emergency terminal.");
    }

    pub(crate) fn close_laptop(&mut self) {
        self.mode = GameMode::Exploring;
        if self.laptop.playing.is_some() {
            self.say("The screen goes dark. The broadcast keeps playing.");
        }
    }

    pub(crate) fn toggle_track(&mut self) {
        let idx = self.laptop.cursor.min(TRACKS.len() - 1);
        if self.laptop.playing == Some(idx) {
            self.laptop.playing = None;
            self.say("Audio transmission stopped.");
        } else {
            self.laptop.playing = Some(idx);
            let t = &TRACKS[idx];
            self.say(format!("Tuning in to transmission {:02}: {}", t.n, t.name));
        }
    }

    // ------------------------------------------------------------ movement

    pub fn player_move_forward(&mut self, n: u8) {
        let sprint = n >= 2 && self.player.stats.stamina > 0.0;
        if sprint {
            self.player.sprinting = true;
            for _ in 0..n {
                self.player.move_forward(&self.map, &self.obstacles);
            }
            self.player.sprinting = false;
            if self.stamina_grace_ms == 0 {
                let weight = self.player.inventory.weight_ratio();
                let cost = 0.35 * n as f64 * (1.0 + weight);
                self.player.stats.stamina = (self.player.stats.stamina - cost).max(0.0);
                self.stamina_state = StaminaState::Draining;
            }
        } else {
            self.player.move_forward(&self.map, &self.obstacles);
        }
        self.walked = true;
    }

    pub fn player_move_backward(&mut self) {
        // no sprinting backwards
        self.player.move_backward(&self.map, &self.obstacles);
        self.walked = true;
    }

    // ------------------------------------------------------------ resting

    fn sleep_hours_by_debt(sleep_debt: f64) -> u32 {
        match sleep_debt {
            d if d < 10.0 => 1,
            d if d < 20.0 => 2,
            d if d < 30.0 => 3,
            d if d < 40.0 => 4,
            d if d < 50.0 => 5,
            d if d < 70.0 => 6,
            d if d < 90.0 => 7,
            _ => 8,
        }
    }

    pub(crate) fn sleep(&mut self) {
        if self.mode == GameMode::Lying {
            self.begin_sleep();
        }
    }

    pub(crate) fn sleep_at_bed(&mut self, idx: usize) {
        self.rest_return = Some((self.player.x, self.player.y));
        self.player.x = self.beds[idx].x;
        self.player.y = self.beds[idx].y;
        self.begin_sleep();
    }

    fn begin_sleep(&mut self) {
        let hours = Self::sleep_hours_by_debt(self.player.stats.sleep);
        self.sleep_hours = hours;
        self.sleep_end_ms = self.now_ms + 2500;
        self.mode = GameMode::Sleeping;
        self.message = None;
    }

    fn finish_sleep(&mut self) {
        let hours = self.sleep_hours;
        let dgh = hours as f64;
        self.advance_clock(dgh);
        self.drain_needs(dgh);
        self.player.stats.sleep = 0.0;
        self.player.stats.stamina = 100.0;
        if self.player.stats.hygiene < 0.0 {
            self.player.stats.hygiene = if self.consume_paper_unit() { 25.0 } else { 0.0 };
        }
        self.sleep_hours = 0;
        if let Some((x, y)) = self.rest_return.take() {
            self.player.x = x;
            self.player.y = y;
        }
        if self.player.stats.body <= 0.0 || self.player.stats.mind <= 0.0 {
            self.die("You died in your sleep.");
            return;
        }
        self.stamina_grace_ms = STAMINA_GRACE_MS;
        self.stamina_state = StaminaState::Optimal;
        self.mode = GameMode::Exploring;
        self.say(format!("You wake up after {hours}h. Energy at 100%."));
    }

    pub fn get_up(&mut self) {
        let was_initial = self.mode == GameMode::InitialWake;
        self.mode = GameMode::Exploring;
        if let Some((x, y)) = self.rest_return.take() {
            self.player.x = x;
            self.player.y = y;
        }
        if was_initial {
            self.player.face(1.0, 0.0);
            self.say("You wake up in ICU Ward 104. The monitors are beeping rhythmically; everyone else vanished during The Fanfare.");
        }
    }

    fn rest_at(&mut self, x: f64, y: f64, mode: GameMode, text: &str) {
        self.rest_return = Some((self.player.x, self.player.y));
        self.player.x = x;
        self.player.y = y;
        self.mode = mode;
        self.say(text.to_string());
    }

    pub(crate) fn sit_on_sofa(&mut self, idx: usize) {
        let (x, y) = (self.sofas[idx].x, self.sofas[idx].y);
        self.rest_at(x, y, GameMode::Sitting, "You sit on the sofa.");
    }

    pub(crate) fn sit_on_bench(&mut self, idx: usize) {
        let (x, y) = (self.benches[idx].x, self.benches[idx].y);
        self.rest_at(x, y, GameMode::Sitting, "You sit on the bench.");
    }

    pub(crate) fn lie_on_bed(&mut self, idx: usize) {
        let (x, y) = (self.beds[idx].x, self.beds[idx].y);
        self.rest_at(x, y, GameMode::Lying, "You lie down on the bed.");
    }

    // ------------------------------------------------------------ inventory

    fn sync_inventory_capacity(&mut self) {
        let cap = if self.player.wardrobe.wearing_pants() { BACKPACK_SLOTS + 2 } else { BACKPACK_SLOTS };
        self.player.inventory.set_capacity(cap);
        let last = self.player.inventory.capacity.saturating_sub(1);
        self.player.inventory_cursor = self.player.inventory_cursor.min(last);
    }

    /// Puts an item in the inventory, or on the floor at the player's feet when it's full.
    /// Returns whether it went into the inventory.
    fn give_item(&mut self, item: Item) -> bool {
        self.sync_inventory_capacity();
        match self.player.inventory.add(item) {
            Ok(()) => true,
            Err(item) => {
                self.drop_at_feet(item);
                false
            }
        }
    }

    /// Drops an item half a step ahead, or right here if that spot is inside a wall.
    fn drop_at_feet(&mut self, item: Item) {
        let p = &self.player;
        let (ax, ay) = (p.x + p.dir_x * 0.5, p.y + p.dir_y * 0.5);
        let (x, y) = if self.map.at(ax, ay).blocks_movement() { (p.x, p.y) } else { (ax, ay) };
        self.dropped.push(DroppedItem { x, y, item });
    }

    pub fn drop_inventory_slot(&mut self, idx: usize) -> bool {
        let Some(item) = self.player.inventory.slots.get_mut(idx).and_then(Option::take) else {
            return false;
        };
        self.say(format!("You drop the {}.", item.label()));
        self.drop_at_feet(item);
        true
    }

    pub fn use_inventory_slot(&mut self, idx: usize) -> bool {
        let Some(item) = self.player.inventory.slots.get(idx).cloned().flatten() else {
            return false;
        };
        match item.kind {
            ItemKind::Laptop => self.open_laptop(),
            ItemKind::Canteen => {
                self.drink_from_canteen();
            }
            ItemKind::PowerSupply => {
                if self.laptop.battery >= 99.0 {
                    self.say("The terminal is already fully charged.");
                } else {
                    self.laptop.battery = (self.laptop.battery + 50.0).min(100.0);
                    self.player.inventory.slots[idx] = None;
                    self.say(format!("You plug the battery pack into the terminal. Battery at {:.0}%.", self.laptop.battery));
                }
            }
            ItemKind::Refresco => {
                let s = &mut self.player.stats;
                s.thirst = (s.thirst - 35.0).max(0.0);
                s.stamina = (s.stamina + 15.0).min(100.0);
                s.hunger = (s.hunger - 5.0).max(0.0);
                self.player.inventory.slots[idx] = Some(Item::new(ItemKind::RefrescoEnvase));
                self.say("You drink the energy beverage.");
            }
            ItemKind::Cafe => {
                let s = &mut self.player.stats;
                s.thirst = (s.thirst - 20.0).max(0.0);
                s.sleep = (s.sleep - 50.0).max(0.0);
                self.player.inventory.slots[idx] = Some(Item::new(ItemKind::CafeEnvase));
                self.say("You drink the coffee. Drowsiness dissipates.");
            }
            ItemKind::Snack => {
                let s = &mut self.player.stats;
                s.hunger = (s.hunger - 25.0).max(0.0);
                s.thirst = (s.thirst + 5.0).min(100.0);
                self.player.inventory.slots[idx] = Some(Item::new(ItemKind::SnackEnvase));
                self.say("You eat the energy bar.");
            }
            ItemKind::WaterBottle | ItemKind::JuiceBox | ItemKind::Crackers | ItemKind::ChocolateBar | ItemKind::Apple | ItemKind::Sandwich => {
                let s = &mut self.player.stats;
                let (thirst, hunger, mind, text) = match item.kind {
                    ItemKind::WaterBottle => (-40.0, 0.0, 0.0, "You drink the bottle of water."),
                    ItemKind::JuiceBox => (-25.0, -6.0, 2.0, "You drink the juice. Too sweet, but good."),
                    ItemKind::Crackers => (6.0, -16.0, 0.0, "You eat the crackers. Dry."),
                    ItemKind::ChocolateBar => (3.0, -12.0, 6.0, "You eat the chocolate. It helps, a little."),
                    ItemKind::Apple => (-6.0, -12.0, 2.0, "You eat the apple."),
                    _ => (0.0, -35.0, 4.0, "You eat the sandwich. Still fresh enough."),
                };
                s.thirst = (s.thirst + thirst).clamp(0.0, 100.0);
                s.hunger = (s.hunger + hunger).clamp(0.0, 100.0);
                s.mind = (s.mind + mind).min(100.0);
                self.player.inventory.slots[idx] = None;
                self.say(text);
            }
            ItemKind::Ibuprofen | ItemKind::Paracetamol => {
                let s = &mut self.player.stats;
                s.mind = (s.mind + 12.0).min(100.0);
                s.body = (s.body + 3.0).min(100.0);
                if let Some(it) = self.player.inventory.slots[idx].as_mut() {
                    it.units = it.units.saturating_sub(1);
                    if it.units == 0 {
                        self.player.inventory.slots[idx] = None;
                    }
                }
                self.say(format!("You take a {}. The ache in your head eases.", if item.kind == ItemKind::Ibuprofen { "ibuprofen" } else { "paracetamol" }));
            }
            ItemKind::Bandage | ItemKind::Gauze | ItemKind::Peroxide => {
                let s = &mut self.player.stats;
                let (body, text) = match item.kind {
                    ItemKind::Bandage => (12.0, "You bandage the cut where the IV line tore out."),
                    ItemKind::Gauze => (6.0, "You press sterile gauze over the IV site."),
                    _ => (4.0, "You clean the IV wound with peroxide. It stings."),
                };
                s.body = (s.body + body).min(100.0);
                self.player.inventory.slots[idx] = None;
                self.say(text);
            }
            ItemKind::RefrescoEnvase | ItemKind::CafeEnvase | ItemKind::SnackEnvase => {
                self.say("Empty container. Drop it (T).");
            }
            ItemKind::Hoodie | ItemKind::Pants | ItemKind::HospitalGown | ItemKind::Scrubs => {
                let Some(garment) = item.garment() else { return false };
                // what was worn before goes into the slot the new garment came from
                let previous = self.player.wardrobe.wear(garment);
                self.player.inventory.slots[idx] = previous.and_then(Garment::item);
                self.sync_inventory_capacity();
                match previous {
                    Some(p) if p != Garment::Socks => self.say(format!("You put on the {} and pack the {}.", garment.label(), p.label())),
                    _ => self.say(format!("You put on the {}.", garment.label())),
                }
            }
            _ => self.say(format!("You don't know what to do with the {}.", item.label())),
        }
        true
    }

    pub(crate) fn drink_from_canteen(&mut self) -> bool {
        if !self.player.inventory.has(ItemKind::Canteen) {
            self.say("You don't have the water flask.");
            return false;
        }
        if self.player.canteen_fill <= 0.0 {
            self.say("The water flask is empty. Fill it at a sink.");
            return false;
        }
        let sip = self.player.canteen_fill.min(40.0);
        self.player.canteen_fill -= sip;
        let s = &mut self.player.stats;
        s.thirst = (s.thirst - sip).max(0.0);
        self.say(format!("You drink from the water flask. {:.0}% left.", self.player.canteen_fill));
        true
    }

    pub(crate) fn toggle_backpack(&mut self) {
        if !self.player.has_backpack {
            self.say("You're not carrying a backpack.");
            return;
        }
        // the backpack slots go down with it; the trouser pockets stay with you
        let contents: Vec<Item> = self.player.inventory.slots.iter_mut().take(BACKPACK_SLOTS).filter_map(Option::take).collect();
        let mut bp = Backpack::player_starter(self.player.x, self.player.y);
        bp.contents = contents;
        self.equippables.push(Equippable::Backpack(bp));
        self.player.has_backpack = false;
        if self.mode == GameMode::Inventory {
            self.mode = GameMode::Exploring;
        }
        self.say("You take off the backpack.");
    }

    fn consume_paper_unit(&mut self) -> bool {
        for slot in self.player.inventory.slots.iter_mut() {
            if let Some(it) = slot.as_mut()
                && it.kind == ItemKind::ToiletPaper
                && it.units > 0
            {
                it.units -= 1;
                if it.units == 0 {
                    *slot = None;
                }
                return true;
            }
        }
        false
    }

    pub fn player_has_paper(&self) -> bool {
        self.player.inventory.slots.iter().flatten().any(|it| it.kind == ItemKind::ToiletPaper && it.units > 0)
    }

    // ------------------------------------------------------------ things to interact with

    fn nearest<I>(&self, positions: I, reach: f64) -> Option<usize>
    where
        I: IntoIterator<Item = Option<(f64, f64)>>,
    {
        let (px, py) = (self.player.x, self.player.y);
        positions
            .into_iter()
            .enumerate()
            .filter_map(|(i, p)| p.map(|(x, y)| (i, dist(x, y, px, py))))
            .filter(|&(_, d)| d < reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn nearby_dropped_idx(&self) -> Option<usize> {
        self.nearest(self.dropped.iter().map(|d| Some((d.x, d.y))), 1.2)
    }

    pub fn nearby_equippable_idx(&self) -> Option<usize> {
        self.nearest(self.equippables.iter().map(|e| Some(e.position())), 1.4)
    }

    pub fn nearby_clothing_idx(&self) -> Option<usize> {
        self.nearest(self.clothing.iter().map(|c| (!c.taken).then_some((c.x, c.y))), 1.6)
    }

    pub fn nearby_pickable_clothing_idx(&self) -> Option<usize> {
        self.nearest(self.clothing.iter().map(|c| (!c.taken && c.is_pickable()).then_some((c.x, c.y))), 1.4)
    }

    pub fn nearby_npc_idx(&self) -> Option<usize> {
        self.nearest(self.npcs.iter().map(|n| Some((n.x, n.y))), 2.2)
    }

    /// The nearest of some footprints within `reach` of the player's position.
    fn nearest_box<I>(&self, boxes: I, reach: f64) -> Option<usize>
    where
        I: IntoIterator<Item = BoxInput>,
    {
        let (px, py) = (self.player.x, self.player.y);
        boxes
            .into_iter()
            .enumerate()
            .map(|(i, b)| {
                let (x0, y0, x1, y1) = b.bounds();
                (i, Obstacle { x0, y0, x1, y1 }.distance(px, py))
            })
            .filter(|&(_, d)| d < reach)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn nearby_sofa_idx(&self) -> Option<usize> {
        self.nearest_box(self.sofas.iter().map(|s| BoxInput::new(BoxKind::SofaSeat, s.x, s.y, s.facing)), 0.8)
    }

    pub fn nearby_bench_idx(&self) -> Option<usize> {
        self.nearest_box(self.benches.iter().map(|b| BoxInput::new(BoxKind::BenchSeat, b.x, b.y, b.facing)), 0.8)
    }

    pub fn nearby_bed_idx(&self) -> Option<usize> {
        self.nearest_box(self.beds.iter().map(|b| BoxInput::new(BoxKind::Bed, b.x, b.y, b.facing)), 0.8)
    }

    pub fn nearby_container_idx(&self) -> Option<usize> {
        let (px, py) = (self.player.x, self.player.y);
        self.containers
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let d = match c.kind.box_kind() {
                    Some(kind) => {
                        let (x0, y0, x1, y1) = BoxInput::new(kind, c.x, c.y, c.facing).bounds();
                        Obstacle { x0, y0, x1, y1 }.distance(px, py)
                    }
                    None => dist(c.x, c.y, px, py),
                };
                (i, d)
            })
            .filter(|&(_, d)| d < 0.85)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }

    pub fn nearby_fixture_idx(&self) -> Option<usize> {
        self.nearest(self.fixtures.iter().map(|f| Some((f.x, f.y))), 1.0)
    }

    /// A window right in front of the player.
    pub fn front_window(&self) -> Option<(usize, usize)> {
        let fx = self.player.x + self.player.dir_x * 0.6;
        let fy = self.player.y + self.player.dir_y * 0.6;
        (self.map.at(fx, fy) == Tile::WindowWall).then_some((fx as usize, fy as usize))
    }

    pub fn nearby_vending_idx(&self) -> Option<usize> {
        self.nearest_box(self.vending.iter().map(|v| BoxInput::new(BoxKind::VendingSnacks, v.x, v.y, v.facing)), 0.8)
    }

    pub fn has_canteen_in_inventory(&self) -> bool {
        self.player.inventory.has(ItemKind::Canteen)
    }

    pub(crate) fn observe_window(&mut self) {
        self.weather_seen = true;
        let h = self.hour;
        let time = if !(5.5..21.0).contains(&h) {
            "It's night. Street lamps buzz over empty cars."
        } else if h < 8.0 {
            "Dawn over the car park. Nothing moves."
        } else if h > 18.5 {
            "The sun is going down behind the city."
        } else {
            "Daylight over the car park and the empty street."
        };
        let sky = match self.weather {
            Weather::Clear => "The sky is clear.",
            Weather::Cloudy => "The sky is overcast.",
        };
        self.say(format!("You look out of the window. {time} {sky}"));
    }

    /// X at a container: take what's in it, as much as fits.
    pub(crate) fn search(&mut self, idx: usize) {
        let label = self.containers[idx].kind.label();
        self.containers[idx].searched = true;
        if self.containers[idx].items.is_empty() {
            self.say(format!("You search the {label}. Nothing useful."));
            return;
        }
        if !self.player.has_backpack {
            let n = self.containers[idx].items.len();
            self.say(format!("There's something in the {label} ({n}), but you need the backpack to carry it."));
            return;
        }
        self.sync_inventory_capacity();
        let mut taken = Vec::new();
        let mut left = Vec::new();
        for item in std::mem::take(&mut self.containers[idx].items) {
            let name = item.label();
            match self.player.inventory.add(item) {
                Ok(()) => taken.push(name),
                Err(item) => left.push(item),
            }
        }
        let full = !left.is_empty();
        self.containers[idx].items = left;
        match (taken.is_empty(), full) {
            (true, _) => self.say(format!("There's more in the {label}, but your backpack is full.")),
            (false, false) => self.say(format!("You search the {label}: {}.", taken.join(", "))),
            (false, true) => self.say(format!("You search the {label}: {}. Your backpack is full.", taken.join(", "))),
        }
    }

    pub(crate) fn use_vending(&mut self, idx: usize, secondary: bool) {
        if !self.player.has_backpack {
            self.say("You need the backpack to carry that.");
            return;
        }
        let (kind, label) = match (self.vending[idx].kind, secondary) {
            (VendingKind::Drinks, false) => (ItemKind::Refresco, "an energy drink"),
            (VendingKind::Drinks, true) => (ItemKind::Cafe, "a coffee"),
            (VendingKind::Snacks, false) => (ItemKind::Snack, "an energy bar"),
            (VendingKind::Snacks, true) => {
                self.say("This machine only has snacks.");
                return;
            }
        };
        if self.vending[idx].stock == 0 {
            self.say("It's empty.");
            return;
        }
        if self.player.inventory.add(Item::new(kind)).is_err() {
            self.say("Backpack full.");
            return;
        }
        self.vending[idx].stock -= 1;
        self.say(format!("You take {label}. {} left.", self.vending[idx].stock));
    }

    pub(crate) fn take_paper(&mut self, idx: usize) {
        let units = self.fixtures[idx].paper_units;
        if units == 0 {
            self.say("There's no paper left here.");
            return;
        }
        if !self.player.has_backpack {
            self.say("You need the backpack to carry that.");
            return;
        }
        // stack onto a roll you already carry
        if let Some(roll) = self.player.inventory.slots.iter_mut().flatten().find(|it| it.kind == ItemKind::ToiletPaper) {
            roll.units += units;
            self.fixtures[idx].paper_units = 0;
            self.say(format!("You take the paper roll (+{units} uses)."));
            return;
        }
        if self.player.inventory.add(Item::new_with_units(ItemKind::ToiletPaper, units)).is_err() {
            self.say("Inventory full.");
            return;
        }
        self.fixtures[idx].paper_units = 0;
        self.say(format!("You take the paper roll ({units} uses)."));
    }

    /// Z at a sink: fill the flask if it isn't full, otherwise wash.
    pub(crate) fn sink_secondary(&mut self) {
        if self.has_canteen_in_inventory() && self.player.canteen_fill < 100.0 {
            self.player.canteen_fill = 100.0;
            self.say("You fill the water flask.");
            return;
        }
        self.player.stats.hygiene = (self.player.stats.hygiene + 20.0).min(100.0);
        self.say("You wash up at the sink.");
    }

    pub(crate) fn use_fixture(&mut self, idx: usize) {
        match self.fixtures[idx].kind {
            FixtureKind::Sink => {
                if self.sink_drink_cooldown_ms > 0 {
                    self.say("You're not thirsty yet.");
                    return;
                }
                let s = &mut self.player.stats;
                s.thirst = (s.thirst - 45.0).max(0.0);
                self.sink_drink_cooldown_ms = 60_000;
                self.say("You drink from the tap.");
            }
            FixtureKind::Shower => {
                let s = &mut self.player.stats;
                s.hygiene = 100.0;
                s.thermal = (s.thermal + 6.0).min(100.0);
                self.say("You take a shower. Good as new.");
            }
            FixtureKind::Toilet => {
                if self.player_has_paper() {
                    self.consume_paper_unit();
                    self.player.stats.hygiene = (self.player.stats.hygiene + 8.0).min(100.0);
                    self.say("You use the toilet, and your own paper.");
                } else if self.fixtures[idx].paper_units > 0 {
                    self.fixtures[idx].paper_units -= 1;
                    self.player.stats.hygiene = (self.player.stats.hygiene + 8.0).min(100.0);
                    self.say("You use the toilet and some of its paper.");
                } else {
                    self.player.stats.hygiene = (self.player.stats.hygiene - 10.0).max(0.0);
                    self.say("You use the toilet. No paper left...");
                }
            }
            FixtureKind::Urinal => {
                self.player.stats.hygiene = (self.player.stats.hygiene - 6.0).max(0.0);
                self.say("You use the urinal.");
            }
        }
    }

    pub(crate) fn pick_dropped(&mut self, idx: usize) {
        if !self.player.has_backpack {
            self.say("You need the backpack to carry that.");
            return;
        }
        let item = self.dropped[idx].item.clone();
        let label = item.label();
        if self.player.inventory.add(item).is_ok() {
            self.dropped.remove(idx);
            self.say(format!("You pick up the {label}."));
        } else {
            self.say("Backpack full.");
        }
    }

    pub(crate) fn pick_equippable(&mut self, idx: usize) {
        let Equippable::Backpack(bp) = self.equippables.remove(idx);
        self.player.has_backpack = true;
        let mut spilled = 0;
        for item in bp.contents {
            if !self.give_item(item) {
                spilled += 1;
            }
        }
        if spilled > 0 {
            self.say(format!("You put the backpack on. {spilled} item(s) didn't fit and fall to the floor."));
        } else {
            self.say("You pick up your backpack. You can now carry up to 6 items.");
        }
    }

    pub(crate) fn pick_clothing(&mut self, idx: usize) {
        if self.player.has_backpack {
            let item = self.clothing[idx].item();
            let label = item.label();
            if self.player.inventory.add(item).is_err() {
                self.say("Backpack full.");
                return;
            }
            self.clothing[idx].taken = true;
            self.say(format!("You store the {label} in the backpack."));
            return;
        }
        let garment = self.clothing[idx].garment();
        self.clothing[idx].taken = true;
        let previous = self.player.wardrobe.wear(garment);
        self.sync_inventory_capacity();
        match previous.and_then(Garment::item) {
            Some(old) => {
                let old_label = old.label();
                self.drop_at_feet(old);
                self.say(format!("You put on the {} and leave the {old_label} on the floor.", garment.label()));
            }
            None => self.say(format!("You put on the {}.", garment.label())),
        }
    }

    // ------------------------------------------------------------ doors

    pub fn front_door_pos(&self) -> Option<(usize, usize)> {
        let fx = self.player.x + self.player.dir_x * 0.6;
        let fy = self.player.y + self.player.dir_y * 0.6;
        if fx < 0.0 || fy < 0.0 {
            return None;
        }
        let (tx, ty) = (fx as usize, fy as usize);
        self.map.get(tx, ty).is_any_door().then_some((tx, ty))
    }

    /// The key a door needs, if it's locked.
    pub fn door_lock(&self, tx: usize, ty: usize) -> Option<ItemKind> {
        self.doors.iter().find(|d| d.tx == tx && d.ty == ty).and_then(|d| d.lock)
    }

    pub(crate) fn toggle_door(&mut self, tx: usize, ty: usize) {
        let tile = self.map.get(tx, ty);
        let Some(open) = tile.door_open() else { return };
        if open && self.player.overlaps_tile(tx, ty) {
            self.say("You're standing in the doorway.");
            return;
        }
        if let Some(key) = self.door_lock(tx, ty)
            && !open
            && !self.player.inventory.has(key)
        {
            self.say(match key {
                ItemKind::ArchiveClearance => "[RESTRICTED: ARCHIVE CLEARANCE REQUIRED]",
                _ => "[SECTOR SEALED: TRIAGE KEYCARD REQUIRED]",
            });
            return;
        }
        // double entrance doors open and close as a pair
        let mut leaves = vec![(tx, ty)];
        if matches!(tile, Tile::MainDoor { .. }) {
            for (nx, ny) in [(tx + 1, ty), (tx.wrapping_sub(1), ty), (tx, ty + 1), (tx, ty.wrapping_sub(1))] {
                if matches!(self.map.get(nx, ny), Tile::MainDoor { .. }) {
                    leaves.push((nx, ny));
                }
            }
        }
        if open && leaves.iter().any(|&(x, y)| self.player.overlaps_tile(x, y)) {
            self.say("You're standing in the doorway.");
            return;
        }
        let now = self.now_ms;
        for (x, y) in leaves {
            let t = self.map.get(x, y);
            self.map.set(x, y, t.with_open(!open));
            if let Some(d) = self.doors.iter_mut().find(|d| d.tx == x && d.ty == y) {
                d.open_since_ms = (!open).then_some(now);
            }
        }
        let what = match tile {
            Tile::MainDoor { .. } => "the main door",
            Tile::BathroomDoor { .. } => "the restroom door",
            Tile::OperatingDoor { .. } => "the secure sector door",
            _ => "the door",
        };
        self.say(format!("You {} {what}.", if open { "close" } else { "open" }));
    }

    /// Self-closing doors shut once their time is up, but never on the player.
    fn auto_close_doors(&mut self) {
        let now = self.now_ms;
        for i in 0..self.doors.len() {
            let d = &self.doors[i];
            let (Some(t), Some(after)) = (d.open_since_ms, d.auto_close_ms) else { continue };
            if now.saturating_sub(t) < after || self.player.overlaps_tile(d.tx, d.ty) {
                continue;
            }
            let (tx, ty) = (d.tx, d.ty);
            let tile = self.map.get(tx, ty);
            self.map.set(tx, ty, tile.with_open(false));
            self.doors[i].open_since_ms = None;
        }
    }
}

pub(crate) fn dist(ax: f64, ay: f64, bx: f64, by: f64) -> f64 {
    ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt()
}

/// A seat box and the backrest behind it.
fn seat_and_back(seat: BoxKind, back: BoxKind, x: f64, y: f64, f: crate::engine::boxes::Facing) -> [BoxInput; 2] {
    let (fx, fy) = f.vector();
    let off = seat.size().1 / 2.0 + back.size().1 / 2.0;
    [BoxInput::new(seat, x, y, f), BoxInput::new(back, x - fx * off, y - fy * off, f)]
}

/// A Project Zomboid style status: shows up only when something needs attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Moodle {
    pub label: &'static str,
    /// 1 (mild) to 4 (critical).
    pub level: u8,
}

impl World {
    /// What's wrong right now, worst first.
    pub fn moodles(&self) -> Vec<Moodle> {
        let s = &self.player.stats;
        let up = |v: f64, t: [f64; 4]| t.iter().filter(|&&x| v >= x).count() as u8;
        let down = |v: f64, t: [f64; 4]| t.iter().filter(|&&x| v <= x).count() as u8;
        let mut out = Vec::new();
        let mut add = |level: u8, labels: [&'static str; 4]| {
            if level > 0 {
                out.push(Moodle { label: labels[level as usize - 1], level });
            }
        };
        add(up(s.thirst, [45.0, 65.0, 80.0, 92.0]), ["Thirsty", "Very thirsty", "Parched", "Dying of thirst"]);
        add(up(s.hunger, [45.0, 65.0, 80.0, 92.0]), ["Peckish", "Hungry", "Very hungry", "Starving"]);
        add(up(s.sleep, [55.0, 70.0, 82.0, 92.0]), ["Drowsy", "Tired", "Very tired", "Exhausted"]);
        add(down(s.hygiene, [40.0, 25.0, 12.0, 4.0]), ["Unwashed", "Dirty", "Filthy", "Disgusting"]);
        add(down(s.body, [55.0, 35.0, 20.0, 8.0]), ["Sore", "Weak", "Injured", "Critical"]);
        add(down(s.mind, [55.0, 35.0, 20.0, 8.0]), ["Uneasy", "Anxious", "Panicky", "Breaking down"]);
        add(down(s.stamina, [35.0, 20.0, 8.0, 2.0]), ["Winded", "Out of breath", "Exhausted", "Collapsing"]);
        add(up(s.thermal.abs(), [40.0, 60.0, 80.0, 95.0]), if s.thermal >= 0.0 { ["Warm", "Hot", "Overheating", "Heatstroke"] } else { ["Chilly", "Cold", "Freezing", "Hypothermic"] });
        out.sort_by_key(|m| std::cmp::Reverse(m.level));
        out
    }

    /// Every light: the ceiling tubes that still work, windows, street lamps, glows.
    fn light_sources(&self) -> Vec<LightSource> {
        let mut v = Vec::new();
        for (idx, f) in self.fluorescents.iter().enumerate() {
            let flickers = match f.state {
                FluorescentState::Dead => continue,
                FluorescentState::Steady => false,
                FluorescentState::Flicker { .. } => true,
            };
            v.push(LightSource { x: f.cx, y: f.cy, kind: LightKind::Tube { idx, flickers }, color: [0.98, 1.0, 0.95], intensity: 0.6, radius: 6.5 });
        }
        v.extend(self.lights.iter().copied());
        v
    }

    /// The precomputed light, built the first time something is drawn.
    pub fn lightmap(&self) -> &LightMap {
        self.lightmap.get_or_init(|| LightMap::build(&self.map, self.light_sources()))
    }
}
