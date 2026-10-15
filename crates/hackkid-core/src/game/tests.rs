//! Behaviour tests for the rules: saves, doors, collisions, the terminal, containers,
//! and a walk of the whole hospital to prove everything stays reachable.

use crate::engine::boxes::BoxInput;
use crate::engine::map::Tile;
use crate::engine::raycaster;
use crate::game::action::{Action, dispatch};
use crate::game::items::{Item, ItemKind};
use crate::game::player::{Garment, Obstacle, PLAYER_RADIUS};
use crate::game::tracks::TRACKS;
use crate::game::world::{GameMode, MessageStyle, Request, Weather, World, weather_at};
use crate::objects::ContainerKind;

fn exploring() -> World {
    let mut w = World::new();
    w.mode = GameMode::Exploring;
    w.rest_return = None;
    w
}

fn place(w: &mut World, x: f64, y: f64, dir: (f64, f64)) {
    let l = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
    w.player.x = x;
    w.player.y = y;
    w.player.face(dir.0 / l, dir.1 / l);
}

fn items(w: &World) -> Vec<ItemKind> {
    w.player.inventory.slots.iter().flatten().map(|i| i.kind).collect()
}

/// Can the player stand here: clear of walls and of everything solid?
fn free(w: &World, x: f64, y: f64) -> bool {
    let r = PLAYER_RADIUS;
    let wall = [(-r, -r), (r, -r), (-r, r), (r, r)].iter().any(|(dx, dy)| {
        let t = w.map.at(x + dx, y + dy);
        t.blocks_movement() && !t.is_any_door()
    });
    !wall && w.obstacles().iter().all(|o| o.distance(x, y) >= r)
}

/// Puts the player somewhere free within `reach` of a footprint, facing it.
fn stand_near(w: &mut World, target: Obstacle, reach: f64) {
    let (cx, cy) = ((target.x0 + target.x1) / 2.0, (target.y0 + target.y1) / 2.0);
    for ring in 1..40 {
        let r = ring as f64 * 0.05;
        for k in 0..32 {
            let a = k as f64 / 32.0 * std::f64::consts::TAU;
            let (x, y) = (cx + a.cos() * (r + (target.x1 - target.x0) / 2.0), cy + a.sin() * (r + (target.y1 - target.y0) / 2.0));
            if target.distance(x, y) < reach - 0.05 && free(w, x, y) {
                place(w, x, y, (cx - x, cy - y));
                return;
            }
        }
    }
    panic!("nowhere to stand near {target:?}");
}

/// Walks (or runs) the player for `ms` milliseconds the way the frontend would: one
/// intent, then the clock moving in 50 ms ticks.
fn walk(w: &mut World, forward: i8, side: i8, run: bool, ms: u64) {
    let start = w.now_ms;
    dispatch(w, Action::Move { forward, side, run, hold_ms: ms as u16 });
    let mut t = start;
    while t < start + ms {
        t += 50;
        w.update(t);
    }
}

fn take_backpack(w: &mut World) {
    place(w, 36.3, 9.6, (1.0, 0.0));
    w.interact();
    assert!(w.player.has_backpack, "backpack should be picked up");
}

fn door(w: &World, pred: impl Fn(&crate::objects::Door) -> bool) -> (usize, usize) {
    let d = w.doors.iter().find(|d| pred(d)).expect("door");
    (d.tx, d.ty)
}

// ---------------------------------------------------------------- saves

#[test]
fn save_and_load_round_trip_without_duplicating_anything() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.vending[0].stock = 1;
    w.fixtures[0].paper_units = 4;
    w.clothing[0].taken = true;
    w.containers[3].items.clear();
    w.containers[3].searched = true;
    w.laptop.battery = 33.0;
    w.julian_line = 3;
    w.map.set(36, 15, Tile::Door { open: true });
    w.hour = 15.5;
    let json = w.save_json();

    let mut loaded = World::new();
    loaded.load_json(&json).expect("loads");
    assert_eq!(loaded.mode, GameMode::Exploring);
    assert!(loaded.player.has_backpack);
    assert!(loaded.equippables.is_empty(), "the backpack must not be back on the sofa");
    assert_eq!(items(&loaded), items(&w));
    assert_eq!(loaded.vending[0].stock, 1);
    assert_eq!(loaded.fixtures[0].paper_units, 4);
    assert!(loaded.clothing[0].taken);
    assert!(loaded.containers[3].items.is_empty() && loaded.containers[3].searched);
    assert_eq!(loaded.containers.len(), w.containers.len());
    assert_eq!(loaded.laptop.battery, 33.0);
    assert_eq!(loaded.julian_line, 3);
    assert_eq!(loaded.map.get(36, 15), Tile::Door { open: true });
    assert_eq!(loaded.hour, 15.5);
}

#[test]
fn loot_differs_between_games_but_survives_a_save() {
    let a = World::with_seed(1);
    let b = World::with_seed(2);
    let contents = |w: &World| w.containers.iter().map(|c| c.items.len()).collect::<Vec<_>>();
    assert_ne!(contents(&a), contents(&b), "a new game rolls new loot");
    let mut c = World::with_seed(2);
    c.mode = GameMode::Exploring;
    let mut d = World::new();
    d.load_json(&c.save_json()).unwrap();
    assert_eq!(contents(&d), contents(&b));
}

