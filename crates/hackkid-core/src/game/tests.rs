//! Behaviour tests for the rules: saves, doors, collisions, the terminal, and a walk of
//! the whole level to prove everything stays reachable.

use crate::engine::map::Tile;
use crate::engine::raycaster;
use crate::game::action::{Action, dispatch};
use crate::game::items::{Item, ItemKind};
use crate::game::player::{Garment, PLAYER_RADIUS};
use crate::game::tracks::TRACKS;
use crate::game::world::{GameMode, MessageStyle, Request, Weather, World, weather_at};

fn exploring() -> World {
    let mut w = World::new();
    w.mode = GameMode::Exploring;
    w
}

fn place(w: &mut World, x: f64, y: f64, dir: (f64, f64)) {
    w.player.x = x;
    w.player.y = y;
    w.player.face(dir.0, dir.1);
}

fn items(w: &World) -> Vec<ItemKind> {
    w.player.inventory.slots.iter().flatten().map(|i| i.kind).collect()
}

fn take_backpack(w: &mut World) {
    place(w, 14.5, 4.8, (0.0, 1.0));
    w.interact();
    assert!(w.player.has_backpack, "backpack should be picked up");
}

// ---------------------------------------------------------------- saves

#[test]
fn save_and_load_round_trip_without_duplicating_anything() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.vending[0].stock = 1;
    w.fixtures[0].paper_units = 4;
    w.clothing[0].taken = true;
    w.laptop.battery = 33.0;
    w.julian_line = 3;
    w.map.set(22, 12, Tile::Door { open: true });
    w.hour = 15.5;
    let json = w.save_json();

    let mut loaded = World::new();
    loaded.load_json(&json).expect("loads");
    assert_eq!(loaded.mode, GameMode::Exploring);
    assert!(loaded.player.has_backpack);
    assert!(loaded.equippables.is_empty(), "the backpack must not be back on the floor");
    assert_eq!(items(&loaded), items(&w));
    assert_eq!(loaded.vending[0].stock, 1);
    assert_eq!(loaded.fixtures[0].paper_units, 4);
    assert!(loaded.clothing[0].taken);
    assert_eq!(loaded.laptop.battery, 33.0);
    assert_eq!(loaded.julian_line, 3);
    assert_eq!(loaded.map.get(22, 12), Tile::Door { open: true });
    assert_eq!(loaded.hour, 15.5);
    // and the floor is empty where the backpack was
    place(&mut loaded, 14.5, 4.8, (0.0, 1.0));
    assert!(loaded.nearby_equippable_idx().is_none());
}

#[test]
fn loading_from_the_title_keeps_the_saved_position_and_facing() {
    let mut w = exploring();
    place(&mut w, 20.5, 9.5, (-1.0, 0.0));
    let json = w.save_json();

    let mut title = World::new();
    dispatch(&mut title, Action::MenuDown);
    dispatch(&mut title, Action::MenuSelect);
    assert_eq!(title.take_requests(), [Request::Load]);
    title.load_json(&json).unwrap();
    assert_eq!(title.mode, GameMode::Exploring, "no replay of the wake-up scene");
    assert_eq!((title.player.x, title.player.y), (20.5, 9.5));
    assert_eq!(title.player.dir_x, -1.0);
}

#[test]
fn saves_from_other_versions_or_damaged_files_are_refused_without_harm() {
    let mut w = exploring();
    place(&mut w, 20.5, 9.5, (1.0, 0.0));
    let old = r#"{"start_date":"2026-05-01","player_x":13.5,"player_y":4.5}"#;
    assert!(w.load_json(old).unwrap_err().contains("another version"));
    assert!(w.load_json("{ not json").is_err());
    assert_eq!((w.player.x, w.player.y), (20.5, 9.5), "a failed load changes nothing");
}

#[test]
fn saving_is_refused_where_it_makes_no_sense() {
    let mut w = World::new();
    dispatch(&mut w, Action::Save);
    assert!(w.take_requests().is_empty(), "nothing to save on the title screen");
    w.mode = GameMode::Exploring;
    dispatch(&mut w, Action::Save);
    assert_eq!(w.take_requests(), [Request::Save]);
}

