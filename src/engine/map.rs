use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Floor,
    Wall,
    BathroomWall,
    Door { open: bool },
    MainDoor { open: bool },
    BathroomDoor { open: bool },
    OperatingDoor { open: bool },
    PushBox,
    Outdoor,
    Sidewalk,
    Road,
    Garden,
    Parking,
    HospitalOuterWall,
    Column,
}

impl Tile {
    pub fn blocks_movement(&self) -> bool {
        match self {
            Tile::Floor => false,
            Tile::Wall => true,
            Tile::BathroomWall => true,
            Tile::Door { open } => !open,
            Tile::MainDoor { open } => !open,
            Tile::BathroomDoor { open } => !open,
            Tile::OperatingDoor { open } => !open,
            Tile::PushBox => true,
            Tile::Outdoor => false,
            Tile::Sidewalk => false,
            Tile::Road => false,
            Tile::Garden => false,
            Tile::Parking => false,
            Tile::HospitalOuterWall => true,
            Tile::Column => true,
        }
    }

    pub fn blocks_sight(&self) -> bool {
        match self {
            Tile::Floor => false,
            Tile::Wall => true,
            Tile::BathroomWall => true,
            Tile::Door { open } => !open,
            Tile::MainDoor { open } => !open,
            Tile::BathroomDoor { open } => !open,
            Tile::OperatingDoor { open } => !open,
            Tile::PushBox => false,
            Tile::Outdoor => false,
            Tile::Sidewalk => false,
            Tile::Road => false,
            Tile::Garden => false,
            Tile::Parking => false,
            Tile::HospitalOuterWall => true,
            Tile::Column => true,
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Tile::Wall => (195, 215, 235),
            Tile::BathroomWall => (205, 225, 240),
            Tile::Door { .. } => (150, 155, 165),
            Tile::MainDoor { .. } => (95, 100, 112),
            Tile::BathroomDoor { .. } => (215, 218, 222),
            Tile::OperatingDoor { .. } => (50, 110, 180),
            Tile::PushBox => (100, 80, 60),
            Tile::Floor => (25, 40, 75),
            Tile::Outdoor => (60, 105, 55),
            Tile::Sidewalk => (155, 150, 142),
            Tile::Road => (34, 34, 38),
            Tile::Garden => (60, 105, 55),
            Tile::Parking => (60, 60, 64),
            Tile::HospitalOuterWall => (185, 205, 225),
            Tile::Column => (160, 180, 205),
        }
    }

    pub fn is_outdoor(&self) -> bool {
        matches!(self, Tile::Outdoor | Tile::Sidewalk | Tile::Road | Tile::Garden | Tile::Parking)
    }

    pub fn render_height(&self) -> f64 {
        match self {
            Tile::HospitalOuterWall => 2.6,
            Tile::Wall => 2.6,
            Tile::BathroomWall => 2.6,
            Tile::Column => 2.4,
            _ => 1.0,
        }
    }

