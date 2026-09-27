mod audio;
mod engine;
mod equippables;
mod game;
mod objects;
mod ui;

use engine::{raycaster, renderer::Renderer};
use game::sprites::collect_sprites;
use game::action::{dispatch, Action};
use game::input::{key_to_action, mouse_to_action};
use game::world::{GameMode, World};
use ui::ceiling::render_ceiling;
use ui::frame::{blit_frame, render_top_border, OVERLAY_ROW, RENDER_START};
use ui::hud::render_top_status;
use ui::hud_area::render_hud_area;
use ui::laptop::render_laptop_view;
use ui::menu::{render_main_menu, render_startup_menu};
use ui::overlay::{render_action_prompt, render_message_overlay, render_styled_message_overlay, OverlayStyle};
use ui::prompt::build_action_label;

use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    style::ResetColor,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write, stdout};
use std::time::{Duration, Instant};

const HUD_OVERHEAD: usize = 17;

fn handle_audio_toggle(key: &crossterm::event::KeyEvent, audio: &Option<audio::AudioHandle>, world: &mut World) -> bool {
    if matches!(key.code, KeyCode::Char('m') | KeyCode::Char('M')) {
        if let Some(a) = audio {
            let now_muted = a.state.toggle_mute();
            let msg = if now_muted { "Música apagada.".to_string() } else { "Música encendida.".to_string() };
            world.set_message(msg, 10);
        }
        return true;
    }
    false
}