#[test]
fn loading_from_the_title_keeps_the_saved_position_and_facing() {
    let mut w = exploring();
    place(&mut w, 19.0, 17.0, (-1.0, 0.0));
    let json = w.save_json();

    let mut title = World::new();
    dispatch(&mut title, Action::MenuDown);
    dispatch(&mut title, Action::MenuSelect);
    assert_eq!(title.take_requests(), [Request::Load]);
    title.load_json(&json).unwrap();
    assert_eq!(title.mode, GameMode::Exploring, "no replay of the wake-up scene");
    assert_eq!((title.player.x, title.player.y), (19.0, 17.0));
    assert_eq!(title.player.dir_x, -1.0);
}

#[test]
fn saves_from_other_versions_or_damaged_files_are_refused_without_harm() {
    let mut w = exploring();
    place(&mut w, 19.0, 17.0, (1.0, 0.0));
    let old = r#"{"version":2,"player_x":13.5,"player_y":4.5}"#;
    assert!(w.load_json(old).unwrap_err().contains("another version"));
    assert!(w.load_json("{ not json").is_err());
    assert_eq!((w.player.x, w.player.y), (19.0, 17.0), "a failed load changes nothing");
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
    assert_eq!(w.take_requests(), [Request::ToggleSound]);
    assert_eq!(w.mode, GameMode::Menu, "the menu stays open after toggling sound");
    dispatch(&mut w, Action::CloseMenu);
    dispatch(&mut w, Action::OpenMenu);
    for _ in 0..4 {
        dispatch(&mut w, Action::MenuDown);
    }
    dispatch(&mut w, Action::MenuSelect);
    assert_eq!(w.mode, GameMode::Startup, "'Quit to title' goes to the title, not out of the game");
}

#[test]
fn you_wake_in_bed_and_get_up_beside_it() {
    let mut w = World::new();
    dispatch(&mut w, Action::MenuSelect);
    assert_eq!(w.mode, GameMode::InitialWake);
    dispatch(&mut w, Action::Wake);
    assert_eq!(w.mode, GameMode::Exploring);
    assert_eq!((w.player.x, w.player.y), crate::game::level::PLAYER_START);
    assert!(free(&w, w.player.x, w.player.y));
}

// ---------------------------------------------------------------- doors and walls

#[test]
fn the_entrance_never_closes_on_the_player() {
    let mut w = exploring();
    let (tx, ty) = door(&w, |d| d.auto_close_ms.is_some() && d.ty == 76);
    place(&mut w, tx as f64 + 0.5, ty as f64 + 1.4, (0.0, -1.0));
    w.update(1_000);
    w.interact();
    assert_eq!(w.map.get(tx, ty).door_open(), Some(true));
    assert_eq!(w.map.get(tx + 1, ty).door_open(), Some(true), "both leaves of the double door open");
    place(&mut w, tx as f64 + 0.5, ty as f64 + 0.5, (0.0, -1.0));
    for t in 0..20 {
        w.update(2_000 + t * 500);
    }
    assert_eq!(w.map.get(tx, ty).door_open(), Some(true), "still open while standing in it");
    walk(&mut w, 1, 0, false, 1_200);
    assert!(w.player.y < ty as f64 - 0.3, "walked through into the lobby");
    for t in 0..14 {
        w.update(20_000 + t * 500);
    }
    assert_eq!(w.map.get(tx, ty).door_open(), Some(false), "closes once the doorway is clear");
}

#[test]
fn a_door_cant_be_closed_from_inside_the_doorway() {
    let mut w = exploring();
    w.map.set(36, 15, Tile::Door { open: true });
    place(&mut w, 36.5, 15.5, (0.0, -1.0));
    w.toggle_door(36, 15);
    assert_eq!(w.map.get(36, 15), Tile::Door { open: true });
}

#[test]
fn movement_is_by_the_clock_not_the_keyboard() {
    use crate::game::player::{RUN_SPEED, WALK_SPEED};
    let mut w = exploring();
    place(&mut w, 30.0, 17.5, (1.0, 0.0));
    walk(&mut w, 1, 0, false, 1_000);
    let walked = w.player.x - 30.0;
    let pace = w.player.speed(false);
    assert!(pace < WALK_SPEED && pace > WALK_SPEED * 0.9, "socks on vinyl: a touch under {WALK_SPEED} m/s");
    assert!((walked - pace).abs() < pace * 0.1, "a second of walking is {pace} m, got {walked}");
    // a tap is one short step; the intent lapses on its own
    place(&mut w, 30.0, 17.5, (1.0, 0.0));
    dispatch(&mut w, Action::Move { forward: 1, side: 0, run: false, hold_ms: 200 });
    for t in 1..=20 {
        w.update(w.now_ms + 50 * (t == 1) as u64 + 50 * (t != 1) as u64);
    }
    assert!((w.player.x - 30.0 - WALK_SPEED * 0.2).abs() < 0.15, "a 200 ms tap is a short step: {}", w.player.x - 30.0);
    // running is a brisk sprint, about three times walking
    place(&mut w, 30.0, 17.5, (1.0, 0.0));
    let pace = w.player.speed(true);
    walk(&mut w, 1, 0, true, 1_000);
    let ran = w.player.x - 30.0;
    assert!(pace < RUN_SPEED && pace > RUN_SPEED * 0.9);
    assert!((ran - pace).abs() < pace * 0.1, "a second of running is {pace} m, got {ran}");
    assert!(ran / walked > 3.5 && ran / walked < 5.2, "run/walk = {}", ran / walked);
    assert!(w.player.stats.stamina < 100.0, "running costs breath");
}

