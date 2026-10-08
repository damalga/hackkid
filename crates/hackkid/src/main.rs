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
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::crossterm::execute;

/// One frame every 50 ms keeps the flickering lights and the clock moving; ratatui
/// only sends the cells that changed.
const TICK: Duration = Duration::from_millis(50);
/// How far a one-column mouse drag turns you, in milliradians.
const DRAG_TURN: i32 = 45;

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

struct App {
    world: World,
    audio: Option<audio::Audio>,
    viewport: Viewport,
    /// The big map (M) is open.
    show_map: bool,
    /// Where the last mouse-drag event was, to turn by the difference.
    drag_from: Option<u16>,
    told_silent: bool,
    start: Instant,
    quit: bool,
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
            drag_from: None,
            told_silent: false,
            start: Instant::now(),
            quit: false,
        }
    }

    fn now_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            let now = self.now_ms();
            self.world.update(now);
            self.handle_requests();
            self.sync_audio();
            if self.world.mode != GameMode::Exploring {
                self.show_map = false;
            }
            terminal.draw(|f| ui::draw(f, &self.world, &mut self.viewport, now, self.show_map))?;
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
                } else if matches!(key.code, KeyCode::Char('m' | 'M')) && self.world.mode == GameMode::Exploring {
                    self.toggle_map();
                } else if self.show_map && key.code == KeyCode::Esc {
                    self.show_map = false;
                } else if let Some(action) = input::key_to_action(key, self.world.mode) {
                    dispatch(&mut self.world, action);
                }
            }
            // drag with the left button to look around
            Event::Mouse(m) => match m.kind {
                MouseEventKind::Down(MouseButton::Left) => self.drag_from = Some(m.column),
                MouseEventKind::Drag(MouseButton::Left) => {
                    if let Some(from) = self.drag_from.replace(m.column)
                        && self.world.mode == GameMode::Exploring
                    {
                        let cols = m.column as i32 - from as i32;
                        if cols != 0 {
                            dispatch(&mut self.world, Action::Turn(cols * DRAG_TURN));
                        }
                    }
                }
                MouseEventKind::Up(_) => self.drag_from = None,
                _ => {}
            },
            // resizes are picked up by the next draw
            _ => {}
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
