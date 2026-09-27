use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::ui::frame::SIDE_PAD;

pub fn render_ceiling<W: Write>(
    out: &mut W,
    view_w: usize,
    render_rows: usize,
    start_row: u16,
    show_wake_text: bool,
) -> io::Result<()> {
    let cx = view_w as f64 / 2.0;
    let cy = render_rows as f64 / 2.0;
    let max_d = (cx.powi(2) + cy.powi(2)).sqrt();

    for ty in 0..render_rows {
        queue!(out,
            cursor::MoveTo(0, start_row + ty as u16),
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveTo(SIDE_PAD, start_row + ty as u16),
        )?;
        for tx in 0..view_w {
            let dx = tx as f64 - cx;
            let dy = ty as f64 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let t = (1.0 - d / max_d).max(0.0);
            let base = 25u8;
            let glow = (t.powf(2.5) * 220.0) as u8;
            let r = base.saturating_add(glow);
            let g = base.saturating_add(glow);
            let b = base.saturating_add((glow as f64 * 0.85) as u8);
            queue!(out, SetBackgroundColor(Color::Rgb { r, g, b }), Print(' '))?;
        }
    }

    if show_wake_text {
        let lines = [
            "Abres los ojos, sólo hay techo, una luz fluorescente y silencio.",
            "No sabes cuánto tiempo llevas dormido.",
            "",
            "[Pulsa cualquier tecla]",
        ];
        let center_y = start_row as i32 + (render_rows as i32 / 2) - 2;
        let put_centered = |out: &mut W, y: i32, text: &str| -> io::Result<()> {
            if text.is_empty() { return Ok(()); }
            if y < 0 { return Ok(()); }
            let x = SIDE_PAD + (view_w.saturating_sub(text.chars().count()) as u16 / 2);
            queue!(out,
                cursor::MoveTo(x, y as u16),
                SetForegroundColor(Color::Black),
                SetBackgroundColor(Color::Rgb { r: 240, g: 235, b: 210 }),
                Print(text),
                ResetColor,
            )
        };
        for (i, line) in lines.iter().enumerate() {
            put_centered(out, center_y + i as i32, line)?;
        }
    }
    Ok(())
}
