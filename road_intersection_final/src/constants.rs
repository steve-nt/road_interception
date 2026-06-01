//! Layout constants — derived from intersection geometry.
//! Changing `LANE_WIDTH` / window size updates stop lines, spawns, and capacity.

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 800;

pub const LANE_WIDTH: i32 = 60;
pub const ROAD_WIDTH: i32 = LANE_WIDTH * 2;

pub const INTERSECTION_X: i32 = (WINDOW_WIDTH as i32 - ROAD_WIDTH) / 2;
pub const INTERSECTION_Y: i32 = (WINDOW_HEIGHT as i32 - ROAD_WIDTH) / 2;

pub const LANE_SOUTH_X: i32 = INTERSECTION_X + LANE_WIDTH / 2;
pub const LANE_NORTH_X: i32 = INTERSECTION_X + LANE_WIDTH + LANE_WIDTH / 2;

pub const LANE_WEST_Y: i32 = INTERSECTION_Y + LANE_WIDTH / 2;
pub const LANE_EAST_Y: i32 = INTERSECTION_Y + LANE_WIDTH + LANE_WIDTH / 2;

pub const STOP_LINE_NORTH: i32 = INTERSECTION_Y;
pub const STOP_LINE_SOUTH: i32 = INTERSECTION_Y + ROAD_WIDTH;
pub const STOP_LINE_EAST: i32 = INTERSECTION_X + ROAD_WIDTH;
pub const STOP_LINE_WEST: i32 = INTERSECTION_X;

pub const SPAWN_NORTH: (i32, i32) = (LANE_SOUTH_X, 0);
pub const SPAWN_SOUTH: (i32, i32) = (LANE_NORTH_X, WINDOW_HEIGHT as i32);
pub const SPAWN_EAST: (i32, i32) = (WINDOW_WIDTH as i32, LANE_WEST_Y);
pub const SPAWN_WEST: (i32, i32) = (0, LANE_EAST_Y);

pub const VEHICLE_LENGTH: i32 = 20;
// pub const _VEHICLE_WIDTH: i32 = 12;  // Unused; vehicles drawn as squares (VEHICLE_LENGTH × VEHICLE_LENGTH)
pub const SAFETY_GAP: i32 = 8;

/// Approach lane length (spawn → stop line). capacity = floor(340 / 28) = 12
pub const LANE_LENGTH: i32 = INTERSECTION_X;

pub const VEHICLE_SPEED: f32 = 2.0;

pub const BASE_GREEN_MS: u64 = 4_000;
pub const ALL_RED_MS: u64 = 800;
pub const MAX_GREEN_MS: u64 = 12_000;
pub const CONGESTION_THRESHOLD: f32 = 0.80;
pub const MIN_CONGESTED_QUEUE: u32 = 5;

// Distance from the road edge to a traffic light (sits on the shoulder).
// pub const _LIGHT_SHOULDER_OFFSET: i32 = 16;  // Unused; light positioning calculated inline