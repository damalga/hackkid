use crate::engine::textures::DoorStyle;
use crate::game::items::ItemKind;

#[derive(Debug, Clone)]
pub struct Door {
    pub tx: usize,
    pub ty: usize,
    pub style: DoorStyle,
    /// The keycard needed to open it, if it's locked.
    pub lock: Option<ItemKind>,
    pub open_since_ms: Option<u64>,
    /// Entrance doors close by themselves after this long.
    pub auto_close_ms: Option<u64>,
}

impl Door {
    pub fn new(tx: usize, ty: usize, style: DoorStyle) -> Self {
        Self { tx, ty, style, lock: None, open_since_ms: None, auto_close_ms: None }
    }
}
