//! The four boxes under the view: vitals, map, inventory, wardrobe.

use hackkid_core::engine::map::Tile;
use hackkid_core::game::player::{BACKPACK_SLOTS, Stats};
use hackkid_core::game::world::{GameMode, StaminaState, World};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

use super::text::ellipsize;
use super::{fill, put};

const LABEL_W: usize = 7;
const WHITE: Color = Color::White;
const DIM: Color = Color::Rgb(180, 180, 200);
const BORDER: Color = Color::Rgb(130, 140, 160);
const ALARM: Color = Color::Rgb(255, 110, 90);
const RECOVER: Color = Color::Rgb(90, 170, 255);
const OPTIMAL: Color = Color::Rgb(90, 220, 120);
const HEADER: Color = Color::Rgb(140, 170, 210);
const DOOR: Color = Color::Rgb(255, 220, 40);

#[derive(Clone, Copy)]
enum Good {
    High,
    Low,
    Centered,
}

fn is_alarm(good: Good, v: f64) -> bool {
    match good {
        Good::High => v <= 10.0,
        Good::Low => v >= 90.0,
        Good::Centered => v.abs() >= 90.0,
    }
}

fn stat_bar(label: &str, value: f64, width: usize) -> String {
    let filled = ((value / 100.0).clamp(0.0, 1.0) * width as f64) as usize;
    format!("{label:<LABEL_W$} {}{} {value:>3.0}", "█".repeat(filled), "░".repeat(width - filled))
}

fn stat_bar_center(label: &str, value: f64, width: usize) -> String {
    let half = width / 2;
    let mut chars = vec!['░'; width];
    let mag = ((value.abs() / 100.0).min(1.0) * half as f64).round() as usize;
    let range = if value > 0.0 { half..(half + mag).min(width) } else { half.saturating_sub(mag)..half };
    for c in &mut chars[range] {
        *c = '█';
    }
    if half < width {
        chars[half] = '│';
    }
    format!("{label:<LABEL_W$} {} {value:+4.0}", chars.iter().collect::<String>())
}

/// A titled group of (label, value, which way is good) rows.
type Group = (&'static str, Vec<(&'static str, f64, Good)>);

fn groups(s: &Stats) -> [Group; 3] {
    [
        ("Vital", vec![("Body", s.body, Good::High), ("Mind", s.mind, Good::High), ("Energy", s.stamina, Good::High), ("Hygiene", s.hygiene, Good::High)]),
        ("Needs", vec![("Thirst", s.thirst, Good::Low), ("Hunger", s.hunger, Good::Low), ("Sleep", s.sleep, Good::Low)]),
        ("Thermal", vec![("Thermal", s.thermal, Good::Centered)]),
    ]
}

fn panel(buf: &mut Buffer, rect: Rect, title: &str) -> Rect {
    fill(buf, rect, Color::Reset);
    let block = Block::bordered()
        .border_style(Style::new().fg(BORDER))
        .title(Line::styled(format!("─ {title} "), Style::new().fg(BORDER)));
    let inner = block.inner(rect);
    block.render(rect, buf);
    inner
}

pub fn draw(buf: &mut Buffer, hud: Rect, world: &World, now_ms: u64) {
    fill(buf, Rect::new(hud.x, hud.y.saturating_sub(1), hud.width, 1), Color::Reset);
    let gap = 1;
    let box_w = (hud.width - 3 * gap) / 4;
    let rect = |i: u16| Rect::new(hud.x + i * (box_w + gap), hud.y, box_w, hud.height);
    let inner = panel(buf, rect(0), "Vitals");
    vitals(buf, inner, world, now_ms);
    let inner = panel(buf, rect(1), "Map");
    map(buf, inner, world);
    let inner = panel(buf, rect(2), "Inventory (I: open)");
    inventory(buf, inner, world);
    let inner = panel(buf, rect(3), "Wardrobe");
    wardrobe(buf, inner, world);
}

fn vitals(buf: &mut Buffer, r: Rect, world: &World, now_ms: u64) {
    let x = r.x + 1;
    let inner_w = r.width.saturating_sub(1) as usize;
    let bar_w = inner_w.saturating_sub(13).max(4);
    let blink_off = (now_ms / 350) % 2 == 1;
    let mut y = r.y;
    for (title, rows) in groups(&world.player.stats) {
        if y >= r.bottom() {
            break;
        }
        let head = format!("─ {title} {}", "─".repeat(inner_w.saturating_sub(title.chars().count() + 4)));
        put(buf, x, y, &head, Style::new().fg(HEADER));
        y += 1;
        for (label, value, good) in rows {
            if y >= r.bottom() {
                break;
            }
            let line = match good {
                Good::Centered => stat_bar_center(label, value, bar_w),
                _ => stat_bar(label, value, bar_w),
            };
            let color = if label == "Energy" {
                match world.stamina_state {
                    StaminaState::Optimal => OPTIMAL,
                    StaminaState::Recovering => RECOVER,
                    _ => WHITE,
                }
            } else if is_alarm(good, value) {
                if blink_off { Color::Reset } else { ALARM }
            } else {
                WHITE
            };
            put(buf, x, y, &line, Style::new().fg(color));
            y += 1;
        }
    }
}

