//! FANFARE in the terminal: ratatui for the screen, rodio for the sound, the save on disk.
//! The game itself lives in `hackkid-core`.

mod audio;
mod input;
mod saves;
mod ui;

use std::io::{self, stdout};
use std::time::{Duration, Instant};

use hackkid_core::engine::renderer::Viewport;
use hackkid_core::game::action::dispatch;
use hackkid_core::game::world::{Request, World};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;

/// One frame every 50 ms keeps the flickering lights and the clock moving; ratatui
/// only sends the cells that changed.
const TICK: Duration = Duration::from_millis(50);

fn main() -> io::Result<()> {
    // ratatui restores the terminal on exit and on panic; this also gives the mouse back
    let terminal = ratatui::try_init()?;
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), DisableMouseCapture);
        hook(info);
    }));
    let result = execute!(stdout(), EnableMouseCapture).and_then(|()| App::new().run(terminal));
    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    result
}

struct App {
    world: World,
    audio: Option<audio::Audio>,
    viewport: Viewport,
    full_hud: bool,
    start: Instant,
    quit: bool,
}

impl App {
    fn new() -> Self {
        // every game rolls its own loot
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64);
        Self { world: World::with_seed(seed), audio: audio::Audio::start(), viewport: Viewport::default(), full_hud: false, start: Instant::now(), quit: false }
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
            terminal.draw(|f| ui::draw(f, &self.world, &mut self.viewport, now, self.full_hud))?;
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
                } else if matches!(key.code, KeyCode::Char('m' | 'M')) && key.modifiers.is_empty() {
                    self.toggle_mute();
                } else if key.code == KeyCode::Tab {
                    self.full_hud = !self.full_hud;
                } else if let Some(action) = input::key_to_action(key, self.world.mode) {
                    dispatch(&mut self.world, action);
                }
            }
            Event::Mouse(m) => {
                if let Some(action) = input::mouse_to_action(m, self.world.mode) {
                    dispatch(&mut self.world, action);
                }
            }
            // resizes are picked up by the next draw
            _ => {}
        }
    }

    fn toggle_mute(&mut self) {
        match &self.audio {
            Some(a) => {
                let muted = a.toggle_mute();
                self.world.say(if muted { "Audio muted." } else { "Audio unmuted." });
            }
            None => self.world.say("No audio device found."),
        }
    }

    fn handle_requests(&mut self) {
        for req in self.world.take_requests() {
            match req {
                Request::Quit => self.quit = true,
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
                if let Some(idx) = self.world.music() {
                    self.world.music_unavailable(idx, "No audio device found.");
                }
            }
        }
    }
}
