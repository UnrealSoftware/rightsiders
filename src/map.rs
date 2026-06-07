// City map definition and sidewalk math for Rightsiders

pub const MAP_WIDTH: usize = 63;
pub const MAP_HEIGHT: usize = 63;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TileType {
    Road,               // Empty street floor (dark gray)
    Wall(u8),           // Solid building wall (0: Neon Grid, 1: Tech, 2: Billboard, 3: Police HQ)
    SidewalkVert,      // Vertical sidewalk (centerline runs along Y axis)
    SidewalkHoriz,     // Horizontal sidewalk (centerline runs along X axis)
    Intersection,       // Walkable intersection area
}

pub struct CityMap {
    pub grid: [[TileType; MAP_HEIGHT]; MAP_WIDTH],
}

impl CityMap {
    pub fn new() -> Self {
        let mut grid = [[TileType::Wall(0); MAP_HEIGHT]; MAP_WIDTH];

        // Let's layout streets and sidewalks dynamically using a repeating grid pattern
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {

                // Grid repeats every 7 tiles:
                // - x % 7 == 4 is a vertical street
                // - x % 7 == 3 or 5 are vertical sidewalks
                let is_vert_road = (x % 7) == 4;
                let is_vert_side = (x % 7) == 3 || (x % 7) == 5;
                let is_horiz_road = (y % 7) == 4;
                let is_horiz_side = (y % 7) == 3 || (y % 7) == 5;

                if (is_vert_road || is_vert_side) && (is_horiz_road || is_horiz_side) {
                    // Intersection of streets/sidewalks
                    grid[x][y] = TileType::Intersection;
                } else if is_vert_road {
                    grid[x][y] = TileType::Road;
                } else if is_horiz_road {
                    grid[x][y] = TileType::Road;
                } else if is_vert_side {
                    grid[x][y] = TileType::SidewalkVert;
                } else if is_horiz_side {
                    grid[x][y] = TileType::SidewalkHoriz;
                } else {
                    // Building block
                    // Procedurally choose wall style based on grid position
                    let wall_style = if (x + y) % 5 == 0 {
                        2 // Warning billboard
                    } else if (x == 1 && y == 1) || (x == MAP_WIDTH - 2 && y == MAP_HEIGHT - 2) {
                        3 // Police HQ
                    } else if (x * y) % 3 == 0 {
                        1 // Tech Panel
                    } else {
                        0 // Neon Grid
                    };
                    grid[x][y] = TileType::Wall(wall_style);
                }
            }
        }

        Self { grid }
    }

    /// Check if a position is solid (wall)
    pub fn is_solid(&self, x: f32, y: f32) -> bool {
        match self.get_tile(x, y) {
            TileType::Wall(_) => true,
            _ => false,
        }
    }

    /// Check the tile type at a position
    pub fn get_tile(&self, x: f32, y: f32) -> TileType {
        let wx = (x.floor() as i32).rem_euclid(MAP_WIDTH as i32) as usize;
        let wy = (y.floor() as i32).rem_euclid(MAP_HEIGHT as i32) as usize;
        self.grid[wx][wy]
    }


    /// Math to check if an entity at `(px, py)` moving in direction `(dx, dy)`
    /// is walking on the LEFT side of the sidewalk.
    /// Returns:
    /// - `Some(true)` if walking on the left side (Violation!)
    /// - `Some(false)` if walking on the right side (Compliant)
    /// - `None` if not on a sidewalk (e.g. intersection, road, or wall)
    #[allow(dead_code)]
    pub fn check_leftside_violation(&self, px: f32, py: f32, dx: f32, dy: f32) -> Option<bool> {
        let tile = self.get_tile(px, py);
        
        let cx = px.floor() + 0.5;
        let cy = py.floor() + 0.5;

        // Ensure direction vector is normalized
        let len = (dx*dx + dy*dy).sqrt();
        if len < 0.01 {
            return Some(false); // Standing still, no active violation
        }
        let ndx = dx / len;
        let ndy = dy / len;

        match tile {
            TileType::SidewalkVert => {
                // For a vertical sidewalk, the centerline is cx (X = constant)
                // We check the cross product: ndx * (py - cy) - ndy * (px - cx)
                // Since it's vertical, ndx is close to 0, ndy is close to 1 or -1.
                // Cross product simplifies to: -ndy * (px - cx)
                // If this is negative, they are on their left side!
                let cross_z = -ndy * (px - cx);
                Some(cross_z < -0.08) // A small buffer of 0.08 grid units to avoid edge fluttering
            }
            TileType::SidewalkHoriz => {
                // For a horizontal sidewalk, the centerline is cy (Y = constant)
                // Cross product simplifies to: ndx * (py - cy)
                // If this is negative, they are on their left side!
                let cross_z = ndx * (py - cy);
                Some(cross_z < -0.08)
            }
            _ => None, // Not on sidewalk (on road or intersection)
        }
    }
}
