# Hackkid

Terminal game written in Rust. First-person exploration of an abandoned hospital in the city of Copenlada. You wake up with no clear memory.

Full design notes: [`GAMEDESIGN.md`](GAMEDESIGN.md).

## Requirements

- **Rust** ≥ 1.85 (edition 2024). Install via [rustup](https://rustup.rs):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Terminal** with truecolor and mouse capture support (Alacritty, Kitty, WezTerm, foot, iTerm2, Windows Terminal). Minimum recommended size: **120×40** cells.

## Installation

```sh
git clone https://github.com/damalga/hackkid.git
cd hackkid
cargo build --release
./target/release/hackkid
```

Development iteration:
```sh
cargo run --release
```

Use `--release` — the debug build is too slow to be playable.

## Controls

| Key | Action |
|-----|--------|
| W / S | Move forward / backward |
| A / D | Turn |
| Mouse wheel | Run (2× speed) |
| X | Interact (pick up, talk, sit, lie down, open door) |
| I | Open / close backpack |
| ↑ ↓ | Navigate slot (backpack open) |
| Enter | Use selected item / menu |
| T | Drop item (backpack) |
| Z | Sleep (lying down) |
| H | Hide |
| Esc / P | Turn off laptop |
| Ctrl+S | Save (`save.json`) |
| Ctrl+L | Load (`save.json`) |
| Ctrl+C | Quit |

## Code layout

```
src/
├── main.rs           bootstrap + game loop
├── engine/           renderer, map, sprites
├── game/             world, player, stats, input, action dispatch, save/load
├── objects/          beds, sofas, doors, NPCs, fluorescents, outlets, etc.
├── equippables/      backpack (extensible to more equippables)
└── ui/               HUD (4 boxes), message overlay, menus, laptop, ceiling
```

## Saving

Save files persist as `save.json` in the working directory. `Ctrl+S` saves, `Ctrl+L` loads.

## Cross-compiling for Raspberry Pi CM4

CM4 = Cortex-A72 (ARMv8 64-bit). Uses [`cross`](https://github.com/cross-rs/cross) + Docker.

```sh
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
scp target/aarch64-unknown-linux-gnu/release/hackkid pi@<cm4-ip>:~/
```

For 32-bit Raspberry Pi OS, use `armv7-unknown-linux-gnueabihf` instead.

## License

Not defined yet. Personal use, not distributed.