#[test]
fn strafing_sidesteps_and_turning_turns() {
    let mut w = exploring();
    place(&mut w, 30.0, 17.5, (1.0, 0.0));
    walk(&mut w, 0, 1, false, 300);
    assert!(w.player.y > 17.6 && (w.player.x - 30.0).abs() < 1e-9, "facing east, right is south");
    dispatch(&mut w, Action::Turn(-1571));
    assert!(w.player.dir_y < -0.99, "a quarter turn left faces north");
    dispatch(&mut w, Action::TurnFor { dir: 1, hold_ms: 500 });
    for t in 1..=10 {
        w.update(w.now_ms + 50 * t.min(1) + 50 * (t > 1) as u64);
    }
    assert!(w.player.dir_x > 0.2, "half a second of the right arrow turns you about 15°");
    dispatch(&mut w, Action::Halt);
    assert_eq!(w.motion.turn, 0.0);
}

#[test]
fn sitting_faces_the_seat_and_crouching_is_slower() {
    let mut w = exploring();
    place(&mut w, 36.3, 9.6, (1.0, 0.0));
    w.sit_on_sofa(3);
    assert_eq!((w.player.dir_x, w.player.dir_y), (-1.0, 0.0), "the sofa in ICU 104 faces west, into the room");
    w.get_up();
    place(&mut w, 30.0, 17.5, (1.0, 0.0));
    walk(&mut w, 1, 0, false, 500);
    let walked = w.player.x - 30.0;
    dispatch(&mut w, Action::ToggleHidden);
    let x = w.player.x;
    walk(&mut w, 1, 0, false, 500);
    assert!(w.player.x - x < walked * 0.7, "crouching creeps");
    walk(&mut w, 1, 0, true, 100);
    assert!(!w.player.hidden, "sprinting stands you up");
}

#[test]
fn walls_keep_the_camera_at_a_distance() {
    let mut w = exploring();
    place(&mut w, 9.0, 17.5, (-1.0, 0.0));
    walk(&mut w, 1, 0, false, 3_000);
    assert!(w.player.x >= 7.0 + PLAYER_RADIUS - 1e-9, "x = {}", w.player.x);
    assert!(w.player.x < 7.5, "got close to the wall");
}

#[test]
fn furniture_is_solid_but_never_traps_you() {
    let mut w = exploring();
    w.hour = 2.0; // night: Julian keeps still
    let julian = (w.npcs[0].x, w.npcs[0].y);
    place(&mut w, julian.0, julian.1 + 1.6, (0.0, -1.0));
    walk(&mut w, 1, 0, false, 2_000);
    assert!(w.player.y > julian.1 + 0.3, "can't walk through Julian");
    // standing up from a sofa puts you back where you were
    let before = (w.player.x, w.player.y);
    w.sit_on_sofa(0);
    assert_eq!(w.mode, GameMode::Sitting);
    w.get_up();
    assert_eq!((w.player.x, w.player.y), before);
    // and from inside a piece of furniture (an old save, say) you can still walk out
    let s = (w.sofas[0].x, w.sofas[0].y);
    place(&mut w, s.0, s.1, (-1.0, 0.0));
    walk(&mut w, 1, 0, false, 800);
    assert!(w.player.x < s.0 - 0.6, "walked off the sofa");
}

#[test]
fn the_bag_stays_open_while_you_walk_and_time_passes() {
    let mut w = exploring();
    take_backpack(&mut w);
    place(&mut w, 30.0, 17.5, (1.0, 0.0));
    dispatch(&mut w, Action::OpenInventory);
    assert_eq!(w.mode, GameMode::Inventory);
    let hour = w.hour;
    walk(&mut w, 1, 0, false, 1_000);
    assert!(w.player.x > 31.0, "walked with the inventory open");
    assert!(w.hour > hour, "the clock kept running");
    dispatch(&mut w, Action::OpenMenu);
    assert_eq!(w.mode, GameMode::Inventory, "the menu key does nothing with the bag open");
    dispatch(&mut w, Action::CloseInventory);
    dispatch(&mut w, Action::OpenMenu);
    let hour = w.hour;
    w.update(w.now_ms + 5_000);
    assert_eq!(w.hour, hour, "the pause menu still pauses");
}

#[test]
fn identical_things_stack_in_one_slot() {
    let mut w = exploring();
    take_backpack(&mut w);
    let free = |w: &World| w.player.inventory.slots.iter().take(6).filter(|s| s.is_none()).count();
    let before = free(&w);
    for _ in 0..3 {
        w.player.inventory.add(Item::new(ItemKind::Crackers)).unwrap();
    }
    assert_eq!(free(&w), before - 1, "three packs of crackers take one slot");
    assert_eq!(w.player.inventory.count(ItemKind::Crackers), 3);
    w.player.inventory.add(Item::new_with_units(ItemKind::Ibuprofen, 4)).unwrap();
    w.player.inventory.add(Item::new_with_units(ItemKind::Ibuprofen, 3)).unwrap();
    assert_eq!(w.player.inventory.count(ItemKind::Ibuprofen), 7, "pill packs pool their doses");
    assert_eq!(free(&w), before - 2);
    let slot = w.player.inventory.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.kind == ItemKind::Crackers)).unwrap();
    w.use_inventory_slot(slot);
    assert_eq!(w.player.inventory.count(ItemKind::Crackers), 2, "eating takes one off the stack");
    let slots_before = w.player.inventory.slots.iter().flatten().count();
    w.player.inventory.add(Item::new(ItemKind::Laptop)).unwrap();
    assert_eq!(w.player.inventory.slots.iter().flatten().count(), slots_before + 1, "a second terminal takes its own slot");
}

