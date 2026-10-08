use serde::{Deserialize, Serialize};

use crate::engine::boxes::{BoxKind, Facing};
use crate::game::items::Item;

/// Furniture worth searching, Project Zomboid style: what's inside is rolled when the
/// world is made, and stays there until you take it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerKind {
    /// The mirrored cabinet over a sink: fixed to the wall.
    MedCabinet,
    BedsideTable,
    Fridge,
    Locker,
    Shelf,
    StoreShelf,
    Desk,
    CrashCart,
    FileCabinet,
    GlassCabinet,
    KitchenCupboard,
}

impl ContainerKind {
    pub fn label(self) -> &'static str {
        match self {
            ContainerKind::MedCabinet => "medicine cabinet",
            ContainerKind::BedsideTable => "bedside table",
            ContainerKind::Fridge => "fridge",
            ContainerKind::Locker => "locker",
            ContainerKind::Shelf => "supply shelf",
            ContainerKind::StoreShelf => "shop shelf",
            ContainerKind::Desk => "desk",
            ContainerKind::CrashCart => "crash cart",
            ContainerKind::FileCabinet => "filing cabinet",
            ContainerKind::GlassCabinet => "glass cabinet",
            ContainerKind::KitchenCupboard => "kitchen cupboard",
        }
    }

    /// The box it's drawn as; `None` for wall-mounted ones (they're decals).
    pub fn box_kind(self) -> Option<BoxKind> {
        Some(match self {
            ContainerKind::MedCabinet => return None,
            ContainerKind::BedsideTable => BoxKind::BedsideTable,
            ContainerKind::Fridge => BoxKind::Fridge,
            ContainerKind::Locker => BoxKind::Locker,
            ContainerKind::Shelf => BoxKind::Shelf,
            ContainerKind::StoreShelf => BoxKind::StoreShelf,
            ContainerKind::Desk => BoxKind::Desk,
            ContainerKind::CrashCart => BoxKind::CrashCart,
            ContainerKind::FileCabinet => BoxKind::FileCabinet,
            ContainerKind::GlassCabinet => BoxKind::GlassCabinet,
            ContainerKind::KitchenCupboard => BoxKind::KitchenCounter,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    pub x: f64,
    pub y: f64,
    pub kind: ContainerKind,
    pub facing: Facing,
    pub items: Vec<Item>,
    pub searched: bool,
}

impl Container {
    pub fn new(x: f64, y: f64, kind: ContainerKind, facing: Facing) -> Self {
        Self { x, y, kind, facing, items: Vec::new(), searched: false }
    }
}
