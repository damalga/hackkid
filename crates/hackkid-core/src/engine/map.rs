use serde::{Deserialize, Serialize};

/// What occupies one 1 m × 1 m cell: walls, doors, open ground.
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
    /// An outer wall with a window in it: solid, but daylight comes in.
    WindowWall,
    /// The tall hedge around the edge of the grounds.
    Hedge,
}

impl Tile {
    pub fn blocks_movement(&self) -> bool {
        match *self {
            Tile::Floor | Tile::Outdoor | Tile::Sidewalk | Tile::Road | Tile::Garden | Tile::Parking => false,
            Tile::Door { open } | Tile::MainDoor { open } | Tile::BathroomDoor { open } | Tile::OperatingDoor { open } => !open,
            Tile::Wall | Tile::BathroomWall | Tile::PushBox | Tile::HospitalOuterWall | Tile::Column | Tile::WindowWall | Tile::Hedge => true,
        }
    }

    pub fn blocks_sight(&self) -> bool {
        match *self {
            Tile::PushBox => false,
            other => other.blocks_movement(),
        }
    }

    /// A flat stand-in colour, used by the map in the HUD.
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Tile::Wall | Tile::BathroomWall => (195, 215, 235),
            Tile::Door { .. } => (150, 155, 165),
            Tile::MainDoor { .. } => (95, 100, 112),
            Tile::BathroomDoor { .. } => (215, 218, 222),
            Tile::OperatingDoor { .. } => (50, 110, 180),
            Tile::PushBox => (100, 80, 60),
            Tile::Floor => (25, 40, 75),
            Tile::Outdoor | Tile::Garden => (60, 105, 55),
            Tile::Sidewalk => (155, 150, 142),
            Tile::Road => (34, 34, 38),
            Tile::Parking => (60, 60, 64),
            Tile::HospitalOuterWall | Tile::WindowWall => (185, 205, 225),
            Tile::Column => (160, 180, 205),
            Tile::Hedge => (40, 80, 40),
        }
    }

    pub fn is_outdoor(&self) -> bool {
        matches!(self, Tile::Outdoor | Tile::Sidewalk | Tile::Road | Tile::Garden | Tile::Parking)
    }

    /// How tall the tile stands, in metres.
    pub fn render_height(&self) -> f64 {
        match self {
            Tile::Hedge => 2.2,
            _ => crate::engine::camera::WALL_H,
        }
    }

    pub fn is_any_door(&self) -> bool {
        self.door_open().is_some()
    }

    /// `Some(open)` for any kind of door, `None` for everything else.
    pub fn door_open(&self) -> Option<bool> {
        match *self {
            Tile::Door { open } | Tile::MainDoor { open } | Tile::BathroomDoor { open } | Tile::OperatingDoor { open } => Some(open),
            _ => None,
        }
    }

    /// The same kind of door, opened or closed. Other tiles come back unchanged.
    pub fn with_open(self, open: bool) -> Tile {
        match self {
            Tile::Door { .. } => Tile::Door { open },
            Tile::MainDoor { .. } => Tile::MainDoor { open },
            Tile::BathroomDoor { .. } => Tile::BathroomDoor { open },
            Tile::OperatingDoor { .. } => Tile::OperatingDoor { open },
            other => other,
        }
    }
}

/// What a floor is made of. Walls take their finish from the floor in front of them,
/// so a wall is tiled on the restroom side and painted on the corridor side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    /// Corridors: vinyl composition tile; walls with a handrail.
    Corridor,
    /// Patient rooms, stores: sheet vinyl; painted walls with a chair rail.
    Room,
    /// Restrooms, kitchens, scrub rooms: ceramic tile.
    Tile,
    /// Lobby and cafeteria: terrazzo; wood-panelled walls.
    Terrazzo,
    /// Operating room: seamless epoxy; steel wall panels.
    Epoxy,
    /// Offices, chapel: carpet; papered walls.
    Carpet,
    /// Plant room: bare concrete; painted block walls.
    Concrete,
    Grass,
    Sidewalk,
    Asphalt,
    Parking,
    /// Concrete pavers around the entrances.
    Pavers,
}

