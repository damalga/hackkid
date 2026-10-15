# FANFARE — Game Design Document

Rust terminal engine + Wolfenstein 3D-style raycaster rendered with half-block characters (`▄`), drawn with ratatui over crossterm. The game itself (`crates/hackkid-core`) has no terminal, file, clock or audio code, so other frontends (a browser build) can reuse it.

## Narrative Overview

- **The Event ("The Fanfare"):** A global sonic/acoustic cataclysm occurred at 04:12 UTC. Every conscious human vanished instantly mid-stride, leaving empty scrubs and beeping electronics behind.
- **The Survivors:** Only ICU patients who were in deep, unarousable comas when the event happened survived.
- **Setting:** Abandoned metropolitan hospital (ICU Ward 104, Triage & Reception, Clinical Archive, Sublevel Backup Generator, Rooftop Broadcast Array). The playable ground floor:
  - **North:** ICU 101–106 (each with an en-suite), the corridor, the plant room with the backup generator.
  - **Middle:** the nurse station, the medication room (locked), the doctor's office, supply, the staff changing room (six lockers; one holds the lab access card), the staff break room, the on-call room; the surgery suite behind a keycard door (recovery, scrub room, operating room, sterile storage); the clinical archive (needs archive clearance).
  - **Across the east street:** the research annex, locked with the lab access card. Lab benches, glass cabinets and sample fridges with adrenaline, vitamin shots, antibiotics and suture kits; almost no working lights.
  - **South:** the restroom, the cafeteria and its kitchen, the chapel, the lobby with reception and vending machines, the gift shop, triage.
  - **Outside:** the plaza and car park, the streets with abandoned cars, the ambulance bay, the service yard.
- **Characters:**
  - **Olivia**, the protagonist: wakes from a coma in ICU 104.
  - **Julian ("Jules")**, early twenties, a biology student. Woke a week ago in ICU 106 and has been checking on Olivia every day. Curious, talkative, a bit anxious; has theories about the tone. Glasses, a week of stubble, a hoodie from a staff locker, his hospital wristband still on. Keeps to the nurse station; gives Olivia the ward map.
  - **Selenia**, late fifties, the first to wake: 23 days before Olivia, and counting. Practical, dry, quietly kind. Grey hair pinned up, a long cardigan, a canvas tote she scavenges with. Camps in the on-call room; knows where the food is and that the triage keycard is in the triage desk; gives Olivia water.

## Gameplay Loop

- Explore the hospital in first person.
- Rest (sit/lie down) to regenerate stats.
- Search furniture for supplies, Project Zomboid style: bedside tables, bathroom cabinets, fridges, lockers, desks, shelves, crash carts. What's inside is rolled for each new game; most cupboards hold little. Food, water, painkillers, bandages, gauze, peroxide, batteries.
- Keycards: the Triage Keycard (in the triage desk) opens surgery and the medication room; Archive Clearance (in the doctor's desk) opens the clinical archive; the Lab Access Card (a random changing-room locker) opens the research annex.
- The annex's finds: adrenaline shot (breath to full, a jolt to the mind, thirstier), vitamin shot (body +15, mind +5), antibiotics (body +8 a dose, a little hard on the stomach), suture kit (body +30).
- Looking up and down: the view tilts about 66° either way (the raycaster shears its horizon, right out of the frame); Space levels it. Looking right down you see the toes of your own feet, striding as you walk: socks, or the sneakers (+6% speed) and work boots (+0.18 insulation) found in lockers.
- Walking without holding a key: a double tap of a movement key locks it in until another tap; holding the left button walks forward. Both exist because laptops switch the trackpad off while a key is held.
- Moodles show what needs attention (Thirsty, Tired, Filthy, Weak, Cold, Stomach ache...), getting worse in four steps.
- Movement is by the clock, not the key-repeat rate: walking 1.5 m/s, running 3.2 m/s (needs breath, costs it, and a loaded backpack slows you), crouching 0.75 m/s. The bag doesn't pause the game.
- Temperature: outdoors follows the hour (about 4 °C before dawn, 18 °C mid-afternoon, 3 °C less under cloud); indoors the heating is dead, so the building sits around 15 °C. What you wear insulates (gown 0.12, scrubs 0.3, heavy jacket 0.7, trousers +0.22); running makes heat. Freezing costs body, overheating costs water.
- Painkillers: each ibuprofen (28) or paracetamol (16) adds to a stomach score that wears off at 3/h; above 50 you're queasy and the mind suffers, above 80 it's an ulcer and body drains. Eating settles it.
- The flashlight: in the nurse station's crash cart (also a small chance in lockers and desks). 40 game hours of batteries; spare batteries in shelves, shops and desks. Its beam is the only light during a brownout.
- Brownouts: about once every three game hours the backup generator stumbles and every tube goes out for 3–8 s, then stutters back. Exit signs and street lamps stay on.
- Identical consumables stack (food and drink ×5, first aid ×6, pills and paper pool their doses).
- Julian and Selenia wander their rooms by day and keep still at night; they stop short of you rather than walk into you.
- Ambient events: every hour or two, with nothing else on screen, a line about a sound in the empty hospital or street; they go into the message log (Tab).
- Mind recovers from the transmissions playing (+2.5/h), food, pills and rest.
- Interact with Julian in ICU Ward 104.
- Use the Emergency Broadcast Terminal: 13 audio transmissions, the tracks of Antwood's *Fanfare* (`april_14` … `waltz_no._2`), all unlocked. A transmission keeps playing after the screen is closed and carries on to the next one, while the battery lasts. The battery charges next to any outlet and with the Battery Pack.
- Watch the weather: it changes every 6 game hours (clear or overcast), visible outdoors and through windows.
- Time: 2 real minutes are one game hour. Thirst, hunger, sleep debt and hygiene build up; body and mind wear down and you die at zero.

## Controls

| Key | Action |
|-----|--------|
| W / S, ↑ / ↓ | Walk (hold to keep walking) |
| A / D, ← / → | Strafe |
| , / . | Turn (keyboard only) |
| Mouse, trackpad | Pointer and two-finger gestures turn and tilt (Space levels) |
| Shift + move | Run |
| E | Use (pick up, search, talk, sit, lie down, open/close door, look out of a window) |
| F | Second action (sleep in a bed, fill flask / wash, coffee, take toilet paper) |
| C | Crouch |
| L | Flashlight |
| Q | Drink from the water flask |
| Tab / I | Bag (↑↓ pick, E use, G drop, B backpack off); the game runs on |
| M | Hospital map |
| Esc | Pause menu (resume, save, load, sound, quit to title, quit) |
| F5 / F9 | Quick save / quick load |
| Ctrl+C | Quit |
| Mouse | Move to look; park at an edge to keep turning; the wheel or two fingers turn (sideways) and tilt (up/down); left click uses, right click second action, middle flashlight; clicks pick and use in menus, the bag and the terminal |
