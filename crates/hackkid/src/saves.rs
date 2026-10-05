//! Where the save file lives: `$HACKKID_SAVE` if set, otherwise `save.json` in the
//! user's data folder (`~/.local/share/hackkid` on Linux,
//! `~/Library/Application Support/hackkid` on macOS, `%APPDATA%\hackkid` on Windows).

use std::fs;
use std::io;
use std::path::PathBuf;

pub fn save_path() -> PathBuf {
    if let Some(p) = std::env::var_os("HACKKID_SAVE") {
        return PathBuf::from(p);
    }
    dirs::data_dir()
        .map(|d| d.join("hackkid").join("save.json"))
        .unwrap_or_else(|| PathBuf::from("save.json"))
}

pub fn write(json: &str) -> io::Result<PathBuf> {
    let path = save_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    // write next to it first, so a crash mid-write can't destroy the previous save
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(path)
}

pub fn read() -> io::Result<String> {
    fs::read_to_string(save_path())
}
