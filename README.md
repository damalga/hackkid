# FANFARE

Terminal game written in Rust. First-person exploration of an abandoned metropolitan hospital after "The Fanfare" acoustic cataclysm. You wake up in ICU Ward 104 with Julian (Jules).

Full design notes: [`GAMEDESIGN.md`](GAMEDESIGN.md).

## Requirements

- **Rust** ≥ 1.88 (edition 2024). Install via [rustup](https://rustup.rs).
- **Terminal** with truecolor and mouse support, at least **80×26** cells (120×40 or bigger looks best). Smaller windows show a notice until you resize.
- On Linux, ALSA development headers to build the sound (`alsa-lib-devel` on Fedora, `libasound2-dev` on Debian/Ubuntu). Without a sound device the game runs silently.

## Building & Running

```sh
cargo run --release
```

or build once and run the binary:

```sh
cargo build --release
./target/release/hackkid
```

## Controls

| Key | Action |
|-----|--------|
| W / S or ↑ ↓ | Move forward / backward |
| A / D or ← → | Turn |
| Mouse wheel | Run (2× speed, uses energy) |
| X | Interact: pick up, talk, sit, lie down, use, open/close a door |
| Z | Second action: sleep in a bed, fill the flask or wash at a sink, take coffee, take toilet paper |
| Q | Drink from the water flask |
| I | Open / close the inventory |
| ↑ ↓ then Enter or X | Use the selected item (inside the inventory) |
| T | Drop the selected item (inside the inventory) |
| B | Take the backpack off (it stays where you leave it) |
| H | Hide |
| M | Mute / unmute |
| Esc or Enter | Pause menu: resume, save, load, quit to title, quit |
| Ctrl+S / Ctrl+L | Quick save / quick load |
| Ctrl+C | Quit at once |

On the emergency terminal: ↑ ↓ to pick a transmission, Enter to play or stop it, Esc to close the screen (the broadcast keeps playing while the battery lasts).

## Saves

One save slot, at `~/.local/share/hackkid/save.json` on Linux (`~/Library/Application Support/hackkid/` on macOS, `%APPDATA%\hackkid\` on Windows). Set `HACKKID_SAVE=/some/file.json` to put it elsewhere. Saves from versions before 0.2 can't be loaded.

## The album on the emergency terminal

The terminal's 13 transmissions are the tracks of Antwood's *Fanfare*. The audio isn't part of this repository: put the album files in `~/.local/share/hackkid/music/` (or `./music/`, or point `HACKKID_MUSIC` at a folder). Any file starting with the track number works, as FLAC, Ogg Vorbis, MP3 or WAV: `01_april_14.flac`, `07 - fanfare.mp3`... Without them the terminal says which file it's missing.

## Project layout

```
crates/hackkid-core   the game: raycaster, hospital, rules, saves as JSON. No terminal, files, clock or audio,
                      so it also compiles to WebAssembly (cargo build -p hackkid-core --target wasm32-unknown-unknown)
crates/hackkid        the terminal frontend: ratatui screen, rodio sound, save file, keyboard and mouse
```

`cargo test` runs the rules (saves, doors, collisions, the terminal, a reachability walk of the whole level) and checks every screen draws at any terminal size.

## License

Personal use, not distributed.
