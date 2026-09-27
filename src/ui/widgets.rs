use crossterm::{
    cursor, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::io::{self, Write};

pub fn draw_box<W: Write>(
    out: &mut W,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    title: &str,
) -> io::Result<()> {
    let border_col = SetForegroundColor(Color::Rgb { r: 130, g: 140, b: 160 });
    let title_str = format!(" {} ", title);
    let title_len = title_str.chars().count() as u16;
    let prefix_dashes: u16 = 2;
    let corners: u16 = 2;
    let dashes_after = (width as i32 - corners as i32 - prefix_dashes as i32 - title_len as i32).max(0) as usize;
    let top = format!(
        "┌{}{}{}┐",
        "─".repeat(prefix_dashes as usize),
        title_str,
        "─".repeat(dashes_after),
    );
    let bot = format!("└{}┘", "─".repeat((width - 2) as usize));
    queue!(out, cursor::MoveTo(x, y), border_col, Print(&top), ResetColor)?;
    for r in 1..height - 1 {
        queue!(out,
            cursor::MoveTo(x, y + r),
            border_col,
            Print("│"),
            cursor::MoveTo(x + width - 1, y + r),
            Print("│"),
            ResetColor,
        )?;
    }
    queue!(out,
        cursor::MoveTo(x, y + height - 1),
        border_col,
        Print(&bot),
        ResetColor,
    )?;
    Ok(())
}