fn main() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture, cursor::Hide, terminal::Clear(ClearType::All))?;

    let (term_w, term_h) = terminal::size()?;
    let view_w = term_w as usize;
    let render_rows = (term_h as usize).saturating_sub(HUD_OVERHEAD);
    let pixel_h = render_rows * 2;
    let hud_start = RENDER_START + render_rows as u16 + 1;

    let renderer = Renderer::new(view_w, pixel_h);
    let mut world = World::new();
    let audio = audio::AudioHandle::start();
    let start = Instant::now();
    let frame_gap = Duration::from_millis(90);

    let rest_tick = Duration::from_millis(400);

    'game: loop {
        if world.should_exit { break 'game; }
        let elapsed_ms = Instant::now().duration_since(start).as_millis() as u64;
        world.process_pending(elapsed_ms);
        world.advance_realtime(elapsed_ms);

        if let Some(a) = &audio {
            let px = world.player.x as usize;
            let py = world.player.y as usize;
            let indoor = !world.map.get(px, py).is_outdoor();
            let dialog = world.active_message().is_some();
            a.state.write(
                world.hour,
                indoor,
                world.player.stats.body,
                world.player.stats.thirst,
                world.player.stats.sleep,
                dialog,
            );
        }

        let weather_label = if world.weather_seen {
            match world.weather {
                game::world::Weather::Clear => "Despejado",
                game::world::Weather::Cloudy => "Nublado",
            }
        } else { "" };
        let top_status = render_top_status(world.day, world.hour, world.player.hidden, weather_label);
        render_top_border(&mut out, view_w, &top_status)?;

        if world.mode == GameMode::Startup {
            for row in RENDER_START..(RENDER_START + render_rows as u16) {
                execute!(out,
                    cursor::MoveTo(0, row),
                    ResetColor,
                    terminal::Clear(ClearType::CurrentLine),
                )?;
            }
            render_startup_menu(&mut out, view_w, render_rows, world.menu_cursor)?;
            out.flush()?;
            if event::poll(std::time::Duration::from_millis(200))? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break 'game;
                    }
                    if !handle_audio_toggle(&key, &audio, &mut world) {
                        if let Some(action) = key_to_action(key, &world.mode) {
                            dispatch(&mut world, action);
                        }
                    }
                }
            }
            continue;
        }

        if world.mode == GameMode::Dead {
            for row in RENDER_START..(RENDER_START + render_rows as u16) {
                execute!(out,
                    cursor::MoveTo(0, row),
                    ResetColor,
                    terminal::Clear(ClearType::CurrentLine),
                )?;
            }
            let mid_row = RENDER_START + (render_rows as u16) / 2;
            render_message_overlay(&mut out, view_w, mid_row.saturating_sub(2), "Has muerto.", true)?;
            render_message_overlay(&mut out, view_w, mid_row + 2, "Pulsa Enter para volver al menú.", false)?;
            out.flush()?;
            if event::poll(std::time::Duration::from_millis(200))? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break 'game;
                    }
                    if !handle_audio_toggle(&key, &audio, &mut world) {
                        if let Some(action) = key_to_action(key, &world.mode) {
                            dispatch(&mut world, action);
                        }
                    }
                }
            }
            continue;
        }

        if world.mode == GameMode::Sleeping {
            for row in RENDER_START..(RENDER_START + render_rows as u16) {
                execute!(out,
                    cursor::MoveTo(0, row),
                    ResetColor,
                    terminal::Clear(ClearType::CurrentLine),
                )?;
            }
            let msg = format!("Durmiendo {}h...", world.sleep_pending_hours);
            let mid_row = RENDER_START + (render_rows as u16) / 2;
            render_message_overlay(&mut out, view_w, mid_row, &msg, true)?;
            render_hud_area(&mut out, &world, view_w, hud_start, elapsed_ms)?;
            out.flush()?;
            if event::poll(std::time::Duration::from_millis(120))? {
                let _ = event::read()?;
            }
            continue;
        }

        if world.mode == GameMode::Laptop {
            render_laptop_view(&mut out, view_w, render_rows, &world)?;
            render_hud_area(&mut out, &world, view_w, hud_start, elapsed_ms)?;
            out.flush()?;
            if event::poll(rest_tick)? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break 'game;
                    }
                    if !handle_audio_toggle(&key, &audio, &mut world) {
                        if let Some(action) = key_to_action(key, &world.mode) {
                            dispatch(&mut world, action);
                        }
                    }
                }
            } else {
                dispatch(&mut world, Action::LaptopIdle);
            }
            continue;
        }

        if world.mode.shows_ceiling() {
            let initial_wake = world.mode == GameMode::InitialWake;
            render_ceiling(&mut out, view_w, render_rows, RENDER_START, initial_wake)?;
            render_hud_area(&mut out, &world, view_w, hud_start, elapsed_ms)?;
            if !initial_wake {
                render_message_overlay(&mut out, view_w, OVERLAY_ROW, "Tumbado. X levantar · Z dormir.", true)?;
            }
            out.flush()?;

            if event::poll(rest_tick)? {
                let evt = event::read()?;
                if let Event::Key(key) = evt {
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break 'game;
                    }
                    if !handle_audio_toggle(&key, &audio, &mut world) {
                        if let Some(action) = key_to_action(key, &world.mode) {
                            dispatch(&mut world, action);
                        }
                    }
                }
            } else if !initial_wake {
                dispatch(&mut world, Action::RestTick);
            }
            continue;
        }

        let p = &world.player;
        let result = raycaster::cast(
            &world.map, p.x, p.y, p.dir_x, p.dir_y, p.plane_x, p.plane_y, view_w,
        );
        let outdoor = world.map.get(p.x as usize, p.y as usize).is_outdoor();
        let cloudy = matches!(world.weather, game::world::Weather::Cloudy);
        let mut frame = renderer.build_frame(&result, &world.doors, outdoor, cloudy, &world.map, p.x, p.y);
        if !outdoor {
            renderer.draw_ceiling_lights(
                &mut frame, &result,
                p.x, p.y,
                &world.fluorescents,
                elapsed_ms,
            );
        }

        let sprites = collect_sprites(&world);
        renderer.draw_sprites(
            &mut frame, &result,
            p.x, p.y, p.dir_x, p.dir_y, p.plane_x, p.plane_y,
            &sprites,
        );

        blit_frame(&mut out, &frame, view_w, render_rows, RENDER_START)?;
        render_hud_area(&mut out, &world, view_w, hud_start, elapsed_ms)?;

        if world.mode == GameMode::Sitting {
            render_message_overlay(&mut out, view_w, OVERLAY_ROW, "Sentado. X para levantarte.", true)?;
        } else if let Some(msg) = world.active_message() {
            let style = match world.message_style {
                game::world::MessageStyle::Protagonist => OverlayStyle::Protagonist,
                game::world::MessageStyle::Other => OverlayStyle::Other,
                game::world::MessageStyle::Neutral => OverlayStyle::Neutral,
            };
            render_styled_message_overlay(&mut out, view_w, OVERLAY_ROW, &msg, style)?;
        }

        if world.mode == GameMode::Exploring {
            if let Some(act) = build_action_label(&world) {
                let labeled = format!("X: {}", act);
                render_action_prompt(&mut out, view_w, render_rows, &labeled)?;
            }
        }
        if world.mode == GameMode::Menu {
            render_main_menu(&mut out, view_w, render_rows, world.menu_cursor)?;
        }

        out.flush()?;

        let has_flicker = world.fluorescents.iter().any(|f| matches!(
            f.state, objects::FluorescentState::Flicker { .. }
        ));
        let is_sitting = world.mode == GameMode::Sitting;
        let has_pending = !world.pending.is_empty();

        let event_ready = if is_sitting {
            event::poll(rest_tick)?
        } else if has_flicker || has_pending {
            event::poll(frame_gap)?
        } else {
            true
        };
        if !event_ready {
            if is_sitting { dispatch(&mut world, Action::RestTick); }
            continue;
        }
        let evt = event::read()?;
        match evt {
            Event::Key(key) => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    break 'game;
                }
                if !handle_audio_toggle(&key, &audio, &mut world) {
                    if let Some(action) = key_to_action(key, &world.mode) {
                        dispatch(&mut world, action);
                    }
                }
            }
            Event::Mouse(m) => {
                if let Some(action) = mouse_to_action(m, &world.mode) {
                    dispatch(&mut world, action);
                }
            }
            _ => {}
        }
    }

    execute!(out, ResetColor, DisableMouseCapture, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

