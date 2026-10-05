# FANFARE — Game Design Document

Rust terminal engine + Wolfenstein 3D-style raycaster rendered with half-block characters (`▄`) over crossterm.

## Narrative Overview

- **The Event ("The Fanfare"):** A global sonic/acoustic cataclysm occurred at 04:12 UTC. Every conscious human vanished instantly mid-stride, leaving empty scrubs and beeping electronics behind.
- **The Survivors:** Only ICU patients who were in deep, unarousable comas when the event happened survived.
- **Setting:** Abandoned metropolitan hospital complex (ICU Ward 104, Triage & Reception, Clinical Archive, Sublevel Backup Generator, Rooftop Broadcast Array).
- **Characters:** Olivia (Protagonist) and Julian / Jules (Companion).

## Gameplay Loop

- Explore the hospital in first person.
- Rest (sit/lie down) to regenerate stats.
- Collect items, keycards, and hospital gear.
- Interact with Julian in ICU Ward 104.
- Use the Emergency Broadcast Terminal (13 Audio Transmissions with week-based release gating starting April 14, 2026).

## Controls

| Key | Action |
|-----|--------|
| W / S | Move forward / backward |
| A / D | Turn |
| Mouse wheel | Run (2× speed) |
| X | Interact (pick up, talk, sit, lie down, open door) |
| I | Open / close inventory |
| ↑ ↓ | Navigate inventory / terminal tracks |
| Enter | Use selected item / toggle audio track playback |
| T | Drop item |
| Z | Sleep / wash |
| H | Toggle hidden state |
| Esc / P | Power off terminal / close menu |
| Ctrl+S | Save game |
| Ctrl+L | Load game |
| Ctrl+C | Quit |
