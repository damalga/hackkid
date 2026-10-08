//! Everything on screen, drawn into ratatui's buffer each frame. ratatui only sends the
//! cells that changed, so the 3D view and the HUD can be redrawn every tick for free.

mod hud;
mod laptop;
mod overlay;
pub mod text;
mod view;

use hackkid_core::engine::renderer::Viewport;
use hackkid_core::game::tracks::TRACKS;
use hackkid_core::game::world::{GameMode, MENU_ITEMS, TITLE_ITEMS, World};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

pub const MIN_W: u16 = 72;
pub const MIN_H: u16 = 20;
/// Status line and the rule under it.
const TOP: u16 = 2;
/// The full HUD's height, with the line above it.
const FULL_HUD_H: u16 = 15;
/// The compact HUD: one line of bars.
const COMPACT_HUD_H: u16 = 1;

const AMBER: Color = Color::Rgb(235, 220, 140);
const RULE: Color = Color::Rgb(120, 130, 150);

/// Draws a frame. `full_hud` asks for the big HUD (the inventory always gets it); the
/// compact one leaves most of the screen to the view.
pub fn draw(f: &mut Frame, world: &World, vp: &mut Viewport, now_ms: u64, full_hud: bool) {
    let area = f.area();
    let buf = f.buffer_mut();
    if area.width < MIN_W || area.height < MIN_H {
        too_small(buf, area);
        return;
    }
    draw_status(buf, area, world);

    let below = Rect::new(area.x, area.y + TOP, area.width, area.height - TOP);
    let full = (full_hud || world.mode == GameMode::Inventory) && area.height >= TOP + FULL_HUD_H + 10;
    let hud_h = if full { FULL_HUD_H } else { COMPACT_HUD_H };
    let view = Rect::new(area.x, area.y + TOP, area.width, area.height - TOP - hud_h);
    let hud_area = Rect::new(area.x, view.bottom(), area.width, hud_h);
    let hud = |buf: &mut Buffer| {
        if full {
            hud::draw(buf, Rect::new(hud_area.x, hud_area.y + 1, hud_area.width, hud_area.height - 1), world, now_ms);
        } else {
            hud::compact(buf, hud_area, world, now_ms);
        }
    };

    match world.mode {
        GameMode::Startup => {
            fill(buf, below, Color::Rgb(8, 10, 18));
            overlay::menu(buf, below, "FANFARE // RECOVERY", &TITLE_ITEMS, world.menu_cursor);
            overlay::hint(buf, below, "↑↓ select · Enter confirm · Ctrl+C quit");
            if let Some((msg, style)) = world.active_message() {
                overlay::message(buf, below, below.y + 1, msg, style);
            }
        }
        GameMode::Dead => {
            fill(buf, below, Color::Rgb(8, 6, 8));
            let mid = below.y + below.height / 2;
            overlay::passive(buf, below, mid.saturating_sub(2), "You have died.");
            overlay::passive(buf, below, mid + 2, "Press Enter to return to the title screen.");
        }
        GameMode::Laptop => laptop::draw(buf, below, world),
        GameMode::Sleeping => {
            fill(buf, view, Color::Rgb(4, 5, 10));
            overlay::passive(buf, view, view.y + view.height / 2 - 1, &format!("Sleeping {}h...", world.sleep_hours));
            hud(buf);
        }
        GameMode::InitialWake | GameMode::Lying => {
            view::ceiling(buf, view, world, vp, now_ms, world.mode == GameMode::InitialWake);
            hud(buf);
            if world.mode == GameMode::Lying {
                draw_messages(buf, view, world);
                overlay::prompt(buf, view, "X: Get up  ·  Z: Sleep");
            }
        }
        _ => {
            view::world(buf, view, world, vp, now_ms);
            hud(buf);
            overlay::moodles(buf, view, &world.moodles(), now_ms);
            draw_messages(buf, view, world);
            match world.mode {
                GameMode::Exploring => {
                    if let Some(label) = world.action_label() {
                        overlay::prompt(buf, view, &label);
                    }
                }
                GameMode::Sitting => overlay::prompt(buf, view, "X: Stand up"),
                GameMode::Inventory => overlay::prompt(buf, view, "↑↓ select · Enter use · T drop · B take off backpack · Esc close"),
                GameMode::Menu => overlay::menu(buf, view, "MENU", &MENU_ITEMS, world.menu_cursor),
                _ => {}
            }
        }
    }
}

fn draw_messages(buf: &mut Buffer, view: Rect, world: &World) {
    if let Some((msg, style)) = world.active_message() {
        overlay::message(buf, view, view.y + 1, msg, style);
    }
}

