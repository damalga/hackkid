# FANFARE — Game Design Document

Rust terminal engine + Wolfenstein 3D-style raycaster rendered with half-block characters (`▄`), drawn with ratatui over crossterm. The game itself (`crates/hackkid-core`) has no terminal, file, clock or audio code, so other frontends (a browser build) can reuse it.

## Narrative Overview

- **The Event ("The Fanfare"):** A global sonic/acoustic cataclysm occurred at 04:12 UTC. Every conscious human vanished instantly mid-stride, leaving empty scrubs and beeping electronics behind.
- **The Survivors:** Only ICU patients who were in deep, unarousable comas when the event happened survived.
- **Setting:** Abandoned metropolitan hospital complex (ICU Ward 104, Triage & Reception, Clinical Archive, Sublevel Backup Generator, Rooftop Broadcast Array).
- **Characters:** Olivia (Protagonist) and Julian / Jules (Companion).

## Gameplay Loop

- Explore the hospital in first person.
- Rest (sit/lie down) to regenerate stats.
- Collect items, keycards, and hospital gear. The Triage Keycard, left at the reception desk, opens the sealed east ward (operating room and waiting room).
- Interact with Julian in ICU Ward 104.
- Use the Emergency Broadcast Terminal: 13 audio transmissions, the tracks of Antwood's *Fanfare* (`april_14` … `waltz_no._2`), all unlocked. A transmission keeps playing after the screen is closed and carries on to the next one, while the battery lasts. The battery charges next to any outlet and with the Battery Pack.
- Watch the weather: it changes every 6 game hours (clear or overcast), visible outdoors and through windows.
- Time: 2 real minutes are one game hour. Thirst, hunger, sleep debt and hygiene build up; body and mind wear down and you die at zero.

## Controls

| Key | Action |
|-----|--------|
| W / S or ↑ ↓ | Move forward / backward |
| A / D or ← → | Turn |
| Mouse wheel | Run (2× speed, uses energy) |
| X | Interact (pick up, talk, sit, lie down, use, open/close door) |
| Z | Second action (sleep in a bed, fill flask / wash, coffee, take toilet paper) |
| Q | Drink from the water flask |
| I | Open / close inventory |
| ↑ ↓ + Enter / X | Use selected inventory item, or play/stop a transmission on the terminal |
| T | Drop selected item |
| B | Take off the backpack |
| H | Toggle hidden state |
| M | Mute |
| Esc / Enter | Pause menu (resume, save, load, quit to title, quit); Esc also closes the terminal and the inventory |
| Ctrl+S / Ctrl+L | Quick save / quick load |
| Ctrl+C | Quit |
