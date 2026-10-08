//! Boxes drawn over the view: messages and dialogue, the action prompt, menus.

use hackkid_core::game::world::MessageStyle;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use super::text::wrap;
use super::{fill, put};

const PASSIVE: (Color, Color) = (Color::Rgb(40, 45, 60), Color::Rgb(220, 220, 230));

fn colors(style: MessageStyle) -> (Color, Color) {
    match style {
        MessageStyle::Protagonist => (Color::Rgb(250, 220, 90), Color::Black),
        MessageStyle::Other => (Color::Rgb(10, 10, 15), Color::Rgb(240, 240, 245)),
        MessageStyle::Neutral => (Color::Rgb(85, 85, 92), Color::White),
    }
}

/// A framed, centred box whose text wraps; never wider or taller than `area`.
fn text_box(buf: &mut Buffer, area: Rect, top: u16, text: &str, (bg, fg): (Color, Color)) {
    let max_inner = (area.width as usize).saturating_sub(8).min(100);
    let max_lines = (area.bottom().saturating_sub(top) as usize).saturating_sub(2).max(1);
    let mut lines = wrap(text, max_inner.saturating_sub(4));
    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            last.push('…');
        }
    }
    let inner = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 4;
    let x = area.x + area.width.saturating_sub(inner as u16 + 2) / 2;
    let style = Style::new().fg(fg).bg(bg);
    put(buf, x, top, &format!("┌{}┐", "─".repeat(inner)), style);
    for (i, l) in lines.iter().enumerate() {
        put(buf, x, top + 1 + i as u16, &format!("│  {l:<w$}  │", w = inner - 4), style);
    }
    put(buf, x, top + 1 + lines.len() as u16, &format!("└{}┘", "─".repeat(inner)), style);
}

pub fn message(buf: &mut Buffer, area: Rect, top: u16, text: &str, style: MessageStyle) {
    text_box(buf, area, top, text, colors(style));
}

pub fn passive(buf: &mut Buffer, area: Rect, top: u16, text: &str) {
    text_box(buf, area, top, text, PASSIVE);
}

/// The "X: ..." prompt near the bottom of the view.
pub fn prompt(buf: &mut Buffer, view: Rect, label: &str) {
    let top = view.bottom().saturating_sub(4);
    text_box(buf, view, top, label, (Color::Rgb(25, 25, 35), Color::Rgb(255, 220, 90)));
}

/// One dim line at the bottom of `area`.
pub fn hint(buf: &mut Buffer, area: Rect, text: &str) {
    let x = area.x + area.width.saturating_sub(text.chars().count() as u16) / 2;
    put(buf, x, area.bottom().saturating_sub(2), text, Style::new().fg(Color::Rgb(120, 125, 145)));
}

pub fn menu(buf: &mut Buffer, area: Rect, title: &str, items: &[&str], cursor: usize) {
    let inner = items.iter().map(|i| i.chars().count() + 2).chain([title.chars().count()]).max().unwrap_or(0) + 6;
    let h = items.len() as u16 + 6;
    let x = area.x + area.width.saturating_sub(inner as u16 + 2) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let bg = Color::Rgb(20, 20, 30);
    let border = Style::new().fg(Color::Rgb(220, 220, 240)).bg(bg);
    fill(buf, Rect::new(x, y, inner as u16 + 2, h), bg);
    put(buf, x, y, &format!("┌{}┐", "─".repeat(inner)), border);
    for r in 1..h - 1 {
        put(buf, x, y + r, "│", border);
        put(buf, x + inner as u16 + 1, y + r, "│", border);
    }
    put(buf, x, y + h - 1, &format!("└{}┘", "─".repeat(inner)), border);
    put(buf, x + 3, y + 2, title, Style::new().fg(Color::White).bg(bg));
    for (i, item) in items.iter().enumerate() {
        let selected = i == cursor;
        let marker = if selected { "▶ " } else { "  " };
        let style = if selected {
            Style::new().fg(Color::White).bg(Color::Rgb(90, 90, 130))
        } else {
            Style::new().fg(Color::White).bg(bg)
        };
        put(buf, x + 1, y + 4 + i as u16, &format!("  {marker}{item:<w$}", w = inner - 4), style);
    }
}

/// Project Zomboid style status boxes down the right of the view, worst first. They only
/// appear when something needs attention; critical ones pulse.
pub fn moodles(buf: &mut Buffer, view: Rect, moodles: &[hackkid_core::game::world::Moodle], now_ms: u64) {
    let pulse = (now_ms / 400).is_multiple_of(2);
    for (i, m) in moodles.iter().enumerate() {
        let y = view.y + 1 + i as u16;
        if y + 4 >= view.bottom() {
            break;
        }
        let (bg, fg) = match m.level {
            1 => (Color::Rgb(206, 186, 70), Color::Black),
            2 => (Color::Rgb(224, 132, 40), Color::Black),
            3 => (Color::Rgb(196, 48, 40), Color::White),
            _ => (if pulse { Color::Rgb(150, 20, 20) } else { Color::Rgb(90, 10, 10) }, Color::White),
        };
        let text = format!(" {} {} ", m.label, "▲".repeat(m.level as usize));
        let x = view.right().saturating_sub(text.chars().count() as u16 + 1);
        put(buf, x, y, &text, Style::new().fg(fg).bg(bg));
    }
}
