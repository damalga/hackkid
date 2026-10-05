use crate::game::world::{GameMode, World};

#[derive(Debug, Clone)]
pub enum Action {
    // Movement
    MoveForward(u8),
    MoveBackward(u8),
    RotateLeft,
    RotateRight,

    // Context-sensitive interaction (X)
    Interact,

    // Inventory navigation (only in GameMode::Inventory)
    InventoryUp,
    InventoryDown,
    InventoryUseSelected,
    InventoryDropSelected,

    // Laptop navigation (only in GameMode::Laptop)
    LaptopUp,
    LaptopDown,
    LaptopClose,
    LaptopTogglePlay,

    // Mode transitions
    OpenInventory,
    CloseInventory,
    OpenMenu,
    CloseMenu,
    MenuUp,
    MenuDown,
    MenuSelect,
    Wake,      // any-key wake during InitialWake
    StandUp,   // exit Sitting/Lying
    Sleep,     // sleep 7h while Lying
    SinkWash,  // wash at sink (Z near sink)
    DrinkCanteen,  // quick sip from canteen (Q)
    ToggleBackpack,  // put on / take off backpack

    // Utility
    ToggleHidden,
    Save,
    Load,
    Restart,

    // Auto ticks (fired by main loop on idle)
    RestTick,
    LaptopIdle,
}

pub fn dispatch(world: &mut World, action: Action) {
    if let Action::Save = &action {
        let _ = world.save_to_disk("save.json");
        return;
    }
    if let Action::Load = &action {
        let _ = world.load_from_disk("save.json");
        return;
    }

    match world.mode {
        GameMode::Startup => match action {
            Action::MenuUp => {
                if world.menu_cursor > 0 { world.menu_cursor -= 1; }
            }
            Action::MenuDown => {
                if world.menu_cursor + 1 < 2 { world.menu_cursor += 1; }
            }
            Action::MenuSelect => {
                match world.menu_cursor {
                    0 => {
                        // New game
                        world.mode = GameMode::InitialWake;
                        world.menu_cursor = 0;
                    }
                    1 => {
                        // Load game — search today's save
                        let _ = world.load_latest_save();
                        world.mode = GameMode::InitialWake;
                        world.menu_cursor = 0;
                    }
                    _ => {}
                }
            }
            _ => {}
        },
        GameMode::InitialWake => match action {
            Action::Wake | Action::Interact => world.get_up(),
            _ => {}
        },
        GameMode::Menu => match action {
            Action::CloseMenu => { world.mode = GameMode::Exploring; world.menu_cursor = 0; }
            Action::MenuUp => {
                if world.menu_cursor > 0 { world.menu_cursor -= 1; }
            }
            Action::MenuDown => {
                if world.menu_cursor + 1 < 2 { world.menu_cursor += 1; }
            }
            Action::MenuSelect => {
                match world.menu_cursor {
                    0 => { world.mode = GameMode::Exploring; world.menu_cursor = 0; }
                    1 => { world.should_exit = true; }
                    _ => {}
                }
            }
            _ => {}
        },
        GameMode::Inventory => match action {
            Action::CloseInventory => world.mode = GameMode::Exploring,
            Action::InventoryUp => {
                if world.player.inventory_cursor > 0 {
                    world.player.inventory_cursor -= 1;
                }
            }
            Action::InventoryDown => {
                if world.player.inventory_cursor + 1 < world.player.inventory.capacity {
                    world.player.inventory_cursor += 1;
                }
            }
            Action::InventoryUseSelected => {
                let c = world.player.inventory_cursor;
                world.use_inventory_slot(c);
            }
            Action::InventoryDropSelected => {
                let c = world.player.inventory_cursor;
                world.drop_inventory_slot(c);
            }
            _ => {}
        },
        GameMode::Sitting | GameMode::Lying => match action {
            Action::StandUp | Action::Interact => world.get_up(),
            Action::RestTick => world.rest_tick(),
            Action::Sleep => world.sleep(),
            _ => {}
        },
        GameMode::Sleeping => {}
        GameMode::Dead => match action {
            Action::Restart | Action::MenuSelect | Action::Interact => {
                *world = World::new();
            }
            _ => {}
        },
        GameMode::Laptop => match action {
            Action::LaptopClose => world.mode = GameMode::Exploring,
            Action::LaptopUp => {
                if world.laptop_cursor > 0 { world.laptop_cursor -= 1; }
            }
            Action::LaptopDown => {
                if world.laptop_cursor < 12 { world.laptop_cursor += 1; }
            }
            Action::LaptopTogglePlay => {
                use chrono::{NaiveDate, Utc};
                let start_date = NaiveDate::from_ymd_opt(2026, 4, 14).unwrap();
                let today = Utc::now().date_naive();
                let days = (today - start_date).num_days();
                let current_week = if days < 0 { 1 } else { ((days / 7) + 1).min(12) };
                let idx = world.laptop_cursor;
                let unlocked = (idx + 1 <= current_week as usize) || (idx >= 11 && current_week >= 12);
                if unlocked {
                    if world.laptop_playing_track == Some(idx) {
                        world.laptop_playing_track = None;
                        world.set_message("Audio transmission stopped.".into(), 8);
                    } else {
                        world.laptop_playing_track = Some(idx);
                        world.set_message(format!("Playing audio track {:02}...", idx + 1), 10);
                    }
                } else {
                    world.set_message("[LOCKED - SIGNAL CARRIER NOT DETECTED]".into(), 10);
                }
            }
            Action::LaptopIdle => world.laptop_idle_tick(),
            _ => {}
        },
        GameMode::Exploring => match action {
            Action::MoveForward(n) => world.player_move_forward(n),
            Action::MoveBackward(n) => world.player_move_backward(n),
            Action::RotateLeft => {
                world.player.rotate(-0.22);
                world.tick();
                world.decay_message();
            }
            Action::RotateRight => {
                world.player.rotate(0.22);
                world.tick();
                world.decay_message();
            }
            Action::ToggleHidden => {
                world.player.hidden = !world.player.hidden;
                world.tick();
            }
            Action::OpenMenu => { world.mode = GameMode::Menu; world.menu_cursor = 0; }
            Action::OpenInventory => {
                if world.player.has_backpack {
                    world.mode = GameMode::Inventory;
                }
            }
            Action::Interact => {
                if !world.pick_nearby()
                    && !world.talk_to_nearby_npc()
                    && !world.sit_on_nearby_sofa()
                    && !world.sit_on_nearby_bench()
                    && !world.lie_on_nearby_bed()
                    && !world.use_nearby_fixture()
                    && !world.use_nearby_vending(false)
                    && !world.observe_window_weather()
                {
                    world.toggle_front_door();
                }
            }
            Action::ToggleBackpack => { world.toggle_backpack(); }
            Action::DrinkCanteen => { world.drink_from_canteen(); }
            Action::SinkWash => {
                if !world.sink_secondary_action()
                    && !world.sleep_at_nearby_bed()
                    && !world.use_nearby_vending(true)
                    && !world.take_paper_from_nearby_toilet()
                {
                    // no-op
                }
            }
            _ => {}
        },
    }
}
