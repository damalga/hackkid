//! The player's preferences, kept between sessions in `settings.json` next to the save.
//! Just the control scheme for now; a flat `key=value` file so nothing has to be parsed.

use std::fs;
use std::io;
use std::path::PathBuf;

use hackkid_core::game::controls::ControlScheme;

use crate::saves;

pub fn path() -> PathBuf {
    saves::save_path().with_file_name("settings.txt")
}

/// The saved control scheme, if there is one.
pub fn read_controls() -> Option<ControlScheme> {
    let text = fs::read_to_string(path()).ok()?;
    text.lines().find_map(|l| match l.trim().split_once('=') {
        Some(("controls", "keys")) => Some(ControlScheme::Keys),
        Some(("controls", "mouse")) => Some(ControlScheme::Mouse),
        _ => None,
    })
}

pub fn write_controls(scheme: ControlScheme) -> io::Result<()> {
    let p = path();
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir)?;
    }
    let name = match scheme {
        ControlScheme::Keys => "keys",
        ControlScheme::Mouse => "mouse",
    };
    fs::write(p, format!("controls={name}\n"))
}