#[test]
fn pause_menu_saves_loads_and_quits_to_title() {
    let mut w = exploring();
    dispatch(&mut w, Action::OpenMenu);
    dispatch(&mut w, Action::MenuDown);
    dispatch(&mut w, Action::MenuSelect);
    assert_eq!(w.take_requests(), [Request::Save]);
    assert_eq!(w.mode, GameMode::Exploring);
    dispatch(&mut w, Action::OpenMenu);
    for _ in 0..3 {
        dispatch(&mut w, Action::MenuDown);
    }
    dispatch(&mut w, Action::MenuSelect);
    assert_eq!(w.mode, GameMode::Startup, "'Quit to title' goes to the title, not out of the game");
}

// ---------------------------------------------------------------- doors and walls

#[test]
fn the_street_door_never_closes_on_the_player() {
    let mut w = exploring();
    place(&mut w, 22.5, 60.4, (0.0, -1.0));
    w.update(1_000);
    w.interact();
    assert_eq!(w.map.get(22, 59), Tile::MainDoor { open: true });
    place(&mut w, 22.5, 59.5, (0.0, -1.0));
    for t in 0..20 {
        w.update(2_000 + t * 500);
    }
    assert_eq!(w.map.get(22, 59), Tile::MainDoor { open: true }, "still open while standing in it");
    for _ in 0..12 {
        dispatch(&mut w, Action::MoveForward(1));
    }
    assert!(w.player.y < 59.0, "walked through into the vestibule");
    for t in 0..12 {
        w.update(20_000 + t * 500);
    }
    assert_eq!(w.map.get(22, 59), Tile::MainDoor { open: false }, "closes once the doorway is clear");
}

#[test]
fn a_door_cant_be_closed_from_inside_the_doorway() {
    let mut w = exploring();
    w.map.set(22, 12, Tile::Door { open: true });
    place(&mut w, 22.5, 12.75, (0.0, -1.0));
    w.toggle_door(22, 12);
    assert_eq!(w.map.get(22, 12), Tile::Door { open: true });
}

#[test]
fn walls_keep_the_camera_at_a_distance() {
    let mut w = exploring();
    // walk east into the wall between the first two ICU rooms (x = 16)
    place(&mut w, 13.5, 4.5, (1.0, 0.0));
    for _ in 0..40 {
        dispatch(&mut w, Action::MoveForward(1));
    }
    assert!(w.player.x <= 16.0 - PLAYER_RADIUS + 1e-9, "x = {}", w.player.x);
    assert!(w.player.x > 15.5, "got close to the wall");
}

#[test]
fn furniture_is_solid_but_never_traps_you() {
    let mut w = exploring();
    let julian = (w.npcs[0].x, w.npcs[0].y);
    place(&mut w, julian.0, julian.1 + 1.5, (0.0, -1.0));
    for _ in 0..30 {
        dispatch(&mut w, Action::MoveForward(1));
    }
    assert!(w.player.y > julian.1 + 0.3, "can't walk through Julian");
    // standing up from a sofa puts you back where you were
    let before = (w.player.x, w.player.y);
    w.sit_on_sofa(0);
    assert_eq!(w.mode, GameMode::Sitting);
    w.get_up();
    assert_eq!((w.player.x, w.player.y), before);
    // and from inside an obstacle (an old save, say) you can still walk out
    let s = (w.sofas[0].x, w.sofas[0].y);
    place(&mut w, s.0, s.1, (0.0, -1.0));
    for _ in 0..6 {
        dispatch(&mut w, Action::MoveForward(1));
    }
    assert!(w.player.y < s.1 - 0.5);
}

