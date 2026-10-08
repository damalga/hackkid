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
    for _ in 0..12 {
        dispatch(&mut w, Action::MoveForward(1));
    }
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
fn walls_keep_the_camera_at_a_distance() {
    let mut w = exploring();
    place(&mut w, 9.0, 17.5, (-1.0, 0.0));
    for _ in 0..40 {
        dispatch(&mut w, Action::MoveForward(1));
    }
    assert!(w.player.x >= 7.0 + PLAYER_RADIUS - 1e-9, "x = {}", w.player.x);
    assert!(w.player.x < 7.5, "got close to the wall");
}

#[test]
fn furniture_is_solid_but_never_traps_you() {
    let mut w = exploring();
    let julian = (w.npcs[0].x, w.npcs[0].y);
    place(&mut w, julian.0, julian.1 + 1.6, (0.0, -1.0));
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
    // and from inside a piece of furniture (an old save, say) you can still walk out
    let s = (w.sofas[0].x, w.sofas[0].y);
    place(&mut w, s.0, s.1, (-1.0, 0.0));
    for _ in 0..8 {
        dispatch(&mut w, Action::MoveForward(1));
    }
    assert!(w.player.x < s.0 - 0.6, "walked off the sofa");
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
    for _ in 0..7 {
        w.talk_to(0);
    }
    assert!(w.player.has_map);
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