impl Material {
    /// Polished enough to mirror the ceiling lights.
    pub fn gloss(self) -> f32 {
        match self {
            Material::Corridor => 0.30,
            Material::Room => 0.22,
            Material::Terrazzo => 0.34,
            Material::Epoxy => 0.40,
            Material::Tile => 0.18,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
    materials: Vec<Material>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            tiles: vec![Tile::Floor; width * height],
            materials: vec![Material::Room; width * height],
        }
    }

    /// Reads a plan drawn in text, one character per metre:
    ///
    /// | char | tile | char | floor |
    /// |---|---|---|---|
    /// | `#` | wall | `.` | corridor vinyl |
    /// | `W` | wall with a window | `r` | room vinyl |
    /// | `\|` | column | `t` | ceramic tile |
    /// | `D` | door | `z` | terrazzo |
    /// | `d` | restroom door | `o` | operating-room epoxy |
    /// | `S` | secure steel door | `c` | carpet |
    /// | `M` `E` | glass entrance door | `m` | concrete |
    /// | `h` | hedge | `,` `:` `=` `p` `_` | grass, sidewalk, road, parking, pavers |
    ///
    /// Walls that touch the outside become the building's facade.
    pub fn parse(text: &str) -> Map {
        let rows: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
        let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
        let mut map = Map::new(width, rows.len());
        for (y, row) in rows.iter().enumerate() {
            for x in 0..width {
                let c = row.chars().nth(x).unwrap_or(',');
                let (tile, material) = match c {
                    '#' => (Tile::Wall, Material::Room),
                    'W' => (Tile::WindowWall, Material::Room),
                    '|' => (Tile::Column, Material::Terrazzo),
                    'D' => (Tile::Door { open: false }, Material::Corridor),
                    'd' => (Tile::BathroomDoor { open: false }, Material::Tile),
                    'S' => (Tile::OperatingDoor { open: false }, Material::Corridor),
                    'M' | 'E' => (Tile::MainDoor { open: false }, Material::Pavers),
                    'h' => (Tile::Hedge, Material::Grass),
                    '.' => (Tile::Floor, Material::Corridor),
                    'r' => (Tile::Floor, Material::Room),
                    't' => (Tile::Floor, Material::Tile),
                    'z' => (Tile::Floor, Material::Terrazzo),
                    'o' => (Tile::Floor, Material::Epoxy),
                    'c' => (Tile::Floor, Material::Carpet),
                    'm' => (Tile::Floor, Material::Concrete),
                    ':' => (Tile::Sidewalk, Material::Sidewalk),
                    '=' => (Tile::Road, Material::Asphalt),
                    'p' => (Tile::Parking, Material::Parking),
                    '_' => (Tile::Outdoor, Material::Pavers),
                    _ => (Tile::Garden, Material::Grass),
                };
                map.set(x, y, tile);
                map.set_material(x, y, material);
            }
        }
        // walls with the outside on any side are the facade
        let mut outer = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) != Tile::Wall {
                    continue;
                }
                let touches_outside = [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    nx >= 0 && ny >= 0 && map.get(nx as usize, ny as usize).is_outdoor()
                });
                if touches_outside {
                    outer.push((x, y));
                }
            }
        }
        for (x, y) in outer {
            map.set(x, y, Tile::HospitalOuterWall);
        }
        map
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        if x >= self.width || y >= self.height {
            return Tile::Wall;
        }
        self.tiles[y * self.width + x]
    }

    /// The tile under a world position. Anything off the map counts as wall.
    pub fn at(&self, x: f64, y: f64) -> Tile {
        if x < 0.0 || y < 0.0 {
            return Tile::Wall;
        }
        self.get(x as usize, y as usize)
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) {
        if x < self.width && y < self.height {
            self.tiles[y * self.width + x] = tile;
        }
    }

    pub fn material(&self, x: usize, y: usize) -> Material {
        if x >= self.width || y >= self.height {
            return Material::Grass;
        }
        self.materials[y * self.width + x]
    }

    /// The floor under a world position.
    pub fn material_at(&self, x: f64, y: f64) -> Material {
        if x < 0.0 || y < 0.0 {
            return Material::Grass;
        }
        self.material(x as usize, y as usize)
    }

    pub fn set_material(&mut self, x: usize, y: usize, m: Material) {
        if x < self.width && y < self.height {
            self.materials[y * self.width + x] = m;
        }
    }
}