#[test]
fn the_sealed_ward_opens_with_the_triage_keycard() {
    let mut w = exploring();
    place(&mut w, 33.5, 19.5, (1.0, 0.0));
    w.interact();
    assert_eq!(w.map.get(34, 19), Tile::OperatingDoor { open: false });
    assert!(w.dropped.iter().any(|d| d.item.kind == ItemKind::TriageKeycard), "the keycard is somewhere in the level");
    w.player.has_backpack = true;
    w.player.inventory.add(Item::new(ItemKind::TriageKeycard)).unwrap();
    w.interact();
    assert_eq!(w.map.get(34, 19), Tile::OperatingDoor { open: true });
}

// ---------------------------------------------------------------- rendering

#[test]
fn every_ray_hits_something_even_down_the_longest_road() {
    let w = World::new();
    for (x, y, dx, dy) in [(47.5, 79.5, 0.0, -1.0), (1.5, 79.5, 1.0, -1.0), (58.5, 1.5, -1.0, 1.0)] {
        let r = raycaster::cast(&w.map, x, y, dx, dy, -dy * 0.66, dx * 0.66, 200);
        assert!(r.columns.iter().all(|c| c.hit), "a ray from ({x}, {y}) found nothing");
    }
}

// ---------------------------------------------------------------- messages and dialogue

#[test]
fn olivia_and_julian_are_drawn_differently() {
    let mut w = exploring();
    w.talk_to(0);
    assert_eq!(w.message.as_ref().unwrap().style, MessageStyle::Protagonist);
    assert!(w.message.as_ref().unwrap().text.starts_with("Olivia"));
    w.talk_to(0);
    assert_eq!(w.message.as_ref().unwrap().style, MessageStyle::Other);
    for _ in 0..7 {
        w.talk_to(0);
    }
    assert!(w.player.has_map);
    assert!(!w.message.as_ref().unwrap().text.contains("Press Q"), "Q drinks; the map is in the HUD");
}

#[test]
fn messages_last_long_enough_to_read_and_then_go() {
    let mut w = exploring();
    w.update(1_000);
    w.say("Short.");
    w.update(2_000);
    assert!(w.message.is_some());
    let long = "x ".repeat(80);
    w.say(long);
    w.update(9_000);
    assert!(w.message.is_some(), "long lines stay up longer");
    w.update(30_000);
    assert!(w.message.is_none());
}

// ---------------------------------------------------------------- inventory and wardrobe

#[test]
fn putting_on_clothes_swaps_instead_of_destroying_them() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.player.inventory.slots[5] = Some(Item::new(ItemKind::Scrubs));
    w.use_inventory_slot(5);
    assert_eq!(w.player.wardrobe.body, Some(Garment::Scrubs));
    assert_eq!(w.player.inventory.slots[5].as_ref().map(|i| i.kind), Some(ItemKind::HospitalGown), "the gown goes back in the bag");
    w.player.inventory.slots[4] = Some(Item::new(ItemKind::Pants));
    w.use_inventory_slot(4);
    assert!(w.player.wardrobe.wearing_pants());
    w.update(100);
    assert_eq!(w.player.inventory.capacity, 8, "trousers add two pockets");
}

#[test]
fn nothing_is_lost_when_the_backpack_wont_fit_everything() {
    let mut w = exploring();
    take_backpack(&mut w);
    // take it off, fill the slots by hand, then put it back on
    w.toggle_backpack();
    assert!(!w.player.has_backpack);
    for i in 0..6 {
        w.player.inventory.slots[i] = Some(Item::new(ItemKind::Snack));
    }
    let on_floor_before = w.dropped.len();
    w.interact();
    assert!(w.player.has_backpack);
    assert_eq!(w.dropped.len(), on_floor_before + 3, "the three items that didn't fit are on the floor");
}

#[test]
fn the_battery_pack_charges_the_terminal() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.laptop.battery = 20.0;
    let slot = w.player.inventory.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.kind == ItemKind::PowerSupply)).unwrap();
    w.use_inventory_slot(slot);
    assert_eq!(w.laptop.battery, 70.0);
    assert!(!w.player.inventory.has(ItemKind::PowerSupply), "used up");
}

// ---------------------------------------------------------------- the terminal