#[test]
fn the_flashlight_lights_the_dark_and_eats_batteries() {
    let mut w = exploring();
    take_backpack(&mut w);
    dispatch(&mut w, Action::ToggleFlashlight);
    assert!(!w.player.flashlight_on, "no flashlight yet");
    let cart = w.containers.iter().position(|c| c.items.iter().any(|i| i.kind == ItemKind::Flashlight)).expect("a flashlight is in the hospital");
    assert_eq!(w.containers[cart].kind, ContainerKind::CrashCart);
    w.player.inventory.add(Item::new(ItemKind::Flashlight)).unwrap();
    dispatch(&mut w, Action::ToggleFlashlight);
    assert!(w.player.flashlight_on && w.flashlight_beam() > 0.99);
    // the beam shows up in the view: the floor ahead is brighter at night
    use crate::engine::renderer::Viewport;
    w.hour = 1.0;
    place(&mut w, 22.0, 50.5, (1.0, 0.0)); // the cross corridor, where a dead tube leaves it dark
    let mut vp = Viewport::new(64, 36);
    let brightness = |vp: &Viewport| (0..64 * 36).map(|i| vp.rgba[i * 4 + 1] as u32).sum::<u32>();
    w.render_view(&mut vp, 0);
    let lit = brightness(&vp);
    w.player.flashlight_on = false;
    w.render_view(&mut vp, 0);
    assert!(lit > brightness(&vp) * 11 / 10, "the beam makes a difference");
    // the batteries run down over a couple of game days of use
    w.player.flashlight_on = true;
    w.player.flashlight_charge = 0.5;
    w.update(w.now_ms + 1_000);
    w.update(w.now_ms + 1_000);
    for _ in 0..40 {
        w.update(w.now_ms + 1_000);
    }
    assert!(!w.player.flashlight_on && w.player.flashlight_charge <= 0.0, "it dies");
    w.player.inventory.add(Item::new(ItemKind::Batteries)).unwrap();
    let slot = w.player.inventory.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.kind == ItemKind::Batteries)).unwrap();
    w.use_inventory_slot(slot);
    assert_eq!(w.player.flashlight_charge, 100.0, "fresh batteries");
    assert!(!w.player.inventory.has(ItemKind::Batteries));
}

#[test]
fn the_generator_stumbles_and_the_lights_go_out_for_a_moment() {
    use crate::game::world::Power;
    let mut w = exploring();
    let mut outages = 0;
    let mut t = 0;
    // a game day and a half, in one-second steps, kept fed and watered
    for _ in 0..(36 * 120) {
        t += 1_000;
        w.player.stats.thirst = 0.0;
        w.player.stats.hunger = 0.0;
        w.player.stats.sleep = 0.0;
        w.update(t);
        if let Power::Out { until_ms } = w.power
            && t < until_ms
            && w.power_level(t) == 0.0
        {
            outages += 1;
        }
    }
    assert!(outages > 3, "the lights went out {outages} s over a day and a half");
    assert_eq!(w.power_level(t + 3_000), 1.0, "and come back");
}

#[test]
fn cold_nights_bite_unless_you_dress_for_them() {
    let mut w = exploring();
    w.hour = 3.0;
    w.weather = Weather::Cloudy;
    place(&mut w, 30.0, 85.0, (0.0, -1.0)); // the car park at night, in a gown
    for _ in 0..240 {
        w.update(w.now_ms + 1_000);
    }
    let gown = w.player.stats.thermal;
    assert!(gown < -50.0, "freezing out there in a gown: {gown}");
    assert!(w.moodles().iter().any(|m| m.label.contains("Cold") || m.label.contains("Freezing")));
    let mut w2 = exploring();
    w2.hour = 3.0;
    w2.weather = Weather::Cloudy;
    w2.player.wardrobe.wear(Garment::HeavyJacket);
    w2.player.wardrobe.wear(Garment::HospitalTrousers);
    place(&mut w2, 30.0, 85.0, (0.0, -1.0));
    for _ in 0..240 {
        w2.update(w2.now_ms + 1_000);
    }
    assert!(w2.player.stats.thermal > gown + 25.0, "the jacket and trousers help: {} vs {gown}", w2.player.stats.thermal);
    assert!(w2.player.stats.body > w.player.stats.body, "the cold costs body");
}

#[test]
fn too_many_pills_on_an_empty_stomach_hurt() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.player.inventory.add(Item::new_with_units(ItemKind::Ibuprofen, 6)).unwrap();
    let slot = w.player.inventory.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.kind == ItemKind::Ibuprofen)).unwrap();
    w.use_inventory_slot(slot);
    assert!(w.moodles().is_empty(), "one pill is fine");
    w.use_inventory_slot(slot);
    w.use_inventory_slot(slot);
    assert!(w.moodles().iter().any(|m| m.label.contains("Stomach") || m.label == "Queasy"), "three in a row: {:?}", w.moodles());
    w.use_inventory_slot(slot);
    let body = w.player.stats.body;
    for _ in 0..60 {
        w.update(w.now_ms + 1_000);
    }
    assert!(w.player.stats.body < body, "an ulcer costs body");
    assert!(w.message.as_ref().is_some_and(|m| m.text.contains("burns")) || w.log.iter().any(|(t, _)| t.contains("burns")));
}