fn map(buf: &mut Buffer, r: Rect, world: &World) {
    if !world.player.has_map {
        let cy = r.y + r.height / 2;
        for (i, msg) in ["Sector map", "not acquired"].iter().enumerate() {
            let x = r.x + r.width.saturating_sub(msg.len() as u16) / 2;
            put(buf, x, cy - 1 + i as u16, msg, Style::new().fg(DIM));
        }
        return;
    }
    let (mw, mh) = (r.width as i32, r.height as i32);
    let (px, py) = (world.player.x as i32, world.player.y as i32);
    let (tw, th) = (world.map.width as i32, world.map.height as i32);
    if world.player.has_city_map {
        // the whole city, downscaled into the box
        let step = (tw as f64 / mw as f64).max(th as f64 / mh as f64).max(1.0);
        let used_w = ((tw as f64 / step).ceil() as i32).min(mw);
        let used_h = ((th as f64 / step).ceil() as i32).min(mh);
        let (off_x, off_y) = ((mw - used_w) / 2, (mh - used_h) / 2);
        for row in 0..used_h {
            for col in 0..used_w {
                let (tx0, ty0) = ((col as f64 * step) as i32, (row as f64 * step) as i32);
                let (tx1, ty1) = ((((col + 1) as f64) * step).ceil() as i32, (((row + 1) as f64) * step).ceil() as i32);
                let mut best: Option<(u8, &str, Color)> = None;
                for ty in ty0..ty1.min(th) {
                    for tx in tx0..tx1.min(tw) {
                        let cand = map_glyph(world.map.get(tx as usize, ty as usize));
                        if best.is_none_or(|b| cand.0 > b.0) {
                            best = Some(cand);
                        }
                    }
                }
                if let Some((_, g, c)) = best {
                    put(buf, r.x + (off_x + col) as u16, r.y + (off_y + row) as u16, g, Style::new().fg(c));
                }
            }
        }
        let (pc, pr) = (off_x + (px as f64 / step) as i32, off_y + (py as f64 / step) as i32);
        if (0..mw).contains(&pc) && (0..mh).contains(&pr) {
            put(buf, r.x + pc as u16, r.y + pr as u16, "●", Style::new().fg(DOOR));
        }
        return;
    }
    // the hospital around the player, one tile per cell
    for row in 0..mh {
        for col in 0..mw {
            let (tx, ty) = (px + col - mw / 2, py + row - mh / 2);
            if tx < 0 || ty < 0 || tx >= tw || ty >= th {
                continue;
            }
            let tile = world.map.get(tx as usize, ty as usize);
            let (g, c) = if tile.is_any_door() {
                ("+", DOOR)
            } else if tile.blocks_movement() {
                ("·", WHITE)
            } else {
                continue;
            };
            put(buf, r.x + col as u16, r.y + row as u16, g, Style::new().fg(c));
        }
    }
    put(buf, r.x + (mw / 2) as u16, r.y + (mh / 2) as u16, "●", Style::new().fg(DOOR));
}

/// (priority, glyph, colour) of a tile on the downscaled city map.
fn map_glyph(tile: Tile) -> (u8, &'static str, Color) {
    match tile {
        t if t.is_any_door() => (5, "+", DOOR),
        t if t.blocks_movement() => (4, "·", WHITE),
        Tile::Road => (3, ":", Color::Rgb(90, 90, 100)),
        Tile::Sidewalk => (2, ".", Color::Rgb(170, 170, 175)),
        Tile::Garden | Tile::Outdoor | Tile::Parking => (1, "*", Color::Rgb(90, 150, 80)),
        _ => (0, " ", Color::Reset),
    }
}

fn inventory(buf: &mut Buffer, r: Rect, world: &World) {
    let x = r.x + 1;
    let inv = &world.player.inventory;
    let has_pockets = world.player.wardrobe.wearing_pants();
    if !world.player.has_backpack {
        put(buf, x, r.y, "(no backpack)", Style::new().fg(DIM));
        if !has_pockets {
            return;
        }
    }
    let slot_w = (r.width as usize).saturating_sub(4);
    let visible = inv.capacity.min(inv.slots.len()).min(r.height as usize);
    for i in 0..visible {
        let pocket = i >= BACKPACK_SLOTS;
        if !world.player.has_backpack && !pocket {
            continue;
        }
        let selected = world.mode == GameMode::Inventory && world.player.inventory_cursor == i;
        let name = match &inv.slots[i] {
            Some(it) if it.units > 0 => format!("{} ({})", it.label(), it.units),
            Some(it) => it.label().to_string(),
            None => "—".into(),
        };
        let prefix = if selected { "▶ " } else if pocket { "* " } else { "  " };
        let line = format!("{prefix}{:<slot_w$}", ellipsize(&name, slot_w));
        let style = if selected { Style::new().fg(WHITE).bg(Color::Rgb(90, 90, 130)) } else { Style::new().fg(WHITE) };
        put(buf, x, r.y + i as u16, &line, style);
    }
}

fn wardrobe(buf: &mut Buffer, r: Rect, world: &World) {
    let x = r.x + 1;
    let inner_w = r.width.saturating_sub(1) as usize;
    let rows = world.player.wardrobe.display_rows();
    for (i, (tag, item)) in rows.iter().enumerate() {
        put(buf, x, r.y + i as u16, &ellipsize(&format!("{tag:<8} {item}"), inner_w), Style::new().fg(WHITE));
    }
    let bag = if world.player.has_backpack { "Worn (B: take off)" } else { "None" };
    put(buf, x, r.y + rows.len() as u16 + 1, &ellipsize(&format!("{:<8} {bag}", "Backpack:"), inner_w), Style::new().fg(WHITE));
    let flask = if world.has_canteen_in_inventory() { format!("{:.0}% (Q: drink)", world.player.canteen_fill) } else { "—".into() };
    put(buf, x, r.y + rows.len() as u16 + 2, &ellipsize(&format!("{:<8} {flask}", "Flask:"), inner_w), Style::new().fg(WHITE));
}
