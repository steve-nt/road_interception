//! Shared constants — each section is owned by the person named in the comment.

// ── Person 1: map geometry ────────────────────────────────────────────────────

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 800;

/// Width of each road arm (pixels).
pub const ROAD_WIDTH: f32 = 80.0;

/// Distance from stop line to spawn point (pixels).
pub const LANE_LENGTH: f32 = 400.0;

// ── Person 2: traffic-light timing ───────────────────────────────────────────

/// Minimum green-phase duration (accumulates at 16 ms/frame).
pub const BASE_GREEN_MS: u32 = 3_000;

/// Extra green time awarded once when the active approach is congested.
pub const CONGESTION_EXTENSION_MS: u32 = 2_000;

/// All-red inter-phase buffer.
pub const ALL_RED_MS: u32 = 500;

// ── Person 3: vehicle dimensions and spacing ─────────────────────────────────

/// Vehicle length in pixels; used in capacity formula and following distance.
pub const VEHICLE_LENGTH: f32 = 20.0;

/// Minimum bumper-to-bumper gap between vehicles.
pub const SAFETY_GAP: f32 = 8.0;

/// Fixed movement speed in pixels per frame (~60 fps).
pub const VEHICLE_SPEED: f32 = 2.0;

/// Minimum distance a recently spawned vehicle must clear before another
/// can spawn on the same lane (~VEHICLE_LENGTH + SAFETY_GAP).
pub const MIN_SPAWN_DISTANCE: f32 = VEHICLE_LENGTH + SAFETY_GAP; // 28.0
