//! Everything on screen, drawn into ratatui's buffer each frame. ratatui only sends the
//! cells that changed, so the 3D view and the HUD can be redrawn every tick for free.

mod hud;
mod laptop;
mod map;
mod overlay;
pub mod text;
mod view;

use std::cell::RefCell;

use hackkid_core::engine::renderer::Viewport;
use hackkid_core::game::tracks::TRACKS;
use hackkid_core::game::world::{GameMode, MENU_ITEMS, Power, TITLE_ITEMS, World};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

/// Where things were drawn, for the mouse: the 3D view, the panels laid over it (where
/// the pointer shouldn't look around), and the rows that can be clicked.
#[derive(Debug, Clone, Default)]
pub struct Hits {
    pub view: Rect,
    pub panels: Vec<Rect>,
    /// Inventory slot rows and their slot index.
    pub slots: Vec<(Rect, usize)>,
    /// Menu rows and their item index.
    pub menu: Vec<(Rect, usize)>,
    /// Terminal track rows and their track index.
    pub tracks: Vec<(Rect, usize)>,
}

thread_local! {
    static HITS: RefCell<Hits> = RefCell::new(Hits::default());
}

/// What the last [`draw`] put where.
pub fn last_hits() -> Hits {
    HITS.with(|h| h.borrow().clone())
}

/// The pointer's cell, if it's over the screen.
pub type Hover = Option<(u16, u16)>;

fn hovered(hover: Hover, r: Rect) -> bool {
    hover.is_some_and(|(c, w)| c >= r.x && c < r.right() && w >= r.y && w < r.bottom())
}

pub const MIN_W: u16 = 72;
pub const MIN_H: u16 = 20;
/// Status line and the rule under it.
const TOP: u16 = 2;
/// The full HUD's height.
const FULL_HUD_H: u16 = 14;
/// The compact HUD: one line of bars.
const COMPACT_HUD_H: u16 = 1;

const AMBER: Color = Color::Rgb(235, 220, 140);
const RULE: Color = Color::Rgb(120, 130, 150);

/// Draws a frame. The 3D view always fills the screen; the HUD floats over its bottom,
/// see-through: one line of bars normally, the full panels in the inventory. `show_map`
/// lays the hospital map over the view. `hover` is the pointer, for highlighting what a
/// click would pick. What went where is kept for [`last_hits`].
pub fn draw(f: &mut Frame, world: &World, vp: &mut Viewport, now_ms: u64, show_map: bool, hover: Hover) {
    let area = f.area();
    let buf = f.buffer_mut();
    let mut hits = Hits::default();
    if area.width < MIN_W || area.height < MIN_H {
        too_small(buf, area);
        HITS.with(|h| *h.borrow_mut() = hits);
        return;
    }
    draw_status(buf, area, world, now_ms);

    let view = Rect::new(area.x, area.y + TOP, area.width, area.height - TOP);
    hits.view = view;
    let full = world.mode == GameMode::Inventory;
    let hud_h = if full { FULL_HUD_H.min(view.height.saturating_sub(4)) } else { COMPACT_HUD_H };
    let hud_area = Rect::new(view.x, view.bottom() - hud_h, view.width, hud_h);
    hits.panels.push(hud_area);
    // what isn't under the HUD, for prompts and boxes
    let clear = Rect::new(view.x, view.y, view.width, view.height - hud_h);
    let hud = |buf: &mut Buffer, hits: &mut Hits| {
        if full {
            hud::draw(buf, hud_area, world, now_ms, hover, hits);
        } else {
            hud::compact(buf, hud_area, world, now_ms);
        }
    };

    match world.mode {
        GameMode::Startup => {
            fill(buf, view, Color::Rgb(8, 10, 18));
            overlay::menu(buf, view, "FANFARE // RECOVERY", &TITLE_ITEMS, world.menu_cursor, hover, &mut hits);
            overlay::hint(buf, view, "↑↓ select · Enter or click to confirm · Ctrl+C quit");
            if let Some((msg, style)) = world.active_message() {
                overlay::message(buf, view, view.y + 1, msg, style);
            }
        }
        GameMode::Dead => {
            fill(buf, view, Color::Rgb(8, 6, 8));
            let mid = view.y + view.height / 2;
            overlay::passive(buf, view, mid.saturating_sub(3), world.death.as_deref().unwrap_or("You have died."));
            let survived = format!("You survived {} day{} in the hospital.", world.day, if world.day == 1 { "" } else { "s" });
            overlay::passive(buf, view, mid, &survived);
            overlay::passive(buf, view, mid + 3, "Press Enter to return to the title screen.");
        }
        GameMode::Laptop => {
            laptop::draw(buf, view, world, hover, &mut hits);
            hits.panels.push(view);
            // what the terminal tells you (a track that can't play, the battery) shows here too
            if let Some((msg, style)) = world.active_message() {
                overlay::message(buf, view, view.bottom().saturating_sub(7), msg, style);
            }
        }
        GameMode::Sleeping => {
            fill(buf, view, Color::Rgb(4, 5, 10));
            overlay::passive(buf, view, view.y + view.height / 2 - 1, &format!("Sleeping {}h...", world.sleep_hours));
            hud(buf, &mut hits);
        }
        GameMode::InitialWake | GameMode::Lying => {
            view::ceiling(buf, view, world, vp, now_ms, world.mode == GameMode::InitialWake);
            hud(buf, &mut hits);
            if world.mode == GameMode::Lying {
                draw_messages(buf, clear, world);
                overlay::prompt(buf, clear, "E: Get up  ·  F: Sleep");
            }
        }
        _ => {
            view::world(buf, view, world, vp, now_ms);
            if show_map {
                map::draw(buf, clear, world);
                hits.panels.push(clear);
            }
            hud(buf, &mut hits);
            overlay::moodles(buf, clear, &world.moodles(), now_ms);
            draw_messages(buf, clear, world);
            match world.mode {
                GameMode::Exploring | GameMode::Inventory if show_map => overlay::prompt(buf, clear, "M, Esc or a click: close the map"),
                GameMode::Exploring => {
                    if let Some(label) = world.action_label() {
                        overlay::prompt(buf, clear, &label);
                    }
                }
                GameMode::Sitting => overlay::prompt(buf, clear, "E: Stand up"),
                GameMode::Inventory => {
                    let label = world.action_label().map(|l| format!("{l}  ·  ")).unwrap_or_default();
                    overlay::prompt(buf, clear, &format!("{label}↑↓ or click: pick · E or click again: use · G or right-click: drop · WASD still walks · Tab close"));
                }
                GameMode::Menu => overlay::menu(buf, clear, "MENU", &MENU_ITEMS, world.menu_cursor, hover, &mut hits),
                _ => {}
            }
        }
    }
    HITS.with(|h| *h.borrow_mut() = hits);
}