#[test]
fn the_terminal_lists_the_album_and_plays_it_through() {
    assert_eq!(TRACKS.len(), 13);
    assert_eq!(TRACKS[0].name, "april_14");
    assert_eq!(TRACKS[6].name, "fanfare");
    assert_eq!(TRACKS[12].name, "waltz_no._2");

    let mut w = exploring();
    take_backpack(&mut w);
    w.open_laptop();
    assert_eq!(w.mode, GameMode::Laptop);
    w.laptop.cursor = 11;
    w.toggle_track();
    assert_eq!(w.music(), Some(11), "every track is unlocked");
    w.close_laptop();
    assert_eq!(w.music(), Some(11), "closing the screen keeps the broadcast going");
    w.music_finished(11);
    assert_eq!(w.music(), Some(12));
    w.music_finished(12);
    assert_eq!(w.music(), None);
}

#[test]
fn a_flat_battery_or_a_dropped_terminal_stops_the_music() {
    let mut w = exploring();
    take_backpack(&mut w);
    place(&mut w, 22.5, 17.5, (0.0, 1.0)); // the hub, away from outlets
    w.laptop.playing = Some(0);
    w.laptop.battery = 0.01;
    w.update(1_000);
    w.update(2_000);
    assert_eq!(w.music(), None);

    w.laptop.battery = 50.0;
    w.laptop.playing = Some(3);
    w.toggle_backpack();
    w.update(3_000);
    assert_eq!(w.music(), None, "the terminal went down with the backpack");
}

#[test]
fn a_dead_battery_charges_by_an_outlet() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.laptop.battery = 0.0;
    let o = (w.outlets[0].x, w.outlets[0].y - 0.6);
    place(&mut w, o.0, o.1, (0.0, 1.0));
    for t in 1..=10 {
        w.update(t * 1_000);
    }
    assert!(w.laptop.battery > 2.0);
}

// ---------------------------------------------------------------- time and weather

#[test]
fn the_weather_changes_over_the_days() {
    let all: Vec<Weather> = (1..10).flat_map(|d| (0..4).map(move |b| weather_at(d, b as f64 * 6.0))).collect();
    assert!(all.contains(&Weather::Clear) && all.contains(&Weather::Cloudy));
}

#[test]
fn a_stalled_frontend_doesnt_skip_hours_of_game_time() {
    let mut w = exploring();
    w.update(1_000);
    let hour = w.hour;
    w.update(10 * 60 * 1_000); // ten real minutes in one go
    assert!(w.hour - hour < 0.01, "at most one second is simulated per update");
}

// ---------------------------------------------------------------- the level

/// Everything sits where it can be seen: no piece of furniture inside a wall.
#[test]
fn nothing_is_placed_inside_a_wall() {
    let w = World::new();
    let mut spots: Vec<(String, f64, f64)> = Vec::new();
    spots.extend(w.beds.iter().map(|o| ("bed".into(), o.x, o.y)));
    spots.extend(w.sofas.iter().map(|o| ("sofa".into(), o.x, o.y)));
    spots.extend(w.benches.iter().map(|o| ("bench".into(), o.x, o.y)));
    spots.extend(w.coat_racks.iter().map(|o| ("coat rack".into(), o.x, o.y)));
    spots.extend(w.fixtures.iter().map(|o| (format!("{:?}", o.kind), o.x, o.y)));
    spots.extend(w.vending.iter().map(|o| ("vending".into(), o.x, o.y)));
    spots.extend(w.receptions.iter().map(|o| ("reception".into(), o.x, o.y)));
    spots.extend(w.burras.iter().map(|o| ("burra".into(), o.x, o.y)));
    spots.extend(w.decors.iter().map(|o| (format!("{:?}", o.kind), o.x, o.y)));
    spots.extend(w.npcs.iter().map(|o| (o.name.clone(), o.x, o.y)));
    spots.extend(w.clothing.iter().map(|o| (o.label().into(), o.x, o.y)));
    spots.extend(w.dropped.iter().map(|o| (o.item.label().into(), o.x, o.y)));
    let bad: Vec<_> = spots.iter().filter(|(_, x, y)| w.map.at(*x, *y).blocks_movement()).collect();
    assert!(bad.is_empty(), "inside a wall: {bad:?}");
}