#[test]
fn you_sleep_only_when_tired_and_death_says_why() {
    let mut w = exploring();
    w.player.stats.sleep = 5.0;
    let bed = w.nearby_bed_idx().unwrap_or(0);
    w.sleep_at_bed(bed);
    assert_ne!(w.mode, GameMode::Sleeping, "not tired enough");
    w.player.stats.sleep = 60.0;
    w.sleep_at_bed(bed);
    assert_eq!(w.mode, GameMode::Sleeping);

    let mut w = exploring();
    w.player.stats.thirst = 100.0;
    w.player.stats.body = 0.01;
    w.update(w.now_ms + 1_000);
    w.update(w.now_ms + 1_000);
    assert_eq!(w.mode, GameMode::Dead);
    assert_eq!(w.death.as_deref(), Some("You died of thirst."));
}

#[test]
fn the_survivors_wander_their_rooms_by_day_and_keep_still_at_night() {
    let mut w = exploring();
    w.hour = 12.0;
    let home = w.npcs[0].home;
    let start = (w.npcs[0].x, w.npcs[0].y);
    let mut moved = false;
    for _ in 0..1_800 {
        w.update(w.now_ms + 100);
        let n = &w.npcs[0];
        assert!(n.x >= home.0 - 0.3 && n.x <= home.2 + 0.3 && n.y >= home.1 - 0.3 && n.y <= home.3 + 0.3, "Julian stays home: ({}, {})", n.x, n.y);
        assert!(!w.map.at(n.x, n.y).blocks_movement(), "never inside a wall");
        moved |= (n.x - start.0).abs() + (n.y - start.1).abs() > 0.5;
    }
    assert!(moved, "he walks about during the day");
    w.hour = 2.0;
    let at = (w.npcs[0].x, w.npcs[0].y);
    for _ in 0..100 {
        w.update(w.now_ms + 100);
    }
    assert_eq!((w.npcs[0].x, w.npcs[0].y), at, "still at night");
}

#[test]
fn the_lab_card_is_in_a_changing_room_locker_and_opens_the_annex() {
    let mut w = exploring();
    take_backpack(&mut w);
    let i = w.containers.iter().position(|c| c.items.iter().any(|it| it.kind == ItemKind::LabAccess)).expect("the card is somewhere");
    let c = &w.containers[i];
    assert_eq!(c.kind, ContainerKind::Locker);
    assert!((31.0..38.0).contains(&c.x) && (20.0..27.0).contains(&c.y), "in the changing room: ({}, {})", c.x, c.y);
    // another game may put it in a different locker
    let others: Vec<usize> = (2..40).map(|s| World::with_seed(s).containers.iter().position(|c| c.items.iter().any(|it| it.kind == ItemKind::LabAccess)).unwrap()).collect();
    assert!(others.iter().any(|&o| o != i), "the locker varies with the seed");
    // the annex door is locked without it
    place(&mut w, 75.4, 29.5, (1.0, 0.0));
    w.interact();
    assert_eq!(w.map.get(76, 29).door_open(), Some(false), "locked");
    assert!(w.message.as_ref().unwrap().text.contains("LAB ACCESS"));
    w.player.inventory.add(Item::new(ItemKind::LabAccess)).unwrap();
    w.interact();
    assert_eq!(w.map.get(76, 29).door_open(), Some(true));
    // and the lab is worth it
    let lab: Vec<&crate::objects::Container> = w.containers.iter().filter(|c| c.x > 76.0).collect();
    assert!(lab.len() >= 8, "cabinets and fridges in the annex: {}", lab.len());
    let kinds: Vec<ItemKind> = lab.iter().flat_map(|c| c.items.iter().map(|i| i.kind)).collect();
    assert!(kinds.contains(&ItemKind::Adrenaline) && kinds.contains(&ItemKind::SutureKit), "guaranteed finds: {kinds:?}");
    // the shots do what they say
    w.player.stats.stamina = 10.0;
    w.player.stats.body = 40.0;
    w.player.inventory.add(Item::new(ItemKind::Adrenaline)).unwrap();
    let slot = w.player.inventory.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.kind == ItemKind::Adrenaline)).unwrap();
    w.use_inventory_slot(slot);
    assert_eq!(w.player.stats.stamina, 100.0);
    w.player.inventory.add(Item::new(ItemKind::SutureKit)).unwrap();
    let slot = w.player.inventory.slots.iter().position(|s| s.as_ref().is_some_and(|i| i.kind == ItemKind::SutureKit)).unwrap();
    w.use_inventory_slot(slot);
    assert!(w.player.stats.body >= 70.0);
}

