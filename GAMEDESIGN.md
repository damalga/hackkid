# FANFARE — Game Design Document

Rust terminal engine + Wolfenstein 3D-style raycaster rendered with half-block characters (`▄`), drawn with ratatui over crossterm. The game itself (`crates/hackkid-core`) has no terminal, file, clock or audio code, so other frontends (a browser build) can reuse it.

## Narrative Overview

- **The Event ("The Fanfare"):** A global sonic/acoustic cataclysm occurred at 04:12 UTC. Every conscious human vanished instantly mid-stride, leaving empty scrubs and beeping electronics behind.
- **The Survivors:** Only ICU patients who were in deep, unarousable comas when the event happened survived.
- **Setting:** Abandoned metropolitan hospital (ICU Ward 104, Triage & Reception, Clinical Archive, Sublevel Backup Generator, Rooftop Broadcast Array). The playable ground floor:
  - **North:** ICU 101–106 (each with an en-suite), the corridor, the plant room with the backup generator.
  - **Middle:** the nurse station, the medication room (locked), the doctor's office, supply, the staff break room, the on-call room; the surgery suite behind a keycard door (recovery, scrub room, operating room, sterile storage); the clinical archive (needs archive clearance).
  - **South:** the restroom, the cafeteria and its kitchen, the chapel, the lobby with reception and vending machines, the gift shop, triage.
  - **Outside:** the plaza and car park, the streets with abandoned cars, the ambulance bay, the service yard.
- **Characters:** Olivia (Protagonist) and Julian / Jules (Companion).

## Gameplay Loop

- Explore the hospital in first person.
- Rest (sit/lie down) to regenerate stats.
- Search furniture for supplies, Project Zomboid style: bedside tables, bathroom cabinets, fridges, lockers, desks, shelves, crash carts. What's inside is rolled for each new game; most cupboards hold little. Food, water, painkillers, bandages, gauze, peroxide, batteries.
- Keycards: the Triage Keycard (in the triage desk) opens surgery and the medication room; Archive Clearance (in the doctor's desk) opens the clinical archive.
- Moodles show what needs attention (Thirsty, Tired, Filthy, Weak...), getting worse in four steps.
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
| X | Interact (pick up, search, talk, sit, lie down, use, open/close door, look out of a window) |
| Z | Second action (sleep in a bed, fill flask / wash, coffee, take toilet paper) |
| Q | Drink from the water flask |
| I | Open / close inventory |
| ↑ ↓ + Enter / X | Use selected inventory item, or play/stop a transmission on the terminal |
| T | Drop selected item |
| B | Take off the backpack |
| H | Toggle hidden state |
| M | Mute |
| Tab | Full / compact HUD |
| Esc / Enter | Pause menu (resume, save, load, quit to title, quit); Esc also closes the terminal and the inventory |
| Ctrl+S / Ctrl+L | Quick save / quick load |
| Ctrl+C | Quit |