fn draw_messages(buf: &mut Buffer, view: Rect, world: &World) {
    if let Some((msg, style)) = world.active_message() {
        overlay::message(buf, view, view.y + 1, msg, style);
    }
}

fn draw_status(buf: &mut Buffer, area: Rect, world: &World, now_ms: u64) {
    let h = world.hour as u32;
    let m = ((world.hour - h as f64) * 60.0) as u32;
    let weather = if world.weather_seen { world.weather.label() } else { "" };
    let temp = format!("{:.0}°C", world.ambient_temp());
    let hidden = if world.player.hidden { "  [CROUCHING]" } else { "" };
    let dark = match world.power {
        Power::Out { until_ms } if now_ms < until_ms => "  [POWER OUT]",
        _ => "",
    };
    let auto = if world.motion.until_ms == u64::MAX && (world.motion.forward != 0.0 || world.motion.side != 0.0) { "  [AUTO-WALK]" } else { "" };
    let mut status = format!("FANFARE // EMERGENCY RECOVERY PROTOCOL — Day {}  {h:02}:{m:02}  {temp}  {weather}{hidden}{dark}{auto}", world.day);
    if debug() {
        let p = &world.player;
        let heading = p.dir_y.atan2(p.dir_x).to_degrees().rem_euclid(360.0);
        status.push_str(&format!("  [{:.2},{:.2} {heading:.0}° {:+.2}]", p.x, p.y, p.pitch));
    }
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

/// `HACKKID_DEBUG=1` shows the player's position, heading and pitch in the status line.
fn debug() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("HACKKID_DEBUG").is_some())
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

/// Darkens whatever is already drawn in a rectangle (the 3D view), keeping its detail:
/// a see-through backdrop for panels laid over it.
pub fn shade(buf: &mut Buffer, rect: Rect, k: f32) {
    let dim = |c: Color| match c {
        Color::Rgb(r, g, b) => Color::Rgb((r as f32 * k) as u8, (g as f32 * k) as u8, (b as f32 * k) as u8),
        other => other,
    };
    let r = rect.intersection(buf.area);
    for y in r.top()..r.bottom() {
        for x in r.left()..r.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                let (fg, bg) = (dim(c.fg), dim(c.bg));
                c.set_fg(fg).set_bg(bg);
            }
        }
    }
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
                for show_map in [false, true] {
                    t.draw(|f| super::draw(f, &w, &mut vp, 1_000, show_map, Some((cols / 2, rows / 2)))).unwrap();
                }
            }
        }
    }

    /// The inventory's HUD floats over the view: the view keeps its full height, and the
    /// scene still shows (darkened) behind the panels.
    #[test]
    fn the_inventory_floats_over_the_view() {
        let mut w = World::new();
        w.mode = GameMode::Exploring;
        w.player.has_backpack = true;
        let mut vp = Viewport::default();
        let mut t = Terminal::new(TestBackend::new(120, 40)).unwrap();
        t.draw(|f| super::draw(f, &w, &mut vp, 1_000, false, None)).unwrap();
        let rows_before = vp.size().1;
        w.mode = GameMode::Inventory;
        t.draw(|f| super::draw(f, &w, &mut vp, 1_000, false, None)).unwrap();
        let hits = super::last_hits();
        assert_eq!(hits.slots.len(), 6, "every backpack slot is a click target");
        assert!(hits.panels.iter().any(|p| p.height > 5), "the HUD is a panel the pointer doesn't look through");
        assert_eq!(vp.size().1, rows_before, "the 3D view isn't squashed");
        let buf = t.backend().buffer();
        let see_through = (0..120).filter(|&x| buf[(x, 36)].symbol() == "▄").count();
        assert!(see_through > 10, "the view shows through between the HUD's contents");
    }

    /// The long line from Julian is shown in full, wrapped, at the recommended size.
    #[test]
    fn dialogue_is_wrapped_not_cut() {
        let mut w = World::new();
        w.mode = GameMode::Exploring;
        let line = "Julian: 'Radio towers are dead, but your emergency terminal can pick up a repeating broadcast on 104.2 MHz. Take this hospital map, and check the terminal.'";
        w.say(line);
        let mut t = Terminal::new(TestBackend::new(120, 40)).unwrap();
        t.draw(|f| super::draw(f, &w, &mut Viewport::default(), 1_000, false, None)).unwrap();
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