#[test]
fn looking_up_and_down_shears_the_view_and_space_levels_it() {
    use crate::engine::renderer::Viewport;
    let mut w = exploring();
    place(&mut w, 9.0, 17.5, (1.0, 0.0));
    let sky_rows = |w: &mut World| {
        let mut vp = Viewport::new(80, 60);
        w.render_view(&mut vp, 0);
        // the ceiling's depth is small above the horizon; count rows whose centre pixel is ceiling
        (0..60).filter(|&row| vp.depth_at(40, row).is_some_and(|d| d < 6.0) && row < 30).count()
    };
    let level = sky_rows(&mut w);
    dispatch(&mut w, Action::Pitch(400));
    assert!(w.player.pitch > 0.39 && w.player.pitch < 0.41);
    let up = sky_rows(&mut w);
    assert!(up != level, "looking up changes what's above the horizon");
    dispatch(&mut w, Action::Pitch(5_000));
    assert!(w.player.pitch <= crate::engine::camera::MAX_PITCH + 1e-9, "the neck has limits");
    dispatch(&mut w, Action::CenterView);
    assert_eq!(w.player.pitch, 0.0);
}

#[test]
fn looking_straight_down_shows_your_feet_and_what_is_on_them() {
    use crate::engine::renderer::Viewport;
    let mut w = exploring();
    place(&mut w, 9.0, 17.5, (1.0, 0.0));
    dispatch(&mut w, Action::Pitch(-5_000));
    let bottom_centre = |w: &World| {
        let mut vp = Viewport::new(100, 60);
        w.render_view(&mut vp, 0);
        let i = (58 * 100 + 44) * 4; // the left foot's toe, at the bottom of the frame
        (vp.rgba[i], vp.rgba[i + 1], vp.rgba[i + 2])
    };
    let socks = bottom_centre(&w);
    w.player.wardrobe.wear(Garment::Boots);
    let boots = bottom_centre(&w);
    assert_ne!(socks, boots, "the feet are drawn, and boots look different from socks");
    assert!(boots.0 > boots.2, "boots are brown: {boots:?}");
    dispatch(&mut w, Action::CenterView);
    let level = bottom_centre(&w);
    assert_ne!(level, boots, "level, you see the floor, not your boots");
    assert!(w.player.speed(false) < crate::game::player::WALK_SPEED, "feet in boots are slower than the base walking pace");
}

#[test]
fn the_empty_hospital_makes_noises_now_and_then() {
    let mut w = exploring();
    let mut heard = 0;
    for _ in 0..900 {
        w.update(w.now_ms + 1_000); // fifteen real minutes: seven game hours
        if w.message.as_ref().is_some_and(|m| m.text.contains("alarm") || m.text.contains("beeps") || m.text.contains("bangs") || m.text.contains("ringing") || m.text.contains("PA system") || m.text.contains("tile") || m.text.contains("ventilation") || m.text.contains("footsteps") || m.text.contains("ticks")) {
            heard += 1;
            w.message = None;
        }
    }
    assert!(heard >= 2, "heard {heard} things");
    assert!(w.log.len() >= 2, "and they're in the log");
}

#[test]
fn locked_doors_need_their_card() {
    let mut w = exploring();
    place(&mut w, 41.5, 24.5, (1.0, 0.0));
    w.interact();
    assert_eq!(w.map.get(42, 24), Tile::OperatingDoor { open: false }, "surgery is sealed");
    w.player.has_backpack = true;
    w.player.inventory.add(Item::new(ItemKind::ArchiveClearance)).unwrap();
    w.interact();
    assert_eq!(w.map.get(42, 24), Tile::OperatingDoor { open: false }, "the wrong card doesn't help");
    w.player.inventory.add(Item::new(ItemKind::TriageKeycard)).unwrap();
    w.interact();
    assert_eq!(w.map.get(42, 24), Tile::OperatingDoor { open: true });
}

#[test]
fn the_keycards_are_in_desks_waiting_to_be_found() {
    let mut w = exploring();
    take_backpack(&mut w);
    for key in [ItemKind::TriageKeycard, ItemKind::ArchiveClearance] {
        let i = w.containers.iter().position(|c| c.items.iter().any(|it| it.kind == key)).expect("a container holds the card");
        assert_eq!(w.containers[i].kind, ContainerKind::Desk);
        let (x0, y0, x1, y1) = BoxInput::new(w.containers[i].kind.box_kind().unwrap(), w.containers[i].x, w.containers[i].y, w.containers[i].facing).bounds();
        stand_near(&mut w, Obstacle { x0, y0, x1, y1 }, 0.85);
        w.player.inventory.slots.iter_mut().skip(3).for_each(|s| *s = None);
        assert_eq!(w.focus(), Some(crate::game::interact::Focus::Container(i)), "facing the desk");
        w.interact();
        assert!(w.player.inventory.has(key), "searching the desk finds the card");
        assert!(w.containers[i].searched);
    }
}

// ---------------------------------------------------------------- rendering

#[test]
fn every_ray_hits_something_even_down_the_longest_road() {
    let w = World::new();
    for (x, y, dx, dy) in [(70.0, 97.5, 0.0, -1.0), (1.5, 1.5, 1.0, 1.0), (77.5, 1.5, -1.0, 1.0)] {
        let l = f64::hypot(dx, dy);
        let (dx, dy) = (dx / l, dy / l);
        let r = raycaster::cast(&w.map, x, y, dx, dy, -dy * 0.8, dx * 0.8, 200);
        assert!(r.columns.iter().all(|c| c.hit), "a ray from ({x}, {y}) found nothing");
    }
}

