use hackkid_core::game::action::Action;
use hackkid_core::game::world::GameMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};

/// Maps a key press to an action for the current mode. Key releases (which Windows
/// reports too) are ignored, so every key acts once.
pub fn key_to_action(key: KeyEvent, mode: GameMode) -> Option<Action> {
    if key.kind == KeyEventKind::Release {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('s') => Some(Action::Save),
            KeyCode::Char('l') => Some(Action::Load),
            _ => None,
        };
    }
    let up = matches!(key.code, KeyCode::Up | KeyCode::Char('w' | 'W'));
    let down = matches!(key.code, KeyCode::Down | KeyCode::Char('s' | 'S'));

    match mode {
        GameMode::Startup | GameMode::Menu => match key.code {
            _ if up => Some(Action::MenuUp),
            _ if down => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            KeyCode::Esc if mode == GameMode::Menu => Some(Action::CloseMenu),
            _ => None,
        },
        GameMode::InitialWake => Some(Action::Wake),
        GameMode::Sleeping => None,
        GameMode::Dead => matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')).then_some(Action::Restart),
        GameMode::Inventory => match key.code {
            KeyCode::Char('i' | 'I') | KeyCode::Esc => Some(Action::CloseInventory),
            _ if up => Some(Action::InventoryUp),
            _ if down => Some(Action::InventoryDown),
            KeyCode::Enter | KeyCode::Char('x' | 'X') => Some(Action::InventoryUseSelected),
            KeyCode::Char('t' | 'T' | 'c' | 'C') => Some(Action::InventoryDropSelected),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            _ => None,
        },
        GameMode::Sitting => matches!(key.code, KeyCode::Char('x' | 'X') | KeyCode::Esc).then_some(Action::StandUp),
        GameMode::Lying => match key.code {
            KeyCode::Char('x' | 'X') | KeyCode::Esc => Some(Action::StandUp),
            KeyCode::Char('z' | 'Z') | KeyCode::Enter => Some(Action::Sleep),
            _ => None,
        },
        GameMode::Laptop => match key.code {
            KeyCode::Esc | KeyCode::Char('p' | 'P') => Some(Action::LaptopClose),
            _ if up => Some(Action::LaptopUp),
            _ if down => Some(Action::LaptopDown),
            KeyCode::Enter | KeyCode::Char('x' | 'X') => Some(Action::LaptopTogglePlay),
            _ => None,
        },
        GameMode::Exploring => match key.code {
            _ if up => Some(Action::MoveForward(1)),
            _ if down => Some(Action::MoveBackward),
            KeyCode::Char('a' | 'A') | KeyCode::Left => Some(Action::RotateLeft),
            KeyCode::Char('d' | 'D') | KeyCode::Right => Some(Action::RotateRight),
            KeyCode::Char('h' | 'H') => Some(Action::ToggleHidden),
            KeyCode::Char('b' | 'B') => Some(Action::ToggleBackpack),
            KeyCode::Enter | KeyCode::Esc => Some(Action::OpenMenu),
            KeyCode::Char('i' | 'I') => Some(Action::OpenInventory),
            KeyCode::Char('x' | 'X') => Some(Action::Interact),
            KeyCode::Char('z' | 'Z') => Some(Action::Secondary),
            KeyCode::Char('q' | 'Q') => Some(Action::DrinkCanteen),
            _ => None,
        },
    }
}

/// The mouse wheel runs, while exploring.
pub fn mouse_to_action(m: MouseEvent, mode: GameMode) -> Option<Action> {
    if mode != GameMode::Exploring {
        return None;
    }
    match m.kind {
        MouseEventKind::ScrollUp => Some(Action::MoveForward(2)),
        MouseEventKind::ScrollDown => Some(Action::MoveBackward),
        _ => None,
    }
}
