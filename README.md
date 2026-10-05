# FANFARE

Terminal game written in Rust. First-person exploration of an abandoned metropolitan hospital after "The Fanfare" acoustic cataclysm. You wake up in ICU Ward 104 with Julian (Jules).

Full design notes: [`GAMEDESIGN.md`](GAMEDESIGN.md).

## Requirements

- **Rust** ≥ 1.85 (edition 2024). Install via [rustup](https://rustup.rs):
  ```sh
  cargo build --release
  ```
- **Terminal** with truecolor and mouse capture support. Minimum recommended size: **120×40** cells.

## Building & Running

```sh
cargo build --release
./target/release/fanfare
```
Or development iteration:
```sh
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| W / S | Move forward / backward |
| A / D | Turn |
| Mouse wheel | Run (2× speed) |
| X | Interact (pick up, talk, sit, lie down, open door) |
| I | Open / close inventory |
| ↑ ↓ | Navigate slot |
| Enter | Use selected item / toggle terminal playback |
| T | Drop item |
| Z | Sleep (lying down) / wash |
| H | Hide |
| Esc / P | Power off terminal |
| Ctrl+S | Save (`save.json`) |
| Ctrl+L | Load (`save.json`) |
| Ctrl+C | Quit |

## License

Personal use, not distributed.
