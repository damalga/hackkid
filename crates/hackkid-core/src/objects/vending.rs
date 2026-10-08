use crate::engine::boxes::Facing;

#[derive(Debug, Clone, Copy)]
pub enum VendingKind {
    Snacks,
    Drinks,
}

#[derive(Debug, Clone)]
pub struct VendingMachine {
    pub x: f64,
    pub y: f64,
    pub kind: VendingKind,
    pub stock: u32,
    pub facing: Facing,
}

impl VendingMachine {
    pub fn new(x: f64, y: f64, kind: VendingKind, stock: u32, facing: Facing) -> Self {
        Self { x, y, kind, stock, facing }
    }
}