#[test]
fn dead_tubes_leave_rooms_darker_and_night_falls_outside() {
    use crate::engine::renderer::Viewport;
    let mut w = World::new();
    let mut vp = Viewport::new(8, 8);
    w.render_view(&mut vp, 0);
    let lit_room = vp.light.sample(34.5, 11.0)[1];
    // somewhere every nearby tube is dead or none reach: the lightmap still gives a floor
    assert!(lit_room > 0.4, "your room is lit ({lit_room})");
    let day = vp.light.sample(30.0, 85.0)[1];
    w.hour = 1.0;
    w.render_view(&mut vp, 0);
    let night = vp.light.sample(30.0, 85.0)[1];
    assert!(night < day * 0.4, "the car park is dark at night ({night} vs {day})");
    let lamp = vp.light.sample(20.0, 85.5)[0];
    assert!(lamp > night * 2.0, "but the street lamps are on ({lamp})");
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
    for _ in 0..8 {
        w.talk_to(0);
    }
    assert!(w.player.has_map);
    // then small talk, starting with what's useful: the backpack is still on the sofa
    w.talk_to(0);
    assert!(w.message.as_ref().unwrap().text.contains("backpack"));
}

#[test]
fn selenia_tells_her_story_and_gives_water_once() {
    use crate::objects::npc::Who;
    let mut w = exploring();
    take_backpack(&mut w);
    let s = w.npcs.iter().position(|n| n.who == Who::Selenia).expect("Selenia is in the hospital");
    let bottles = |w: &World| w.player.inventory.slots.iter().flatten().filter(|i| i.kind == ItemKind::WaterBottle).count();
    w.talk_to(s);
    assert!(w.message.as_ref().unwrap().text.contains("Twenty-three days"));
    for _ in 0..8 {
        w.talk_to(s);
    }
    assert_eq!(bottles(&w), 1, "she hands over a bottle of water");
    for _ in 0..6 {
        w.talk_to(s);
    }
    assert_eq!(bottles(&w), 1, "only once");
    assert!(w.message.as_ref().unwrap().text.starts_with("Selenia"));
}

#[test]
fn messages_last_long_enough_to_read_and_then_go() {
    let mut w = exploring();
    w.update(1_000);
    w.say("Short.");
    w.update(2_000);
    assert!(w.message.is_some());
    w.say("x ".repeat(80));
    w.update(9_000);
    assert!(w.message.is_some(), "long lines stay up longer");
    w.update(30_000);
    assert!(w.message.is_none());
}

#[test]
fn moodles_show_what_needs_attention() {
    let mut w = exploring();
    assert!(w.moodles().is_empty(), "nothing wrong at the start");
    w.player.stats.thirst = 85.0;
    w.player.stats.hygiene = 10.0;
    let m = w.moodles();
    assert_eq!(m[0].label, "Parched");
    assert!(m.iter().any(|m| m.label == "Filthy"));
}

// ---------------------------------------------------------------- inventory and wardrobe

#[test]
fn putting_on_clothes_swaps_instead_of_destroying_them() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.player.inventory.slots[5] = Some(Item::new(ItemKind::Scrubs));
    w.use_inventory_slot(5);
    assert_eq!(w.player.wardrobe.body, Some(Garment::Scrubs));
    assert_eq!(w.player.inventory.slots[5].as_ref().map(|i| i.kind), Some(ItemKind::HospitalGown));
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
fn found_food_and_first_aid_do_something() {
    let mut w = exploring();
    take_backpack(&mut w);
    w.player.stats.hunger = 70.0;
    w.player.stats.body = 50.0;
    w.player.inventory.slots[4] = Some(Item::new(ItemKind::Sandwich));
    w.player.inventory.slots[5] = Some(Item::new(ItemKind::Bandage));
    w.use_inventory_slot(4);
    w.use_inventory_slot(5);
    assert!(w.player.stats.hunger < 40.0);
    assert!(w.player.stats.body > 60.0);
    w.player.inventory.slots[4] = Some(Item::new_with_units(ItemKind::Ibuprofen, 2));
    w.use_inventory_slot(4);
    assert_eq!(w.player.inventory.slots[4].as_ref().map(|i| i.units), Some(1), "one pill at a time");
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
    place(&mut w, 40.0, 30.0, (0.0, 1.0)); // the spine corridor, away from outlets
    assert!(!w.near_outlet());
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
    let o = &w.outlets[0];
    let target = Obstacle::around(o.x, o.y, 0.01);
    stand_near(&mut w, target, 1.4);
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
    w.update(10 * 60 * 1_000);
    assert!(w.hour - hour < 0.01, "at most one second is simulated per update");
}

// ---------------------------------------------------------------- the level

