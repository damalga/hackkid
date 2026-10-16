//! FANFARE in the terminal: ratatui for the screen, rodio for the sound, the save on disk.
//! The game itself lives in `hackkid-core`.

mod audio;
mod input;
mod quiet;
mod saves;
mod settings;
mod ui;

use std::io::{self, stdout};
use std::time::{Duration, Instant};

use hackkid_core::engine::renderer::Viewport;
use hackkid_core::game::action::{Action, dispatch};
use hackkid_core::game::controls::ControlScheme;
use hackkid_core::game::net::Screen;
use hackkid_core::game::world::{GameMode, MusicSource, Request, World};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::Rect;

/// One frame every 50 ms keeps the flickering lights and the clock moving; ratatui
/// only sends the cells that changed.
const TICK: Duration = Duration::from_millis(50);

fn main() -> io::Result<()> {
    // audio libraries write warnings to stderr, which is the screen: send them to a log
    quiet::stderr_to_log(&saves::log_path());
    // ratatui restores the terminal on exit and on panic; this also gives the mouse and
    // stderr back first, so a crash report is readable
    let terminal = ratatui::try_init()?;
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        quiet::restore_stderr();
        let _ = execute!(stdout(), DisableMouseCapture);
        hook(info);
    }));
    let result = execute!(stdout(), EnableMouseCapture).and_then(|()| App::new().run(terminal));
    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    quiet::restore_stderr();
    result
}

// ---------------------------------------------------------------- the mouse

/// How far one column of pointer movement turns you, in milliradians (≈ 0.6°): a
/// sweep across a 120-column view is about 70°.
const LOOK_MRAD: i32 = 10;
/// ...and one row of pointer movement tilts the view (a row is about two columns tall).
const PITCH_MRAD: i32 = 18;
/// The wheel, or two fingers on a trackpad, also look round: a sideways notch turns, an
/// up or down notch tilts. Scrolling has no edge, so this is the way round the full 360°.
/// A trackpad reports a notch about every line of travel, so a notch is sized like one
/// cell of pointer movement: two fingers then feel like one.
const WHEEL_TURN_MRAD: i32 = 24;
const WHEEL_PITCH_MRAD: i32 = 20;
/// ...and however fast the notches come, scrolling turns you at most this much per
/// 100 ms (about 15°), so a flick can't spin you.
const SCROLL_CAP_MRAD: i32 = 260;
/// The outer part of the view where parking the pointer keeps you turning, as a share
/// of the view's width, and how fast at the very edge.
const EDGE_ZONE: f32 = 0.12;
const EDGE_RATE: f64 = 1.4;
/// How long parking the pointer at an edge keeps you turning before it needs a nudge,
/// so leaving the window through an edge doesn't spin you for ever.
const EDGE_BUDGET_MS: i64 = 2_600;
/// A pointer that hasn't moved for this long starts a fresh gesture: no jump when it
/// comes back in somewhere else.
const GESTURE_GAP_MS: u64 = 400;
/// Holding the left button longer than this walks you forward (a shorter press is a click).
const HOLD_TO_WALK_MS: u64 = 180;
/// In the Keys scheme a vertical wheel notch (two fingers up or down) is one step
/// forward or back: held this long so a stream of notches walks you smoothly.
const WHEEL_STEP_HOLD_MS: u16 = 220;

/// The pointer, as far as a terminal lets us know it: where it was last seen, whether a
/// button is down, and whether this press has turned into a drag.
#[derive(Default)]
struct Mouse {
    pos: Option<(u16, u16)>,
    last_motion_ms: u64,
    held: Option<MouseButton>,
    held_since_ms: u64,
    dragged: bool,
    edge_budget_ms: i64,
    /// How much scrolling has turned or tilted the view in the current 100 ms window.
    scroll_window_ms: u64,
    scroll_spent_mrad: i32,
}