fn draw_status(buf: &mut Buffer, area: Rect, world: &World) {
    let h = world.hour as u32;
    let m = ((world.hour - h as f64) * 60.0) as u32;
    let weather = if world.weather_seen { world.weather.label() } else { "" };
    let hidden = if world.player.hidden { "  [HIDDEN]" } else { "" };
    let status = format!("FANFARE // EMERGENCY RECOVERY PROTOCOL — Day {}  {h:02}:{m:02}  {weather}{hidden}", world.day);
    fill(buf, Rect::new(area.x, area.y, area.width, TOP), Color::Reset);
    put(buf, area.x + 2, area.y, &status, Style::new().fg(AMBER));
    if let Some(idx) = world.music() {
        let t = &TRACKS[idx];
        let np = format!("♪ {:02} {}  ", t.n, t.name);
        let x = area.right().saturating_sub(np.chars().count() as u16);
        put(buf, x, area.y, &np, Style::new().fg(Color::Rgb(155, 160, 22)));
    }
    put(buf, area.x, area.y + 1, &"─".repeat(area.width as usize), Style::new().fg(RULE));
}

fn too_small(buf: &mut Buffer, area: Rect) {
    fill(buf, area, Color::Reset);
    let lines = [
        "Terminal too small".to_string(),
        format!("{}×{}, FANFARE needs at least {MIN_W}×{MIN_H}", area.width, area.height),
    ];
    let y0 = area.y + area.height / 2;
    for (i, l) in lines.iter().enumerate() {
        let x = area.x + area.width.saturating_sub(l.chars().count() as u16) / 2;
        put(buf, x, (y0 + i as u16).saturating_sub(1), l, Style::new().fg(AMBER));
    }
}

/// Writes text at (x, y), clipped to the buffer. Nothing outside it is ever touched.
pub fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    let a = buf.area;
    if y < a.top() || y >= a.bottom() || x < a.left() || x >= a.right() {
        return;
    }
    buf.set_stringn(x, y, text, (a.right() - x) as usize, style);
}

/// Blanks a rectangle to a background colour.
pub fn fill(buf: &mut Buffer, rect: Rect, bg: Color) {
    let r = rect.intersection(buf.area);
    for y in r.top()..r.bottom() {
        for x in r.left()..r.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                c.reset();
                c.set_bg(bg);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hackkid_core::engine::renderer::Viewport;
    use hackkid_core::game::world::{GameMode, World};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    fn worlds() -> Vec<World> {
        let modes = [
            GameMode::Startup, GameMode::InitialWake, GameMode::Exploring, GameMode::Sitting, GameMode::Lying,
            GameMode::Sleeping, GameMode::Inventory, GameMode::Laptop, GameMode::Menu, GameMode::Dead,
        ];
        modes
            .iter()
            .map(|&mode| {
                let mut w = World::new();
                w.mode = mode;
                w.player.has_backpack = true;
                w.player.has_map = true;
                w.laptop.playing = Some(6);
                w.say("Julian: 'Radio towers are dead, but your emergency terminal can pick up a repeating broadcast on 104.2 MHz. Take this hospital map, and check the terminal.'");
                w
            })
            .collect()
    }

    /// Every screen, at every size from tiny to huge, draws without panicking.
    #[test]
    fn every_mode_draws_at_any_size() {
        for w in worlds() {
            for (cols, rows) in [(1, 1), (20, 8), (79, 25), (80, 26), (100, 30), (120, 40), (250, 80)] {
                let mut t = Terminal::new(TestBackend::new(cols, rows)).unwrap();
                let mut vp = Viewport::default();
                for full in [false, true] {
                    t.draw(|f| super::draw(f, &w, &mut vp, 1_000, full)).unwrap();
                }
            }
        }
    }

    /// The long line from Julian is shown in full, wrapped, at the recommended size.
    #[test]
    fn dialogue_is_wrapped_not_cut() {
        let mut w = World::new();
        w.mode = GameMode::Exploring;
        let line = "Julian: 'Radio towers are dead, but your emergency terminal can pick up a repeating broadcast on 104.2 MHz. Take this hospital map, and check the terminal.'";
        w.say(line);
        let mut t = Terminal::new(TestBackend::new(120, 40)).unwrap();
        t.draw(|f| super::draw(f, &w, &mut Viewport::default(), 1_000, false)).unwrap();
        let buf = t.backend().buffer();
        let screen: String = (0..buf.area.height)
            .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        for word in line.split_whitespace() {
            assert!(screen.contains(word), "missing {word:?}");
        }
    }
}
