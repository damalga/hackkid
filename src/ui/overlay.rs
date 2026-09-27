use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

use crate::ui::frame::{RENDER_START, SIDE_PAD};

#[derive(Debug, Clone, Copy)]
pub enum OverlayStyle {
    Passive,
    Protagonist,
    Other,
    Neutral,
}

pub fn render_message_overlay<W: Write>(
    out: &mut W,
    view_w: usize,
    row: u16,
    msg: &str,
    passive: bool,
) -> io::Result<()> {
    let style = if passive { OverlayStyle::Passive } else { OverlayStyle::Protagonist };
    render_styled_message_overlay(out, view_w, row, msg, style)
}

pub fn render_styled_message_overlay<W: Write>(
    out: &mut W,
    view_w: usize,
    row: u16,
    msg: &str,
    style: OverlayStyle,
) -> io::Result<()> {
    let padded = format!("  {}  ", msg);
    let max_len = view_w.saturating_sub(4);
    let display: String = padded.chars().take(max_len).collect();
    let inner_len = display.chars().count();
    let box_w = inner_len + 2;
    let x = SIDE_PAD + ((view_w.saturating_sub(box_w)) / 2) as u16;

    let (bg, fg) = match style {
        OverlayStyle::Passive => (Color::Rgb { r: 40, g: 45, b: 60 }, Color::Rgb { r: 220, g: 220, b: 230 }),
        OverlayStyle::Protagonist => (Color::Rgb { r: 250, g: 220, b: 90 }, Color::Black),
        OverlayStyle::Other => (Color::Rgb { r: 10, g: 10, b: 15 }, Color::Rgb { r: 240, g: 240, b: 245 }),
        OverlayStyle::Neutral => (Color::Rgb { r: 85, g: 85, b: 92 }, Color::White),
    };

    let top = format!("┌{}┐", "─".repeat(inner_len));
    let mid = format!("│{}│", display);
    let bot = format!("└{}┘", "─".repeat(inner_len));
    queue!(out,
        cursor::MoveTo(x, row),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(top),
        cursor::MoveTo(x, row + 1),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(mid),
        cursor::MoveTo(x, row + 2),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(bot),
        ResetColor,
    )
}

pub fn render_action_prompt<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    action: &str,
) -> io::Result<()> {
    let inner = format!("  {}  ", action);
    let inner_len = inner.chars().count();
    let box_w = inner_len + 2;
    let start_x = SIDE_PAD + ((view_w.saturating_sub(box_w)) as u16 / 2);
    let row_top = RENDER_START + (render_rows as u16).saturating_sub(4);
    let top = format!("┌{}┐", "─".repeat(inner_len));
    let mid = format!("│{}│", inner);
    let bot = format!("└{}┘", "─".repeat(inner_len));
    let fg = SetForegroundColor(Color::Rgb { r: 255, g: 220, b: 90 });
    let bg = SetBackgroundColor(Color::Rgb { r: 25, g: 25, b: 35 });
    queue!(out, cursor::MoveTo(start_x, row_top), bg, fg, Print(&top), ResetColor)?;
    queue!(out, cursor::MoveTo(start_x, row_top + 1), bg, fg, Print(&mid), ResetColor)?;
    queue!(out, cursor::MoveTo(start_x, row_top + 2), bg, fg, Print(&bot), ResetColor)?;
    Ok(())
}
