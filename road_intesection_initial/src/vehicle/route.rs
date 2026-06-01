// Person 3 — Route enum, per-route colours, and turn-path waypoints.
//
// Route colour legend (for audit):
//   Left     → Yellow  RGB(255, 200,   0)
//   Straight → Blue    RGB(  0, 160, 255)
//   Right    → Red     RGB(255,  80,  80)

use rand::Rng;
use sdl2::pixels::Color;

use crate::constants::{ROAD_WIDTH, VEHICLE_LENGTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::map::intersection::{LaneId, Point};

/// Turn direction assigned at spawn — immutable for the vehicle's lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Left,
    Straight,
    Right,
}

impl Route {
    /// Pick a random route using the thread-local RNG.
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(0u8..3) {
            0 => Self::Left,
            1 => Self::Straight,
            _ => Self::Right,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Route::Left => Color::RGB(255, 200, 0),
            Route::Straight => Color::RGB(0, 160, 255),
            Route::Right => Color::RGB(255, 80, 80),
        }
    }
}

/// Build the ordered waypoint list for a vehicle spawned in `lane_id` with `route`.
///
/// Layout (all values derived from constants):
/// ```
///   cx = 400, cy = 400, half = 40 (ROAD_WIDTH/2), off = 20 (ROAD_WIDTH/4)
/// ```
///
/// Waypoint[0] is always the **stop-line checkpoint** — the vehicle parks here
/// when the light is red and advances when the light is green.
/// Subsequent waypoints guide it through the intersection and off-screen.
pub fn build_waypoints(lane_id: LaneId, route: Route) -> Vec<Point> {
    let cx = WINDOW_WIDTH as f32 / 2.0; // 400
    let cy = WINDOW_HEIGHT as f32 / 2.0; // 400
    let half = ROAD_WIDTH / 2.0; // 40
    let off = ROAD_WIDTH / 4.0; // 20
    let vhalf = VEHICLE_LENGTH / 2.0; // 10
    const EDGE: f32 = 50.0; // off-screen clearance

    // Stop-line checkpoint: vehicle centre sits just behind the stop line.
    let stop = match lane_id {
        LaneId::NorthIn => Point::new(cx + off, cy - half - vhalf), // (420, 350)
        LaneId::SouthIn => Point::new(cx - off, cy + half + vhalf), // (380, 450)
        LaneId::EastIn => Point::new(cx + half + vhalf, cy - off),  // (450, 380)
        LaneId::WestIn => Point::new(cx - half - vhalf, cy + off),  // (350, 420)
    };

    // Off-screen exit points.
    let exit_south = Point::new(cx + off, WINDOW_HEIGHT as f32 + EDGE); // (420, 850)
    let exit_north = Point::new(cx - off, -EDGE); // (380, -50)
    let exit_east = Point::new(WINDOW_WIDTH as f32 + EDGE, cy - off); // (850, 380)
    let exit_west = Point::new(-EDGE, cy + off); // (-50, 420)

    // Intersection pivot points at the edges of the conflict box.
    let pivot_ne = Point::new(cx + half, cy - off); // (440, 380) — east arm, upper lane
    let pivot_sw = Point::new(cx - half, cy + off); // (360, 420) — west arm, lower lane
    let pivot_se = Point::new(cx + off, cy + half); // (420, 440) — south arm, right lane
    let pivot_nw = Point::new(cx - off, cy - half); // (380, 360) — north arm, left lane

    // Turn direction logic (right-hand traffic):
    //   NorthIn (going south ↓):  Left = east,  Right = west
    //   SouthIn (going north ↑):  Left = west,  Right = east
    //   EastIn  (going west  ←):  Left = south, Right = north
    //   WestIn  (going east  →):  Left = north, Right = south
    match (lane_id, route) {
        (LaneId::NorthIn, Route::Straight) => vec![stop, exit_south],
        (LaneId::NorthIn, Route::Left) => vec![stop, pivot_ne, exit_east],
        (LaneId::NorthIn, Route::Right) => vec![stop, pivot_sw, exit_west],

        (LaneId::SouthIn, Route::Straight) => vec![stop, exit_north],
        (LaneId::SouthIn, Route::Left) => vec![stop, pivot_sw, exit_west],
        (LaneId::SouthIn, Route::Right) => vec![stop, pivot_ne, exit_east],

        (LaneId::EastIn, Route::Straight) => vec![stop, exit_west],
        (LaneId::EastIn, Route::Left) => vec![stop, pivot_se, exit_south],
        (LaneId::EastIn, Route::Right) => vec![stop, pivot_nw, exit_north],

        (LaneId::WestIn, Route::Straight) => vec![stop, exit_east],
        (LaneId::WestIn, Route::Left) => vec![stop, pivot_nw, exit_north],
        (LaneId::WestIn, Route::Right) => vec![stop, pivot_se, exit_south],
    }
}
