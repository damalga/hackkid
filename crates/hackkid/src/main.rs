//! FANFARE in the terminal: ratatui for the screen, rodio for the sound, the save on disk.
//! The game itself lives in `hackkid-core`.

mod audio;
mod input;
mod quiet;
mod saves;
mod ui;

use std::io::{self, stdout};
use std::time::{Duration, Instant};

use hackkid_core::engine::renderer::Viewport;
use hackkid_core::game::action::{Action, dispatch};
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

/// The wheel, or two fingers on a trackpad, sideways turns you (full 360°, no edge).
/// Vertical wheel notches run you forward or walk you back — one notch is one step.
/// Pointer movement does not turn or tilt the view: A, D, Z and X do that.
const WHEEL_TURN_MRAD: i32 = 24;
/// How long one vertical wheel notch keeps you running (or stepping back): long enough
/// that a stream of notches feels like a held key, short enough that a single notch is
/// one step.
const WHEEL_STEP_HOLD_MS: u16 = 220;
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
/// Holding the left button longer than this walks you forward (a shorter press is a click).
const HOLD_TO_WALK_MS: u64 = 180;

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
    /// Chain of consecutive left or right triple-taps: direction (-1 or 1), how many
    /// in a row, when the last one landed. Each one spins a step further.
    spin_chain: Option<(i8, u8, u64)>,
}

impl App {
    fn new() -> Self {
        // every game rolls its own loot
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        Self {
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
            spin_chain: None,
        }
    }

    fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        let result = self.run_loop(&mut terminal);
        // explicitly tear the audio stream down before main gives the terminal back, so
        // a quit (Ctrl+C or otherwise) cuts the sound at once instead of letting rodio
        // keep playing from its own thread while the process winds down.
        self.audio.take();
        result
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            let now = self.now_ms();
            let dt = now.saturating_sub(self.last_frame_ms);
            self.last_frame_ms = now;
            self.edge_turn(dt);
            self.hold_to_walk(now);
            if let Some((key, count)) = self.keys.take_tap_sequence(now) {
                let back = matches!(key.code, KeyCode::Char('s' | 'S') | KeyCode::Down);
                let left = matches!(key.code, KeyCode::Char('a' | 'A') | KeyCode::Left);
                let right = matches!(key.code, KeyCode::Char('d' | 'D') | KeyCode::Right);
                let exploring = matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory);
                // Animated spins so you see the sweep: three taps backwards swing you
                // 180°, three taps left or right spin you a step (30°), and each next
                // triple-tap in the same direction within the chain window adds another
                // step. A pause resets the chain so a stray tap turns the usual amount.
                const SPIN_STEP_MRAD: i32 = ((std::f64::consts::PI / 6.0) * 1000.0) as i32;
                const SPIN_CHAIN_WINDOW_MS: u64 = 1_500;
                const SPIN_CHAIN_MAX: u8 = 6;
                let dir = if right { 1_i8 } else if left { -1_i8 } else { 0 };
                let turn_spin = count >= 3 && dir != 0 && exploring;
                let back_spin = count >= 3 && back && exploring;
                if turn_spin {
                    let step = match self.spin_chain {
                        Some((d, n, at)) if d == dir && now.saturating_sub(at) < SPIN_CHAIN_WINDOW_MS => n.saturating_add(1).min(SPIN_CHAIN_MAX),
                        _ => 1,
                    };
                    self.spin_chain = Some((dir, step, now));
                    dispatch(&mut self.world, Action::Spin(SPIN_STEP_MRAD * step as i32 * dir as i32));
                } else if back_spin {
                    self.spin_chain = None;
                    dispatch(&mut self.world, Action::Spin((std::f64::consts::PI * 1000.0) as i32));
                } else if count == 2
                    && let Some(action) = input::key_to_action(key, self.world.mode, input::LOCK_HOLD_MS)
                    && matches!(action, Action::Move { .. })
                {
                    self.spin_chain = None;
                    dispatch(&mut self.world, action);
                    self.world.say("Walking on. Tap a movement key to stop.");
                }
            }
            self.world.update(now);
            self.handle_requests();
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
                } else {
                    let hold = self.keys.hold_for(key, self.now_ms());
                    if let Some(action) = input::key_to_action(key, self.world.mode, hold) {
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

    /// The wheel (or two fingers up and down on a trackpad): wheel-up runs you forward
    /// one step, wheel-down walks you back one step. Over lists it scrolls them.
    /// Looking up and down is Z and X on the keyboard, not the wheel.
    fn scroll(&mut self, up: bool, over: Over, _now: u64) {
        let action = match (self.world.mode, over) {
            (GameMode::Startup | GameMode::Menu, _) => Some(if up { Action::MenuUp } else { Action::MenuDown }),
            (GameMode::Laptop, _) => Some(if up { Action::LaptopUp } else { Action::LaptopDown }),
            (GameMode::Inventory, Over::Slot(_) | Over::Panel) => Some(if up { Action::InventoryUp } else { Action::InventoryDown }),
            (GameMode::Exploring | GameMode::Inventory, _) => Some(Action::Move {
                forward: if up { 1 } else { -1 },
                side: 0,
                run: up,
                hold_ms: WHEEL_STEP_HOLD_MS,
            }),
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
        if self.show_map || self.mouse.edge_budget_ms <= 0 || !matches!(self.world.mode, GameMode::Exploring | GameMode::Inventory | GameMode::Sitting) {
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
    Nothing,
}
