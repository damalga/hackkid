use crate::engine::map::{Map, Tile};

pub struct Ray {
    pub hit: bool,
    pub distance: f64,
    pub tile: Tile,
    pub side: Side,
    pub wall_x: f64,
    pub ray_dir_x: f64,
    pub ray_dir_y: f64,
    pub hit_tx: i32,
    pub hit_ty: i32,
    pub open_door: Option<OpenDoorHit>,
}

#[derive(Debug, Clone, Copy)]
pub struct OpenDoorHit {
    pub distance: f64,
    pub tile: Tile,
    pub side: Side,
    pub wall_x: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Side {
    Horizontal,
    Vertical,
}

pub struct RaycastResult {
    pub columns: Vec<Ray>,
}

pub fn cast(
    map: &Map,
    pos_x: f64,
    pos_y: f64,
    dir_x: f64,
    dir_y: f64,
    plane_x: f64,
    plane_y: f64,
    screen_width: usize,
) -> RaycastResult {
    let mut columns = Vec::with_capacity(screen_width);

    for x in 0..screen_width {
        // Camera space: -1.0 (left) to 1.0 (right)
        let camera_x = 2.0 * x as f64 / screen_width as f64 - 1.0;
        let ray_dir_x = dir_x + plane_x * camera_x;
        let ray_dir_y = dir_y + plane_y * camera_x;

        let mut map_x = pos_x as i32;
        let mut map_y = pos_y as i32;

        // DDA step sizes
        let delta_dist_x = if ray_dir_x == 0.0 { f64::MAX } else { (1.0 / ray_dir_x).abs() };
        let delta_dist_y = if ray_dir_y == 0.0 { f64::MAX } else { (1.0 / ray_dir_y).abs() };

        let (step_x, mut side_dist_x) = if ray_dir_x < 0.0 {
            (-1, (pos_x - map_x as f64) * delta_dist_x)
        } else {
            (1, (map_x as f64 + 1.0 - pos_x) * delta_dist_x)
        };
        let (step_y, mut side_dist_y) = if ray_dir_y < 0.0 {
            (-1, (pos_y - map_y as f64) * delta_dist_y)
        } else {
            (1, (map_y as f64 + 1.0 - pos_y) * delta_dist_y)
        };

        let mut hit = false;
        let mut side = Side::Horizontal;
        let mut hit_tile = Tile::Floor;
        let mut open_door: Option<OpenDoorHit> = None;
        let max_steps = 64;

        for _ in 0..max_steps {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_x += step_x;
                side = Side::Vertical;
            } else {
                side_dist_y += delta_dist_y;
                map_y += step_y;
                side = Side::Horizontal;
            }

            if map_x < 0 || map_y < 0 {
                break;
            }
            let tile = map.get(map_x as usize, map_y as usize);
            if tile.is_any_door() && !tile.blocks_sight() && open_door.is_none() {
                let d = match side {
                    Side::Vertical => (map_x as f64 - pos_x + (1.0 - step_x as f64) / 2.0) / ray_dir_x,
                    Side::Horizontal => (map_y as f64 - pos_y + (1.0 - step_y as f64) / 2.0) / ray_dir_y,
                }.abs();
                let raw = match side {
                    Side::Vertical => pos_y + d * ray_dir_y,
                    Side::Horizontal => pos_x + d * ray_dir_x,
                };
                open_door = Some(OpenDoorHit {
                    distance: d,
                    tile,
                    side,
                    wall_x: raw - raw.floor(),
                });
            }
            if tile.blocks_sight() {
                hit = true;
                hit_tile = tile;
                break;
            }
        }

        let distance = if hit {
            match side {
                Side::Vertical => (map_x as f64 - pos_x + (1.0 - step_x as f64) / 2.0) / ray_dir_x,
                Side::Horizontal => (map_y as f64 - pos_y + (1.0 - step_y as f64) / 2.0) / ray_dir_y,
            }
            .abs()
        } else {
            f64::MAX
        };

        let wall_x = if hit {
            let raw = match side {
                Side::Vertical => pos_y + distance * ray_dir_y,
                Side::Horizontal => pos_x + distance * ray_dir_x,
            };
            raw - raw.floor()
        } else {
            0.0
        };

        columns.push(Ray {
            hit, distance, tile: hit_tile, side, wall_x, ray_dir_x, ray_dir_y,
            hit_tx: map_x, hit_ty: map_y,
            open_door,
        });
    }

    RaycastResult { columns }
}
