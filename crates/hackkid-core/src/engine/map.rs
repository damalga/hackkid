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
}