/// Everything sits where it can be seen: nothing inside a wall, no furniture inside
/// other furniture.
#[test]
fn nothing_is_placed_inside_a_wall_or_inside_something_else() {
    let w = World::new();
    let mut spots: Vec<(String, f64, f64)> = Vec::new();
    spots.extend(w.fixtures.iter().map(|o| (format!("{:?}", o.kind), o.x, o.y)));
    spots.extend(w.props.iter().map(|o| (format!("{:?}", o.kind), o.x, o.y)));
    spots.extend(w.npcs.iter().map(|o| (o.name.clone(), o.x, o.y)));
    spots.extend(w.clothing.iter().map(|o| (o.label().into(), o.x, o.y)));
    spots.extend(w.dropped.iter().map(|o| (o.item.label().into(), o.x, o.y)));
    spots.extend(w.outlets.iter().map(|o| ("outlet".into(), o.x, o.y)));
    let bad: Vec<_> = spots.iter().filter(|(_, x, y)| w.map.at(*x, *y).blocks_movement()).collect();
    assert!(bad.is_empty(), "inside a wall: {bad:?}");

    let boxes = w.boxes();
    let mut bad = Vec::new();
    for (i, b) in boxes.iter().enumerate() {
        let (x0, y0, x1, y1) = b.bounds();
        let corners = [(x0 + 0.01, y0 + 0.01), (x1 - 0.01, y0 + 0.01), (x0 + 0.01, y1 - 0.01), (x1 - 0.01, y1 - 0.01)];
        if corners.iter().any(|&(x, y)| w.map.at(x, y).blocks_movement()) {
            bad.push(format!("{:?} at ({:.2}, {:.2}) pokes into a wall", b.kind, b.x, b.y));
        }
        for c in &boxes[i + 1..] {
            let (a0, b0, a1, b1) = c.bounds();
            if x0 < a1 - 0.01 && a0 < x1 - 0.01 && y0 < b1 - 0.01 && b0 < y1 - 0.01 {
                bad.push(format!("{:?} at ({:.2}, {:.2}) overlaps {:?} at ({:.2}, {:.2})", b.kind, b.x, b.y, c.kind, c.x, c.y));
            }
        }
    }
    assert!(bad.is_empty(), "{bad:#?}");
}

/// Walks the hospital on a 0.1 m grid with every door open, and checks that each door,
/// each thing to use or search, and each outlet can be reached from where you wake.
#[test]
fn everything_in_the_hospital_can_be_reached() {
    let w = World::new();
    const STEP: f64 = 0.1;
    let (gw, gh) = ((w.map.width as f64 / STEP) as usize, (w.map.height as f64 / STEP) as usize);
    let at = |i: usize, j: usize| ((i as f64 + 0.5) * STEP, (j as f64 + 0.5) * STEP);
    let mut reached = vec![false; gw * gh];
    let (sx, sy) = crate::game::level::PLAYER_START;
    let start = ((sx / STEP) as usize, (sy / STEP) as usize);
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
            if !reached[nj * gw + ni] && free(&w, x, y) {
                reached[nj * gw + ni] = true;
                queue.push_back((ni, nj));
            }
        }
    }
    let reachable_within = |t: Obstacle, reach: f64| {
        let (i0, i1) = (((t.x0 - reach) / STEP).max(0.0) as usize, (((t.x1 + reach) / STEP) as usize).min(gw - 1));
        let (j0, j1) = (((t.y0 - reach) / STEP).max(0.0) as usize, (((t.y1 + reach) / STEP) as usize).min(gh - 1));
        (j0..=j1).any(|j| (i0..=i1).any(|i| {
            let (x, y) = at(i, j);
            reached[j * gw + i] && t.distance(x, y) < reach - 0.05
        }))
    };
    let point = |x: f64, y: f64| Obstacle::around(x, y, 0.0);
    let rect = |b: BoxInput| {
        let (x0, y0, x1, y1) = b.bounds();
        Obstacle { x0, y0, x1, y1 }
    };

    let mut missing: Vec<String> = Vec::new();
    let mut check = |what: String, t: Obstacle, reach: f64| {
        if !reachable_within(t, reach) {
            missing.push(format!("{what} at ({:.2}, {:.2})", (t.x0 + t.x1) / 2.0, (t.y0 + t.y1) / 2.0));
        }
    };
    for d in &w.doors {
        check(format!("door {:?}", w.map.get(d.tx, d.ty)), point(d.tx as f64 + 0.5, d.ty as f64 + 0.5), 0.6);
    }
    for o in &w.dropped { check(o.item.label().into(), point(o.x, o.y), 1.2); }
    for o in &w.equippables { let (x, y) = o.position(); check(o.label(), point(x, y), 1.4); }
    for o in w.clothing.iter().filter(|c| c.is_pickable()) { check(o.label().into(), point(o.x, o.y), 1.4); }
    for o in &w.npcs { check(o.name.clone(), point(o.x, o.y), 2.2); }
    for o in &w.sofas { check("sofa".into(), rect(BoxInput::new(crate::engine::boxes::BoxKind::SofaSeat, o.x, o.y, o.facing)), 0.8); }
    for o in &w.benches { check("bench".into(), rect(BoxInput::new(crate::engine::boxes::BoxKind::BenchSeat, o.x, o.y, o.facing)), 0.8); }
    for o in &w.beds { check("bed".into(), rect(BoxInput::new(crate::engine::boxes::BoxKind::Bed, o.x, o.y, o.facing)), 0.8); }
    for o in &w.fixtures { check(format!("{:?}", o.kind), point(o.x, o.y), 1.0); }
    for o in &w.vending { check("vending machine".into(), rect(BoxInput::new(crate::engine::boxes::BoxKind::VendingSnacks, o.x, o.y, o.facing)), 0.8); }
    for o in &w.containers {
        let t = match o.kind.box_kind() {
            Some(k) => rect(BoxInput::new(k, o.x, o.y, o.facing)),
            None => point(o.x, o.y),
        };
        check(o.kind.label().into(), t, 0.85);
    }
    for o in &w.outlets { check("outlet".into(), point(o.x, o.y), 1.5); }
    assert!(missing.is_empty(), "out of reach: {missing:#?}");
}