/// Walks the level on a 0.1-tile grid with every door open and checks that each door,
/// each thing to interact with and each outlet can be reached from the bed you wake in.
#[test]
fn everything_in_the_hospital_can_be_reached() {
    let w = World::new();
    const STEP: f64 = 0.1;
    let (gw, gh) = ((w.map.width as f64 / STEP) as usize, (w.map.height as f64 / STEP) as usize);
    let at = |i: usize, j: usize| ((i as f64 + 0.5) * STEP, (j as f64 + 0.5) * STEP);
    let r = PLAYER_RADIUS;
    let free = |x: f64, y: f64| {
        let wall = [(-r, -r), (r, -r), (-r, r), (r, r)].iter().any(|(dx, dy)| {
            let t = w.map.at(x + dx, y + dy);
            t.blocks_movement() && !t.is_any_door()
        });
        !wall && w.obstacles().iter().all(|o| (x - o.x).powi(2) + (y - o.y).powi(2) >= (o.r + r).powi(2))
    };
    let mut reached = vec![false; gw * gh];
    let start = ((w.player.x / STEP) as usize, (w.player.y / STEP) as usize);
    let mut queue = std::collections::VecDeque::from([start]);
    reached[start.1 * gw + start.0] = true;
    while let Some((i, j)) = queue.pop_front() {
        for (di, dj) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let (ni, nj) = (i as i32 + di, j as i32 + dj);
            if ni < 0 || nj < 0 || ni as usize >= gw || nj as usize >= gh {
                continue;
            }
            let (ni, nj) = (ni as usize, nj as usize);
            let (x, y) = at(ni, nj);
            if !reached[nj * gw + ni] && free(x, y) {
                reached[nj * gw + ni] = true;
                queue.push_back((ni, nj));
            }
        }
    }
    let reachable_within = |ox: f64, oy: f64, reach: f64| {
        let (i0, i1) = (((ox - reach) / STEP).max(0.0) as usize, (((ox + reach) / STEP) as usize).min(gw - 1));
        let (j0, j1) = (((oy - reach) / STEP).max(0.0) as usize, (((oy + reach) / STEP) as usize).min(gh - 1));
        (j0..=j1).any(|j| (i0..=i1).any(|i| {
            let (x, y) = at(i, j);
            reached[j * gw + i] && (x - ox).powi(2) + (y - oy).powi(2) < (reach - 0.05).powi(2)
        }))
    };

    let mut missing: Vec<String> = Vec::new();
    let mut check = |what: String, x: f64, y: f64, reach: f64| {
        if !reachable_within(x, y, reach) {
            missing.push(format!("{what} at ({x:.2}, {y:.2})"));
        }
    };
    for d in &w.doors {
        check(format!("door {:?}", w.map.get(d.tx, d.ty)), d.tx as f64 + 0.5, d.ty as f64 + 0.5, 0.6);
    }
    for o in &w.dropped { check(o.item.label().into(), o.x, o.y, 1.2); }
    for o in &w.equippables { let (x, y) = o.position(); check(o.label(), x, y, 1.4); }
    for o in w.clothing.iter().filter(|c| c.is_pickable()) { check(o.label().into(), o.x, o.y, 1.4); }
    for o in &w.npcs { check(o.name.clone(), o.x, o.y, 2.2); }
    for o in &w.sofas { check("sofa".into(), o.x, o.y, 1.0); }
    for o in &w.benches { check("bench".into(), o.x, o.y, 0.9); }
    for o in &w.beds { check("bed".into(), o.x, o.y, 1.0); }
    for o in &w.fixtures { check(format!("{:?}", o.kind), o.x, o.y, 1.0); }
    for o in &w.vending { check("vending machine".into(), o.x, o.y, 1.4); }
    for o in &w.windows { check("window".into(), o.x, o.y, 1.2); }
    for o in &w.outlets { check("outlet".into(), o.x, o.y, 1.5); }
    assert!(missing.is_empty(), "out of reach: {missing:#?}");
}
