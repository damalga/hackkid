//! The four boxes under the view: vitals, map, inventory, wardrobe.

use hackkid_core::game::player::{BACKPACK_SLOTS, Stats};
use hackkid_core::game::world::{GameMode, MessageStyle, StaminaState, World};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

use super::text::{ellipsize, wrap};
use super::{Hits, Hover, hovered, put, shade};

const LABEL_W: usize = 7;
const WHITE: Color = Color::White;
const DIM: Color = Color::Rgb(180, 180, 200);
const BORDER: Color = Color::Rgb(130, 140, 160);
const ALARM: Color = Color::Rgb(255, 110, 90);
const RECOVER: Color = Color::Rgb(90, 170, 255);
const OPTIMAL: Color = Color::Rgb(90, 220, 120);
const HEADER: Color = Color::Rgb(140, 170, 210);

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

/// A see-through panel: the view behind darkens, the border and contents go on top.
fn panel(buf: &mut Buffer, rect: Rect, title: &str) -> Rect {
    shade(buf, rect, 0.3);
    let block = Block::bordered()
        .border_style(Style::new().fg(BORDER))
        .title(Line::styled(format!("─ {title} "), Style::new().fg(BORDER)));
    let inner = block.inner(rect);
    block.render(rect, buf);
    inner
}

pub fn draw(buf: &mut Buffer, hud: Rect, world: &World, now_ms: u64, hover: Hover, hits: &mut Hits) {
    let gap = 1;
    let box_w = (hud.width - 3 * gap) / 4;
    let rect = |i: u16| Rect::new(hud.x + i * (box_w + gap), hud.y, box_w, hud.height);
    let inner = panel(buf, rect(0), "Vitals");
    vitals(buf, inner, world, now_ms);
    let inner = panel(buf, rect(1), "Log");
    log(buf, inner, world);
    let inner = panel(buf, rect(2), "Inventory");
    inventory(buf, inner, world, hover, hits);
    let inner = panel(buf, rect(3), "Wardrobe");
    wardrobe(buf, inner, world);
}

/// The last few messages, newest at the bottom, each in its speaker's colour.
fn log(buf: &mut Buffer, r: Rect, world: &World) {
    let x = r.x + 1;
    let width = (r.width as usize).saturating_sub(2).max(8);
    let mut lines: Vec<(String, MessageStyle)> = Vec::new();
    for (text, style) in &world.log {
        for l in wrap(text, width) {
            lines.push((l, *style));
        }
    }
    let shown = lines.len().min(r.height as usize);
    for (i, (l, style)) in lines[lines.len() - shown..].iter().enumerate() {
        let newest = i + 1 == shown;
        let color = match (style, newest) {
            (MessageStyle::Protagonist, true) => Color::Rgb(250, 220, 90),
            (MessageStyle::Protagonist, false) => Color::Rgb(170, 150, 70),
            (MessageStyle::Other, true) => Color::Rgb(240, 240, 245),
            (MessageStyle::Other, false) => Color::Rgb(160, 160, 170),
            (MessageStyle::Neutral, true) => Color::Rgb(205, 205, 212),
            (MessageStyle::Neutral, false) => Color::Rgb(125, 125, 135),
        };
        put(buf, x, r.y + i as u16, l, Style::new().fg(color));
    }
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
                if blink_off { Color::Rgb(90, 40, 34) } else { ALARM }
            } else {
                WHITE
            };
            put(buf, x, y, &line, Style::new().fg(color));
            y += 1;
        }
    }
}

fn inventory(buf: &mut Buffer, r: Rect, world: &World, hover: Hover, hits: &mut Hits) {
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
            Some(it) if it.count() > 1 => format!("{} ×{}", it.label(), it.count()),
            Some(it) => it.label().to_string(),
            None => "—".into(),
        };
        let prefix = if selected { "▶ " } else if pocket { "* " } else { "  " };
        let line = format!("{prefix}{:<slot_w$}", ellipsize(&name, slot_w));
        let row = Rect::new(x, r.y + i as u16, (slot_w + 2) as u16, 1);
        if world.mode == GameMode::Inventory {
            hits.slots.push((row, i));
        }
        let style = if selected {
            Style::new().fg(WHITE).bg(Color::Rgb(90, 90, 130))
        } else if hovered(hover, row) && world.mode == GameMode::Inventory {
            Style::new().fg(WHITE).bg(Color::Rgb(60, 60, 84))
        } else {
            Style::new().fg(WHITE)
        };
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
    let torch = if world.has_flashlight() {
        format!("{:.0}% {}", world.player.flashlight_charge, if world.player.flashlight_on { "on (L: off)" } else { "off (L: on)" })
    } else {
        "—".into()
    };
    put(buf, x, r.y + rows.len() as u16 + 3, &ellipsize(&format!("{:<8} {torch}", "Light:"), inner_w), Style::new().fg(WHITE));
}

/// One line under the view: small bars for every stat, the flask, and the keys for more.
pub fn compact(buf: &mut Buffer, r: Rect, world: &World, now_ms: u64) {
    shade(buf, r, 0.35);
    let s = &world.player.stats;
    let blink_off = (now_ms / 350) % 2 == 1;
    let mut x = r.x + 1;
    let bar = |v: f64| {
        let n = ((v / 100.0).clamp(0.0, 1.0) * 5.0).round() as usize;
        format!("{}{}", "█".repeat(n), "░".repeat(5 - n))
    };
    let stats = [
        ("Body", s.body, Good::High),
        ("Mind", s.mind, Good::High),
        ("Energy", s.stamina, Good::High),
        ("Hygiene", s.hygiene, Good::High),
        ("Thirst", s.thirst, Good::Low),
        ("Hunger", s.hunger, Good::Low),
        ("Sleep", s.sleep, Good::Low),
    ];
    for (i, (label, v, good)) in stats.into_iter().enumerate() {
        if i == 4 {
            put(buf, x, r.y, "│", Style::new().fg(BORDER));
            x += 2;
        }
        let color = if is_alarm(good, v) { if blink_off { Color::Rgb(90, 40, 34) } else { ALARM } } else { WHITE };
        let text = format!("{label} {} ", bar(v));
        if x + text.chars().count() as u16 >= r.right() {
            return;
        }
        put(buf, x, r.y, label, Style::new().fg(DIM));
        put(buf, x + label.chars().count() as u16 + 1, r.y, &bar(v), Style::new().fg(color));
        x += text.chars().count() as u16 + 1;
    }
    let keys = "Tab: bag · M: map · L: light · Esc: menu ";
    let kx = r.right().saturating_sub(keys.chars().count() as u16);
    if kx > x {
        put(buf, kx, r.y, keys, Style::new().fg(Color::Rgb(110, 116, 136)));
    }
    if world.has_flashlight() {
        let on = world.player.flashlight_on;
        let torch = format!("☼ {:.0}%", world.player.flashlight_charge);
        let tx = kx.saturating_sub(torch.chars().count() as u16 + 2);
        if tx > x {
            let color = if !on { DIM } else if world.player.flashlight_charge < 10.0 { ALARM } else { Color::Rgb(255, 225, 140) };
            put(buf, tx, r.y, &torch, Style::new().fg(color));
        }
    }
}
