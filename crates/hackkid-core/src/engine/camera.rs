//! The camera and the real-world scale everything is built on: 1 map tile is 1 metre.

/// Eye height of someone standing.
pub const EYE_H: f64 = 1.6;
/// ...sitting on a sofa or bench.
pub const SEATED_EYE_H: f64 = 1.15;
/// ...crouching to keep out of sight.
pub const CROUCH_EYE_H: f64 = 1.0;
/// Floor to (suspended) ceiling.
pub const WALL_H: f64 = 2.8;
/// Height of a door opening.
pub const DOOR_H: f64 = 2.1;
/// Texture density for walls, floors, ceilings, furniture and sprites alike: the
/// pixel-art grain is the same everywhere.
pub const TEX_RES: f64 = 20.0;
/// Length of the camera plane: about 77° of horizontal field of view.
pub const FOV_PLANE: f64 = 0.8;

/// How the 3D world maps onto a `w` × `h` pixel frame.
#[derive(Debug, Clone, Copy)]
pub struct View {
    pub w: usize,
    pub h: usize,
    /// Pixels per unit of lateral offset at distance 1.
    pub f_h: f64,
    /// Pixels per metre of height at distance 1.
    pub f_v: f64,
    /// Screen row of the horizon.
    pub horizon: f64,
    /// Eye height above the floor, metres.
    pub eye: f64,
}

impl View {
    /// Square pixels (the terminal's half blocks are), but never less than about 50° of
    /// vertical view, so very wide terminals still show floor and ceiling.
    pub fn new(w: usize, h: usize) -> Self {
        let f_h = w as f64 / (2.0 * FOV_PLANE);
        let f_v = f_h.min(h as f64 * 1.07).max(1.0);
        Self { w, h, f_h, f_v, horizon: h as f64 / 2.0, eye: EYE_H }
    }

    /// Screen row of a point `height` metres above the floor at depth `d`.
    #[inline]
    pub fn row_of(&self, height: f64, d: f64) -> f64 {
        self.horizon - self.f_v * (height - self.eye) / d
    }
}
