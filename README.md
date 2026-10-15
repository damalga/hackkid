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
| W / S or ↑ / ↓ | Walk forward / back (hold to keep walking; a tap is one step; a double tap keeps you walking until you tap a movement key) |
| A / D or ← / → | Strafe left / right |
| , / . | Turn (for keyboard-only play; the mouse or trackpad is the usual way) |
| Space | Level the view (after looking up or down) |
| Shift + move | Run (uses energy; a loaded backpack slows you) |
| E | Use: pick up, search, talk, sit, lie down, open/close a door, look out of a window |
| F | Second action: sleep in a bed, fill the flask or wash at a sink, take coffee, take toilet paper |
| C | Crouch: lower and slower, keeping out of sight |
| L | Flashlight on / off (once you've found one) |
| Q | Drink from the water flask |
| Tab or I | The bag: ↑↓ pick, E use, G drop, B take the backpack off. The game keeps running and WASD still walks |
| T | The emergency terminal, standing, straight from the bag. On it: Tab switches between the BROADCAST and NET screens, Esc closes it |
| M | Map of the hospital (once Julian has given it to you) |
| Esc | Pause menu: resume, save, load, sound on/off, quit to title, quit |
| F5 / F9 | Quick save / quick load (Ctrl+S / Ctrl+L work too) |
| Ctrl+C | Quit at once |

**Mouse and trackpad.** Move the pointer over the view to look around (no button needed): sideways turns, up and down tilts the view, right down to your feet. Hold the left button to walk forward while you look. Two fingers on a trackpad (or the wheel) also look: sideways turns, with no limit, so that's the way round the full 360°; up and down tilts. Parking the pointer near the left or right edge keeps turning too; move it back to stop. Space levels the view again. Left click uses whatever the prompt offers, right click is the second action, the middle button is the flashlight. Menus, the bag and the terminal's track list take clicks: one to pick, a second to use or play; right-click drops an item, closes the terminal or the menu.

**A note for laptops.** Linux and macOS switch the trackpad off while a key is held ("disable while typing"), so holding W and looking around at once may not work. Double-tap W to lock the walk, or hold the left button to walk, and look with the other hand. On GNOME you can also turn the behaviour off: `gsettings set org.gnome.desktop.peripherals.touchpad disable-while-typing false`.

On the emergency terminal: ↑ ↓ to pick a transmission, E to play or stop it, Esc to close the screen (the broadcast keeps playing while the battery lasts). Tab flips to the **NET** screen, a command line onto the hospital's building network: type `help` for the commands, `scan` to start. ↑ ↓ recall what you typed, PgUp/PgDn (or the wheel) scroll back, Esc leaves a running test, a second Esc closes the terminal.

## Saves

One save slot, at `~/.local/share/hackkid/save.json` on Linux (`~/Library/Application Support/hackkid/` on macOS, `%APPDATA%\hackkid\` on Windows). Set `HACKKID_SAVE=/some/file.json` to put it elsewhere. Saves from earlier versions can't be loaded (the hospital changed).

## The album on the emergency terminal

The terminal's 13 transmissions are the tracks of Antwood's *Fanfare*. The audio isn't part of this repository: put the album files in `~/.local/share/hackkid/music/` (or `./music/`, or point `HACKKID_MUSIC` at a folder). Any file starting with the track number works, as FLAC, Ogg Vorbis, MP3 or WAV: `01_april_14.flac`, `07 - fanfare.mp3`...

Without the files the game is just as playable: each transmission is replaced by a synthesized one (a chord under radio static, the track number in Morse) that plays through and moves on to the next, and the terminal says it's only receiving a carrier tone. Without a sound device at all, the game runs silently.

Warnings from the sound libraries (ALSA underruns and the like) go to `hackkid.log` next to the save instead of over the game.

## Staying alive

Thirst, hunger, sleep and hygiene build up; body and mind wear down and you die at zero (the game tells you what got you). Food, drink, pills and bandages come from the cupboards and fridges; identical things stack in one slot. Painkillers on an empty stomach add up to an ulcer, so eat. Nights are cold, more so outside and under cloud: a gown isn't enough, a heavy jacket and trousers are. Running warms you up and works up a sweat. Some ceiling tubes are dead and the backup generator stumbles now and then, so a flashlight (there's one in the nurse station's crash cart) and spare batteries matter after dark. The transmissions on the terminal are company: they keep your mind together.

Across the east street stands the **research annex**: locked, dark (it was never on the hospital's generator), and holding the best supplies in the game: adrenaline and vitamin shots, antibiotics, suture kits. Its access card is in one of the lockers of the staff changing room, next to supply; which locker changes from game to game. Selenia will tell you as much.

## The network

The hospital's machines are still running, and the terminal's NET screen (T, then Tab) reaches them over the staff Wi-Fi. `scan` finds them: the staff intranet, two workstations and the facilities controller that drives the electronic locks. `connect intranet` reads the notices and the staff directory, which say where the passwords were written down: a sticky note in the reception desk, an IT memo in the doctor's desk. `login <host> <user> <password>` gets you onto a workstation, `files` and `read` show what the staff left there (the next password is in a helpdesk ticket; there's some lore too), and once you're on `doorctl`, `status` lists the locks and `unlock DISP-1` releases the **dispensary** behind the gift shop, a room with no card reader at all and shelves of medicines. The surgery, medication-room and archive locks can be released the same way, as an alternative to their keycards, but the controller asks for the relay code first.

Two skill tests stand in where the paper trail runs out. `override <host>` is a timing test: a cursor sweeps a bar and you press Enter while it's inside the window, three or four times, with the window shrinking and the sweep speeding up; three misses and that host refuses connections for half a minute. The relay code is a memory test: the controller shows a code for a moment, then you type it back, twice. Both run in real time on the shell's screen, and the whole thing costs terminal battery. Everything about the network is invented for the game, and new hosts, commands and tests slot into `crates/hackkid-core/src/game/net.rs`.

## What you see

The view is drawn in 3D at real-world scale (1 tile = 1 m, eye height 1.6 m, 2.1 m doors, 2.8 m ceilings) and printed with half blocks, two pixels per character cell. Every surface has pixel-art textures at the same density; walls take their finish from the room in front of them (handrails in corridors, tiles in restrooms, wood panelling in the lobby, steel in the operating room, brick and concrete outside). Furniture and cars are solid 3D blocks; people, plants and spindly things are flat sprites; signs, clocks, outlets and extinguishers sit flat on the walls.

Light is real too: a light map works out where each ceiling tube, window and street lamp reaches. Some tubes flicker and some are dead, so parts of the hospital are dark. Daylight follows the clock (2 real minutes per game hour), with sunsets, night and stars, and the weather changes every 6 game hours.

## Editing the hospital

The floor plan is [`crates/hackkid-core/src/game/hospital.txt`](crates/hackkid-core/src/game/hospital.txt), one character per metre: `#` wall, `W` window, `|` column, `D` door, `d` restroom door, `S` steel door (which ones are locked, by what card or by the network, is in `level.rs`), `M`/`E` glass entrance doors, `h` hedge; floors `.` corridor, `r` room, `t` tile, `z` terrazzo, `o` operating room, `c` carpet, `m` concrete; outside `,` grass, `:` sidewalk, `=` road, `p` parking, `_` pavers. Ceiling lights are placed automatically. Furniture, containers, signs and lights are in [`level.rs`](crates/hackkid-core/src/game/level.rs); `cargo test` checks that nothing ends up inside a wall or out of reach.

## Project layout

```
crates/hackkid-core   the game: raycaster, hospital, rules, saves as JSON. No terminal, files, clock or audio,
                      so it also compiles to WebAssembly (cargo build -p hackkid-core --target wasm32-unknown-unknown)
crates/hackkid        the terminal frontend: ratatui screen, rodio sound, save file, keyboard and mouse
```

`cargo test` runs the rules (saves, loot, doors and keycards, collisions, the terminal, a reachability walk of the whole hospital), the renderer (sky vs. ceiling, frame time) and checks every screen draws at any terminal size.

## License

Personal use, not distributed.
