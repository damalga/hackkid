# FANFARE

Terminal game written in Rust. First-person exploration of an abandoned metropolitan hospital after "The Fanfare" acoustic cataclysm. You wake up in ICU Ward 104 with Julian (Jules). Survival in the spirit of Project Zomboid: search cupboards, keep fed, watered, rested and clean, and find the keycards that open the sealed wings.

Full design notes: [`GAMEDESIGN.md`](GAMEDESIGN.md).

## Requirements

- **Rust** ≥ 1.88 (edition 2024). Install via [rustup](https://rustup.rs).
- **Terminal** with truecolor and mouse support, at least **72×20** cells (120×40 or bigger looks best). Smaller windows show a notice until you resize.
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
| X | Interact: pick up, search, talk, sit, lie down, use, open/close a door, look out of a window |
| Z | Second action: sleep in a bed, fill the flask or wash at a sink, take coffee, take toilet paper |
| Q | Drink from the water flask |
| I | Open / close the inventory |
| ↑ ↓ then Enter or X | Use the selected item (inside the inventory) |
| T | Drop the selected item (inside the inventory) |
| B | Take the backpack off (it stays where you leave it) |
| H | Hide |
| M | Mute / unmute |
| Tab | Full HUD (vitals, map, inventory, wardrobe) / compact HUD |
| Esc or Enter | Pause menu: resume, save, load, quit to title, quit |
| Ctrl+S / Ctrl+L | Quick save / quick load |
| Ctrl+C | Quit at once |

On the emergency terminal: ↑ ↓ to pick a transmission, Enter to play or stop it, Esc to close the screen (the broadcast keeps playing while the battery lasts).

## Saves

One save slot, at `~/.local/share/hackkid/save.json` on Linux (`~/Library/Application Support/hackkid/` on macOS, `%APPDATA%\hackkid\` on Windows). Set `HACKKID_SAVE=/some/file.json` to put it elsewhere. Saves from earlier versions can't be loaded (the hospital changed).

## The album on the emergency terminal

The terminal's 13 transmissions are the tracks of Antwood's *Fanfare*. The audio isn't part of this repository: put the album files in `~/.local/share/hackkid/music/` (or `./music/`, or point `HACKKID_MUSIC` at a folder). Any file starting with the track number works, as FLAC, Ogg Vorbis, MP3 or WAV: `01_april_14.flac`, `07 - fanfare.mp3`... Without them the terminal says which file it's missing.

## What you see

The view is drawn in 3D at real-world scale (1 tile = 1 m, eye height 1.6 m, 2.1 m doors, 2.8 m ceilings) and printed with half blocks, two pixels per character cell. Every surface has pixel-art textures at the same density; walls take their finish from the room in front of them (handrails in corridors, tiles in restrooms, wood panelling in the lobby, steel in the operating room, brick and concrete outside). Furniture and cars are solid 3D blocks; people, plants and spindly things are flat sprites; signs, clocks, outlets and extinguishers sit flat on the walls.

Light is real too: a light map works out where each ceiling tube, window and street lamp reaches. Some tubes flicker and some are dead, so parts of the hospital are dark. Daylight follows the clock (2 real minutes per game hour), with sunsets, night and stars, and the weather changes every 6 game hours.

## Editing the hospital

The floor plan is [`crates/hackkid-core/src/game/hospital.txt`](crates/hackkid-core/src/game/hospital.txt), one character per metre: `#` wall, `W` window, `|` column, `D` door, `d` restroom door, `S` locked steel door, `M`/`E` glass entrance doors, `h` hedge; floors `.` corridor, `r` room, `t` tile, `z` terrazzo, `o` operating room, `c` carpet, `m` concrete; outside `,` grass, `:` sidewalk, `=` road, `p` parking, `_` pavers. Ceiling lights are placed automatically. Furniture, containers, signs and lights are in [`level.rs`](crates/hackkid-core/src/game/level.rs); `cargo test` checks that nothing ends up inside a wall or out of reach.

## Project layout

```
crates/hackkid-core   the game: raycaster, hospital, rules, saves as JSON. No terminal, files, clock or audio,
                      so it also compiles to WebAssembly (cargo build -p hackkid-core --target wasm32-unknown-unknown)
crates/hackkid        the terminal frontend: ratatui screen, rodio sound, save file, keyboard and mouse
```

`cargo test` runs the rules (saves, loot, doors and keycards, collisions, the terminal, a reachability walk of the whole hospital), the renderer (sky vs. ceiling, frame time) and checks every screen draws at any terminal size.

## License

Personal use, not distributed.