    pub fn is_any_door(&self) -> bool {
        matches!(self, Tile::Door { .. } | Tile::MainDoor { .. } | Tile::BathroomDoor { .. } | Tile::OperatingDoor { .. })
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Floor; width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        if x >= self.width || y >= self.height {
            return Tile::Wall;
        }
        self.tiles[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    pub fn hospital_ward() -> Self {
        let w = 60;
        let h = 82;
        let mut map = Self::new(w, h);

        Self::stamp_origin(&mut map, 10, 0);
        Self::stamp_hub(&mut map, 10, 12);
        Self::stamp_bathrooms(&mut map, 2, 17);
        Self::stamp_right_ward(&mut map, 34, 12);
        Self::stamp_front_ward(&mut map, 20, 23);
        Self::stamp_vestibule(&mut map, 10, 44);
        Self::stamp_around_hospital(&mut map);
        Self::stamp_parking(&mut map);

        map.set(22, 12, Tile::Door { open: false });
        map.set(10, 19, Tile::BathroomDoor { open: false });
        map.set(34, 19, Tile::OperatingDoor { open: false });
        map.set(22, 23, Tile::Door { open: false });
        map.set(22, 44, Tile::Door { open: false });
        map.set(22, 59, Tile::MainDoor { open: false });

        map
    }

    fn stamp_origin(map: &mut Map, x0: usize, y0: usize) {
        let w = 25;
        let h = 13;
        for x in 0..w {
            map.set(x0 + x, y0, Tile::Wall);
            map.set(x0 + x, y0 + h - 1, Tile::Wall);
        }
        for y in 0..h {
            map.set(x0, y0 + y, Tile::Wall);
            map.set(x0 + w - 1, y0 + y, Tile::Wall);
        }
        // Column walls between rooms
        for y in 1..6 {
            for x in [6usize, 12, 18] {
                map.set(x0 + x, y0 + y, Tile::Wall);
            }
        }
        // Cross wall separating rooms from hallway
        for x in 1..w - 1 {
            map.set(x0 + x, y0 + 6, Tile::Wall);
        }
        // Doors to hallway at the center of each room
        for x in [3usize, 9, 15, 21] {
            map.set(x0 + x, y0 + 6, Tile::Door { open: false });
        }
        // Bathroom sub-room in NE corner of each patient room (3 wide × 2 tall)
        // Bathroom interior: (bx1..=bx1+2, y0+1..=y0+2)
        // Walls (all BathroomWall — pintadas blancas):
        //   west (bx1-1, y0+1..=y0+2)
        //   south (bx1..=bx1+2, y0+3) with door at center
        //   north (bx1..=bx1+2, y0)  — segmento del muro exterior
        //   east (bx1+3, y0+1..=y0+2) — segmento del muro de columna entre habitaciones
        let room_starts = [1usize, 7, 13, 19];
        for &start in &room_starts {
            let bx1 = x0 + start + 2;
            let west_wall_x = x0 + start + 1;
            let east_wall_x = x0 + start + 5;
            map.set(west_wall_x, y0 + 1, Tile::BathroomWall);
            map.set(west_wall_x, y0 + 2, Tile::BathroomWall);
            map.set(bx1, y0 + 3, Tile::BathroomWall);
            map.set(bx1 + 1, y0 + 3, Tile::BathroomDoor { open: false });
            map.set(bx1 + 2, y0 + 3, Tile::BathroomWall);
            map.set(bx1, y0, Tile::BathroomWall);
            map.set(bx1 + 1, y0, Tile::BathroomWall);
            map.set(bx1 + 2, y0, Tile::BathroomWall);
            map.set(east_wall_x, y0 + 1, Tile::BathroomWall);
            map.set(east_wall_x, y0 + 2, Tile::BathroomWall);
        }
    }

    fn stamp_hub(map: &mut Map, x0: usize, y0: usize) {
        let w = 25;
        let h = 12;
        for x in 0..w {
            map.set(x0 + x, y0, Tile::Wall);
            map.set(x0 + x, y0 + h - 1, Tile::Wall);
        }
        for y in 0..h {
            map.set(x0, y0 + y, Tile::Wall);
            map.set(x0 + w - 1, y0 + y, Tile::Wall);
        }
        // Two interior columns (tall)
        map.set(x0 + 8, y0 + 6, Tile::Column);
        map.set(x0 + 16, y0 + 6, Tile::Column);
    }

    fn stamp_bathrooms(map: &mut Map, x0: usize, y0: usize) {
        let w = 9;
        let h = 6;
        for x in 0..w {
            map.set(x0 + x, y0, Tile::BathroomWall);
            map.set(x0 + x, y0 + h - 1, Tile::BathroomWall);
        }
        for y in 0..h {
            map.set(x0, y0 + y, Tile::BathroomWall);
        }
        for y in 1..h - 1 {
            for x in [4usize] {
                map.set(x0 + x, y0 + y, Tile::BathroomWall);
            }
        }
    }

    fn stamp_right_ward(map: &mut Map, x0: usize, y0: usize) {
        let w = 11;
        let h = 12;
        for x in 0..w {
            map.set(x0 + x, y0, Tile::BathroomWall);
            map.set(x0 + x, y0 + h - 1, Tile::BathroomWall);
        }
        for y in 0..h {
            map.set(x0 + w - 1, y0 + y, Tile::BathroomWall);
        }
        for y in 1..4 {
            map.set(x0 + 5, y0 + y, Tile::BathroomWall);
        }
        for x in 1..w - 1 {
            map.set(x0 + x, y0 + 4, Tile::BathroomWall);
        }
        // Operating room door (blue) on left, regular door (brown) on right for waiting room
        map.set(x0 + 2, y0 + 4, Tile::OperatingDoor { open: false });
        map.set(x0 + 7, y0 + 4, Tile::Door { open: false });
    }

    fn stamp_front_ward(map: &mut Map, x0: usize, y0: usize) {
        let w = 5;
        let h = 22;
        for x in 0..w {
            map.set(x0 + x, y0, Tile::Wall);
            map.set(x0 + x, y0 + h - 1, Tile::Wall);
        }
        for y in 0..h {
            map.set(x0, y0 + y, Tile::Wall);
            map.set(x0 + w - 1, y0 + y, Tile::Wall);
        }
    }

    fn stamp_around_hospital(map: &mut Map) {
        let w = map.width;
        let h = map.height;
        // Interior bounding boxes (x0, y0, x1, y1) exclusive on end
        let boxes: &[(usize, usize, usize, usize)] = &[
            (10, 0, 35, 12),   // origin rooms
            (10, 12, 35, 24),  // hub
            (2, 17, 11, 23),   // public bathrooms
            (34, 12, 45, 24),  // right ward
            (20, 23, 25, 45),  // front ward corridor
            (10, 44, 35, 60),  // vestibule
        ];
        let mut interior = vec![false; w * h];
        for &(x0, y0, x1, y1) in boxes {
            for y in y0..y1.min(h) {
                for x in x0..x1.min(w) {
                    interior[y * w + x] = true;
                }
            }
        }
        // Default: non-interior Floor → Garden (background)
        for y in 0..h {
            for x in 0..w {
                if !interior[y * w + x] && map.get(x, y) == Tile::Floor {
                    map.set(x, y, Tile::Garden);
                }
            }
        }
        // Sidewalk + Road ring wrapping hospital bounding
        // Bounding of exterior wall footprint: x=2..45, y=0..59 (approx)
        // South ring:
        for x in 0..w {
            if map.get(x, 60) == Tile::Garden { map.set(x, 60, Tile::Sidewalk); }
            if map.get(x, 61) == Tile::Garden { map.set(x, 61, Tile::Road); }
            if map.get(x, 62) == Tile::Garden { map.set(x, 62, Tile::Road); }
            if map.get(x, 63) == Tile::Garden { map.set(x, 63, Tile::Sidewalk); }
        }
        // East ring:
        for y in 0..h {
            if map.get(46, y) == Tile::Garden { map.set(46, y, Tile::Sidewalk); }
            if map.get(47, y) == Tile::Garden { map.set(47, y, Tile::Road); }
            if map.get(48, y) == Tile::Garden { map.set(48, y, Tile::Road); }
            if map.get(49, y) == Tile::Garden { map.set(49, y, Tile::Sidewalk); }
        }
        // Perimeter walls
        for x in 0..w {
            map.set(x, 0, Tile::Wall);
            map.set(x, h - 1, Tile::Wall);
        }
        for y in 0..h {
            map.set(0, y, Tile::Wall);
            map.set(w - 1, y, Tile::Wall);
        }
        // Convert hospital outer Wall tiles (with outdoor neighbor) to tall HospitalOuterWall
        let mut outer: Vec<(usize, usize)> = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if !matches!(map.get(x, y), Tile::Wall) { continue; }
                // Skip perimeter of map (edge walls) — those are not hospital
                if x == 0 || y == 0 || x == w - 1 || y == h - 1 { continue; }
                let mut has_outdoor = false;
                for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h { continue; }
                    if map.get(nx as usize, ny as usize).is_outdoor() {
                        has_outdoor = true; break;
                    }
                }
                if has_outdoor { outer.push((x, y)); }
            }
        }
        for (x, y) in outer {
            map.set(x, y, Tile::HospitalOuterWall);
        }
    }

    fn stamp_parking(map: &mut Map) {
        let w = map.width;
        let h = map.height;
        // South parking lot: 2 rows deep, spanning most of the frontage below the south sidewalk
        let south_y = 65;
        for dy in 0..2 {
            let y = south_y + dy;
            if y >= h { continue; }
            for x in 6..(w - 6) {
                if map.get(x, y) == Tile::Garden {
                    map.set(x, y, Tile::Parking);
                }
            }
        }
        // East parking column next to the east sidewalk (x=49). Two tiles wide.
        let east_x = 51;
        for x in east_x..(east_x + 2).min(w) {
            for y in 6..30 {
                if map.get(x, y) == Tile::Garden {
                    map.set(x, y, Tile::Parking);
                }
            }
        }
    }

    fn stamp_vestibule(map: &mut Map, x0: usize, y0: usize) {
        let w = 25;
        let h = 16;
        for x in 0..w {
            map.set(x0 + x, y0, Tile::Wall);
            map.set(x0 + x, y0 + h - 1, Tile::Wall);
        }
        for y in 0..h {
            map.set(x0, y0 + y, Tile::Wall);
            map.set(x0 + w - 1, y0 + y, Tile::Wall);
        }
        // Interior columns (tall)
        for &cx in &[4usize, 10, 14, 20] {
            for &cy in &[4usize, 11] {
                map.set(x0 + cx, y0 + cy, Tile::Column);
            }
        }
    }
}