impl Mouse {
    /// How much of a wanted scroll turn to apply now, within the per-window cap.
    fn scroll_allow(&mut self, now_ms: u64, want: i32) -> i32 {
        if now_ms.saturating_sub(self.scroll_window_ms) >= 100 {
            self.scroll_window_ms = now_ms;
            self.scroll_spent_mrad = 0;
        }
        let room = (SCROLL_CAP_MRAD - self.scroll_spent_mrad).max(0);
        let apply = want.clamp(-room, room);
        self.scroll_spent_mrad += apply.abs();
        apply
    }
}

struct App {
    world: World,
    audio: Option<audio::Audio>,
    viewport: Viewport,
    /// The big map (M) is open.
    show_map: bool,
    mouse: Mouse,
    keys: input::KeyRepeat,
    /// Where things were drawn last frame, for the pointer.
    hits: ui::Hits,
    told_silent: bool,
    start: Instant,
    last_frame_ms: u64,
    quit: bool,
    /// The control scheme as last written to the settings file.
    controls_saved: ControlScheme,
}

impl App {
    fn new() -> Self {
        // every game rolls its own loot
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        let mut app = Self {
            world: World::with_seed(seed),
            audio: audio::Audio::start(),
            viewport: Viewport::default(),
            show_map: false,
            mouse: Mouse::default(),
            keys: input::KeyRepeat::default(),
            hits: ui::Hits::default(),
            told_silent: false,
            start: Instant::now(),
            last_frame_ms: 0,
            quit: false,
            controls_saved: ControlScheme::Mouse,
        };
        if let Some(scheme) = settings::read_controls() {
            app.world.controls = scheme;
            app.controls_saved = scheme;
        }
        app
    }

    /// Writes the control scheme out when it changes, so it's there next time.
    fn remember_controls(&mut self) {
        let scheme = self.world.controls;
        if scheme != self.controls_saved {
            self.controls_saved = scheme;
            if let Err(e) = settings::write_controls(scheme) {
                self.world.say(format!("Couldn't save the settings: {e}"));
            }
        }
    }

    fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            let now = self.now_ms();
            let dt = now.saturating_sub(self.last_frame_ms);
            self.last_frame_ms = now;
            self.edge_turn(dt);
            self.hold_to_walk(now);
            if let Some(key) = self.keys.take_double_tap(now)
                && let Some(action) = input::key_to_action(key, self.world.mode, input::LOCK_HOLD_MS, self.world.controls)
                && matches!(action, Action::Move { .. })
            {
                dispatch(&mut self.world, action);
                self.world.say("Walking on. Tap a movement key to stop.");
            }
            self.world.update(now);
            self.handle_requests();
            self.remember_controls();
            self.sync_audio();
            if !matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory) {
                self.show_map = false;
            }
            let hover = self.mouse.pos;
            terminal.draw(|f| ui::draw(f, &self.world, &mut self.viewport, now, self.show_map, hover))?;
            self.hits = ui::last_hits();
            if event::poll(TICK)? {
                // handle everything queued up (key repeat can outpace frames), then draw once
                loop {
                    self.handle_event(event::read()?);
                    if self.quit || !event::poll(Duration::ZERO)? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: Event) {
        match ev {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    self.quit = true;
                } else if matches!(key.code, KeyCode::Char('m' | 'M')) && matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory) {
                    self.toggle_map();
                } else if self.show_map && key.code == KeyCode::Esc {
                    self.show_map = false;
                } else if self.world.mode == GameMode::Laptop && self.world.laptop.screen == Screen::Net {
                    if let Some(action) = input::net_key(key) {
                        dispatch(&mut self.world, action);
                    }
                } else {
                    let hold = self.keys.hold_for(key, self.now_ms());
                    if let Some(action) = input::key_to_action(key, self.world.mode, hold, self.world.controls) {
                        dispatch(&mut self.world, action);
                    }
                }
            }
            Event::Mouse(m) => self.handle_mouse(m),
            Event::FocusLost => {
                // nobody is at the keys: stop walking, and forget where the pointer was
                dispatch(&mut self.world, Action::Halt);
                self.mouse = Mouse::default();
            }
            // resizes are picked up by the next draw
            _ => {}
        }
    }

    /// Where the pointer is, in terms of what's drawn there.
    fn over(&self, col: u16, row: u16) -> Over {
        let inside = |r: &Rect| col >= r.x && col < r.right() && row >= r.y && row < r.bottom();
        if let Some(&(_, i)) = self.hits.slots.iter().find(|(r, _)| inside(r)) {
            return Over::Slot(i);
        }
        if let Some(&(_, i)) = self.hits.menu.iter().find(|(r, _)| inside(r)) {
            return Over::MenuItem(i);
        }
        if let Some(&(_, i)) = self.hits.tracks.iter().find(|(r, _)| inside(r)) {
            return Over::Track(i);
        }
        if let Some(&(_, i)) = self.hits.tabs.iter().find(|(r, _)| inside(r)) {
            return Over::Tab(i);
        }
        if self.hits.panels.iter().any(inside) {
            return Over::Panel;
        }
        if inside(&self.hits.view) {
            return Over::View;
        }
        Over::Nothing
    }

    fn handle_mouse(&mut self, m: MouseEvent) {
        let now = self.now_ms();
        let (col, row) = (m.column, m.row);
        let over = self.over(col, row);
        match m.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => {
                // hovering a menu or a list picks the item under the pointer
                match (over, self.world.mode) {
                    (Over::MenuItem(i), GameMode::Menu | GameMode::Startup) => dispatch(&mut self.world, Action::MenuPick(i)),
                    (Over::Track(i), GameMode::Laptop) => dispatch(&mut self.world, Action::LaptopPick(i)),
                    _ => {}
                }
                // the pointer only looks round in the Mouse look scheme
                let looking = self.world.controls == ControlScheme::Mouse
                    && matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory | GameMode::Sitting)
                    && !self.show_map;
                let gesture = now.saturating_sub(self.mouse.last_motion_ms) < GESTURE_GAP_MS;
                if let Some((px, py)) = self.mouse.pos
                    && looking
                    && gesture
                    && over == Over::View
                {
                    let cols = col as i32 - px as i32;
                    if cols != 0 {
                        dispatch(&mut self.world, Action::Turn(cols * LOOK_MRAD));
                        self.mouse.edge_budget_ms = EDGE_BUDGET_MS;
                    }
                    // up the screen is up: rows count downwards
                    let rows = py as i32 - row as i32;
                    if rows != 0 {
                        dispatch(&mut self.world, Action::Pitch(rows * PITCH_MRAD));
                    }
                }
                if self.mouse.pos != Some((col, row)) {
                    self.mouse.edge_budget_ms = EDGE_BUDGET_MS;
                }
                if matches!(m.kind, MouseEventKind::Drag(_)) {
                    self.mouse.dragged = true;
                }
                self.mouse.pos = Some((col, row));
                self.mouse.last_motion_ms = now;
            }
            MouseEventKind::Down(b) => {
                self.mouse.held = Some(b);
                self.mouse.held_since_ms = now;
                self.mouse.dragged = false;
                self.mouse.pos = Some((col, row));
                self.mouse.last_motion_ms = now;
            }
            MouseEventKind::Up(b) => {
                let walked = b == MouseButton::Left && now.saturating_sub(self.mouse.held_since_ms) > HOLD_TO_WALK_MS && over == Over::View;
                let click = self.mouse.held == Some(b) && !self.mouse.dragged && !walked;
                self.mouse.held = None;
                if click {
                    self.click(b, over);
                }
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let up = m.kind == MouseEventKind::ScrollUp;
                self.mouse.pos = Some((col, row));
                self.scroll(up, over, now);
            }
            MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight => {
                // two fingers sideways on a trackpad turns you, as far round as you like
                if matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory | GameMode::Sitting) {
                    let dir = if m.kind == MouseEventKind::ScrollRight { 1 } else { -1 };
                    let mrad = self.mouse.scroll_allow(now, dir * WHEEL_TURN_MRAD);
                    if mrad != 0 {
                        dispatch(&mut self.world, Action::Turn(mrad));
                    }
                }
            }
        }
    }

    /// Left click uses, right click does the second thing, the middle button is the light.
    fn click(&mut self, b: MouseButton, over: Over) {
        let mode = self.world.mode;
        let action = match (mode, over, b) {
            (GameMode::Startup | GameMode::Menu, Over::MenuItem(i), MouseButton::Left) => {
                dispatch(&mut self.world, Action::MenuPick(i));
                Some(Action::MenuSelect)
            }
            (GameMode::Menu, _, MouseButton::Right) => Some(Action::CloseMenu),
            (GameMode::InitialWake, _, _) => Some(Action::Wake),
            (GameMode::Dead, _, _) => Some(Action::Restart),
            (GameMode::Inventory, Over::Slot(i), MouseButton::Left) => {
                if self.world.player.inventory_cursor == i {
                    Some(Action::InventoryUseSelected)
                } else {
                    Some(Action::InventoryPick(i))
                }
            }
            (GameMode::Inventory, Over::Slot(i), MouseButton::Right) => {
                dispatch(&mut self.world, Action::InventoryPick(i));
                Some(Action::InventoryDropSelected)
            }
            (GameMode::Laptop, Over::Track(i), MouseButton::Left) => {
                if self.world.laptop.cursor == i {
                    Some(Action::LaptopTogglePlay)
                } else {
                    Some(Action::LaptopPick(i))
                }
            }
            (GameMode::Laptop, Over::Tab(i), MouseButton::Left) => {
                let on_net = self.world.laptop.screen == Screen::Net;
                (on_net != (i == 1)).then_some(Action::LaptopSwitch)
            }
            (GameMode::Laptop, _, MouseButton::Right) => Some(Action::LaptopClose),
            (GameMode::Exploring | GameMode::Inventory, Over::View, MouseButton::Left) => {
                if self.show_map {
                    self.show_map = false;
                    None
                } else {
                    Some(Action::Interact)
                }
            }
            (GameMode::Exploring | GameMode::Inventory, Over::View, MouseButton::Right) => Some(Action::Secondary),
            (GameMode::Exploring | GameMode::Inventory | GameMode::Sitting, _, MouseButton::Middle) => Some(Action::ToggleFlashlight),
            (GameMode::Sitting | GameMode::Lying, Over::View, MouseButton::Left) => Some(Action::StandUp),
            (GameMode::Lying, Over::View, MouseButton::Right) => Some(Action::Sleep),
            _ => None,
        };
        if let Some(a) = action {
            dispatch(&mut self.world, a);
        }
    }

    /// The wheel (or two fingers up and down on a trackpad): tilts the view, scrolls lists.
    fn scroll(&mut self, up: bool, over: Over, now: u64) {
        let action = match (self.world.mode, over) {
            (GameMode::Startup | GameMode::Menu, _) => Some(if up { Action::MenuUp } else { Action::MenuDown }),
            (GameMode::Laptop, _) if self.world.laptop.screen == Screen::Net => Some(Action::LaptopPage(if up { -1 } else { 1 })),
            (GameMode::Laptop, _) => Some(if up { Action::LaptopUp } else { Action::LaptopDown }),
            (GameMode::Inventory, Over::Slot(_) | Over::Panel) => Some(if up { Action::InventoryUp } else { Action::InventoryDown }),
            // Keys scheme: a notch up is a step forward, a notch down a step back
            (GameMode::Exploring | GameMode::Inventory, _) if self.world.controls == ControlScheme::Keys => {
                Some(Action::Move { forward: if up { 1 } else { -1 }, side: 0, run: false, hold_ms: WHEEL_STEP_HOLD_MS })
            }
            (GameMode::Exploring | GameMode::Inventory | GameMode::Sitting, _) => {
                let mrad = self.mouse.scroll_allow(now, if up { WHEEL_PITCH_MRAD } else { -WHEEL_PITCH_MRAD });
                (mrad != 0).then_some(Action::Pitch(mrad))
            }
            _ => None,
        };
        if let Some(a) = action {
            dispatch(&mut self.world, a);
        }
    }

    /// Holding the left button down over the view walks you forward, while the pointer
    /// (or two fingers) looks around: walking and looking on one trackpad.
    fn hold_to_walk(&mut self, now: u64) {
        if self.mouse.held != Some(MouseButton::Left) || now.saturating_sub(self.mouse.held_since_ms) < HOLD_TO_WALK_MS {
            return;
        }
        let Some((col, row)) = self.mouse.pos else { return };
        if self.over(col, row) == Over::View && matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory) && !self.show_map {
            dispatch(&mut self.world, Action::Move { forward: 1, side: 0, run: false, hold_ms: 120 });
        }
    }

    /// Parking the pointer near the left or right edge of the view keeps you turning that
    /// way, faster the nearer the edge, until the budget runs out or the pointer moves.
    fn edge_turn(&mut self, dt_ms: u64) {
        let Some((col, row)) = self.mouse.pos else { return };
        if self.show_map
            || self.world.controls != ControlScheme::Mouse
            || self.mouse.edge_budget_ms <= 0
            || !matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory | GameMode::Sitting)
        {
            return;
        }
        if self.over(col, row) != Over::View {
            return;
        }
        let v = self.hits.view;
        let zone = (v.width as f32 * EDGE_ZONE).max(2.0);
        let x = (col - v.x) as f32 + 0.5;
        let depth = if x < zone {
            -(1.0 - x / zone)
        } else if x > v.width as f32 - zone {
            1.0 - (v.width as f32 - x) / zone
        } else {
            return;
        };
        self.mouse.edge_budget_ms -= dt_ms as i64;
        let rate = depth.signum() * depth.abs().powf(1.5) * EDGE_RATE as f32;
        let mrad = (rate as f64 * dt_ms as f64) as i32;
        if mrad != 0 {
            dispatch(&mut self.world, Action::Turn(mrad));
        }
    }

    fn toggle_map(&mut self) {
        if self.world.player.has_map {
            self.show_map = !self.show_map;
        } else {
            self.world.say("You don't have a map of the hospital yet.");
        }
    }

    fn toggle_mute(&mut self) {
        match &self.audio {
            Some(a) => {
                let muted = a.toggle_mute();
                self.world.say(if muted { "Sound off." } else { "Sound on." });
            }
            None => self.world.say("No sound device found: the game runs silently."),
        }
    }

    fn handle_requests(&mut self) {
        for req in self.world.take_requests() {
            match req {
                Request::Quit => self.quit = true,
                Request::ToggleSound => self.toggle_mute(),
                Request::Save => match saves::write(&self.world.save_json()) {
                    Ok(path) => self.world.say(format!("Game saved to {}", path.display())),
                    Err(e) => self.world.say(format!("Couldn't save: {e}")),
                },
                Request::Load => match saves::read() {
                    Ok(json) => match self.world.load_json(&json) {
                        Ok(()) => self.world.say("Game loaded."),
                        Err(why) => self.world.say(why),
                    },
                    Err(e) if e.kind() == io::ErrorKind::NotFound => self.world.say("No saved game yet."),
                    Err(e) => self.world.say(format!("Couldn't read the save: {e}")),
                },
            }
        }
    }

    fn sync_audio(&mut self) {
        match &mut self.audio {
            Some(a) => a.sync(&mut self.world),
            None => {
                // no sound device: the terminal still works, in silence
                if let Some(idx) = self.world.music()
                    && self.world.laptop.source.is_none()
                {
                    self.world.music_started(idx, MusicSource::Silent);
                    if !self.told_silent {
                        self.told_silent = true;
                        self.world.say("No sound device found: the transmissions play silently.");
                    }
                }
            }
        }
    }
}

/// What's under the pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Over {
    View,
    Panel,
    Slot(usize),
    MenuItem(usize),
    Track(usize),
    Tab(usize),
    Nothing,
}
