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
| W / S | Move forward / backward |
| A / D | Strafe left / right |
| ← / → (or drag with the mouse) | Turn |
| ↑ / ↓ | Move forward / backward (the arrows alone also play old-school) |
| Shift + move | Run (uses energy) |
| E | Use: pick up, search, talk, sit, lie down, open/close a door, look out of a window |
| F | Second action: sleep in a bed, fill the flask or wash at a sink, take coffee, take toilet paper |
| C | Crouch: lower and slower, keeping out of sight |
| Q | Drink from the water flask |
| Tab or I | Inventory (↑↓ select, E use, G drop, B take the backpack off) |
| M | Map of the hospital (once Julian has given it to you) |
| Esc | Pause menu: resume, save, load, sound on/off, quit to title, quit |
| F5 / F9 | Quick save / quick load (Ctrl+S / Ctrl+L work too) |
| Ctrl+C | Quit at once |

On the emergency terminal: ↑ ↓ to pick a transmission, E to play or stop it, Esc to close the screen (the broadcast keeps playing while the battery lasts).

## Saves

One save slot, at `~/.local/share/hackkid/save.json` on Linux (`~/Library/Application Support/hackkid/` on macOS, `%APPDATA%\hackkid\` on Windows). Set `HACKKID_SAVE=/some/file.json` to put it elsewhere. Saves from earlier versions can't be loaded (the hospital changed).

## The album on the emergency terminal

The terminal's 13 transmissions are the tracks of Antwood's *Fanfare*. The audio isn't part of this repository: put the album files in `~/.local/share/hackkid/music/` (or `./music/`, or point `HACKKID_MUSIC` at a folder). Any file starting with the track number works, as FLAC, Ogg Vorbis, MP3 or WAV: `01_april_14.flac`, `07 - fanfare.mp3`...

Without the files the game is just as playable: each transmission is replaced by a synthesized one (a chord under radio static, the track number in Morse) that plays through and moves on to the next, and the terminal says it's only receiving a carrier tone. Without a sound device at all, the game runs silently.

Warnings from the sound libraries (ALSA underruns and the like) go to `hackkid.log` next to the save instead of over the game.

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
