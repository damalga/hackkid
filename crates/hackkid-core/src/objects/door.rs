use crate::engine::textures::DoorStyle;
use crate::game::items::ItemKind;

/// What keeps a door shut.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lock {
    /// A card reader: the card opens it.
    Card(ItemKind),
    /// An electronic lock with no reader: only the facilities controller releases it.
    Remote,
}

#[derive(Debug, Clone)]
pub struct Door {
    pub tx: usize,
    pub ty: usize,
    pub style: DoorStyle,
    pub lock: Option<Lock>,
    /// The lock's name on the facilities controller, if it's wired to it (see
    /// [`crate::game::net`]). Releasing it there clears `lock` for good.
    pub net_id: Option<&'static str>,
    pub open_since_ms: Option<u64>,
    /// Entrance doors close by themselves after this long.
    pub auto_close_ms: Option<u64>,
}

impl Door {
    pub fn new(tx: usize, ty: usize, style: DoorStyle) -> Self {
        Self { tx, ty, style, lock: None, net_id: None, open_since_ms: None, auto_close_ms: None }
    }
}
