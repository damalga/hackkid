use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::game::action::Action;
use crate::game::world::GameMode;

/// Maps a key event to an Action based on the current game mode.
/// Returns `None` for keys that should be ignored (or handled elsewhere, e.g. Ctrl+C exit).
pub fn key_to_action(key: KeyEvent, mode: &GameMode) -> Option<Action> {
    if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Save);
    }
    if key.code == KeyCode::Char('l') && key.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Load);
    }

    match mode {
        GameMode::Startup => match key.code {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        GameMode::InitialWake => Some(Action::Wake),

        GameMode::Sleeping => None,

        GameMode::Dead => match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => Some(Action::Restart),
            _ => None,
        },

        GameMode::Menu => match key.code {
            KeyCode::Esc => Some(Action::CloseMenu),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },

        GameMode::Inventory => match key.code {
            KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Esc => {
                Some(Action::CloseInventory)
            }
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::InventoryUp),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                Some(Action::InventoryDown)
            }
            KeyCode::Enter => None,  // Enter only selects (cursor already positioned)
            KeyCode::Char('x') | KeyCode::Char('X') => Some(Action::InventoryUseSelected),
            KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('t') | KeyCode::Char('T') => {
                Some(Action::InventoryDropSelected)
            }
            _ => None,
        },

        GameMode::Sitting => match key.code {
            KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Esc => Some(Action::StandUp),
            _ => None,
        },

        GameMode::Lying => match key.code {
            KeyCode::Char('x') | KeyCode::Char('X') | KeyCode::Esc => Some(Action::StandUp),
            KeyCode::Char('z') | KeyCode::Char('Z') | KeyCode::Enter => Some(Action::Sleep),
            _ => None,
        },

        GameMode::Laptop => match key.code {
            KeyCode::Esc | KeyCode::Char('p') | KeyCode::Char('P') => Some(Action::LaptopClose),
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Some(Action::LaptopUp),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Some(Action::LaptopDown),
            KeyCode::Enter | KeyCode::Char('x') | KeyCode::Char('X') => Some(Action::LaptopTogglePlay),
            _ => None,
        },

        GameMode::Exploring => match key.code {
            KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Up => Some(Action::MoveForward(1)),
            KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Down => {
                Some(Action::MoveBackward(1))
            }
            KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Left => Some(Action::RotateLeft),
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Right => Some(Action::RotateRight),
            KeyCode::Char('h') => Some(Action::ToggleHidden),
            KeyCode::Char('b') | KeyCode::Char('B') => Some(Action::ToggleBackpack),
            KeyCode::Enter => Some(Action::OpenMenu),
            KeyCode::Char('i') | KeyCode::Char('I') => Some(Action::OpenInventory),
            KeyCode::Char('x') | KeyCode::Char('X') => Some(Action::Interact),
            KeyCode::Char('z') | KeyCode::Char('Z') => Some(Action::SinkWash),
            KeyCode::Char('q') | KeyCode::Char('Q') => Some(Action::DrinkCanteen),
            _ => None,
        },
    }
}

/// Maps a mouse event to an Action. Only Exploring mode reacts to wheel scroll.
pub fn mouse_to_action(m: MouseEvent, mode: &GameMode) -> Option<Action> {
    if *mode != GameMode::Exploring {
        return None;
    }
    match m.kind {
        MouseEventKind::ScrollUp => Some(Action::MoveForward(2)),
        MouseEventKind::ScrollDown => Some(Action::MoveBackward(2)),
        _ => None,
    }
}
