use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

use crate::ui::frame::{RENDER_START, SIDE_PAD};

fn draw_selectable<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    title: &str,
    items: &[&str],
    cursor_idx: usize,
) -> io::Result<()> {
    let mut lines: Vec<String> = Vec::new();
    lines.push(title.to_string());
    lines.push(String::new());
    for (i, item) in items.iter().enumerate() {
        let marker = if i == cursor_idx { "▶ " } else { "  " };
        lines.push(format!("{}{}", marker, item));
    }
    let max_len = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let inner_w = max_len + 4;
    let box_w = inner_w as u16 + 2;
    let box_h = lines.len() as u16 + 4;
    let start_x = SIDE_PAD + ((view_w.saturating_sub(box_w as usize)) as u16 / 2);
    let start_y = RENDER_START + ((render_rows as u16).saturating_sub(box_h)) / 2;
    let border_col = SetForegroundColor(Color::Rgb { r: 220, g: 220, b: 240 });
    let bg = SetBackgroundColor(Color::Rgb { r: 20, g: 20, b: 30 });
    let fg = SetForegroundColor(Color::White);
    let sel_bg = SetBackgroundColor(Color::Rgb { r: 90, g: 90, b: 130 });

    let top = format!("┌{}┐", "─".repeat(inner_w));
    let bot = format!("└{}┘", "─".repeat(inner_w));
    queue!(out, cursor::MoveTo(start_x, start_y), bg, border_col, Print(&top), ResetColor)?;
    let blank = format!(" {:<width$} ", "", width = inner_w - 2);
    for r in 1..box_h - 1 {
        queue!(out,
            cursor::MoveTo(start_x, start_y + r),
            bg,
            border_col,
            Print("│"),
            bg,
            fg,
            Print(&blank),
            bg,
            border_col,
            Print("│"),
            ResetColor,
        )?;
    }
    for (i, line) in lines.iter().enumerate() {
        let row = start_y + 2 + i as u16;
        let payload = format!(" {:<width$} ", line, width = inner_w - 2);
        let is_item = i >= 2;
        let selected = is_item && (i - 2) == cursor_idx;
        if selected {
            queue!(out,
                cursor::MoveTo(start_x + 1, row),
                sel_bg,
                fg,
                Print(&payload),
                ResetColor,
            )?;
        } else {
            queue!(out,
                cursor::MoveTo(start_x + 1, row),
                bg,
                fg,
                Print(&payload),
                ResetColor,
            )?;
        }
    }
    queue!(out, cursor::MoveTo(start_x, start_y + box_h - 1), bg, border_col, Print(&bot), ResetColor)?;
    Ok(())
}

pub fn render_main_menu<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    cursor_idx: usize,
) -> io::Result<()> {
    let items = ["Continuar", "Salir de la partida"];
    draw_selectable(out, view_w, render_rows, "MENÚ", &items, cursor_idx)
}

pub fn render_startup_menu<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    cursor_idx: usize,
) -> io::Result<()> {
    let items = ["Nueva partida", "Cargar partida"];
    draw_selectable(out, view_w, render_rows, "HACKKID", &items, cursor_idx)
}
