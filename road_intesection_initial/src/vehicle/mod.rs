// Person 3 — Vehicle struct, movement, following distance, and rendering.

pub mod route;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::constants::{
    MIN_SPAWN_DISTANCE, SAFETY_GAP, VEHICLE_LENGTH, VEHICLE_SPEED, WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::map::intersection::{Intersection, LaneId, Point};
use crate::traffic_light::controller::TrafficLightController;
use route::{build_waypoints, Route};

// ── Vehicle ───────────────────────────────────────────────────────────────────

pub struct Vehicle {
    pub id: u32,
    pub lane_id: LaneId,
    pub route: Route,
    pub position: Point,
    waypoints: Vec<Point>,
    /// Index of the next waypoint to move toward.
    /// Index 0 is the stop-line checkpoint.
    waypoint_idx: usize,
    pub active: bool,
}

impl Vehicle {
    /// True once the vehicle has advanced past the stop-line checkpoint.
    pub fn past_stop_line(&self) -> bool {
        self.waypoint_idx > 0
    }

    /// Scalar progress along the approach lane — higher value = closer to stop line.
    /// Used for following-distance comparisons between vehicles in the same lane.
    pub fn lane_progress(&self) -> f32 {
        match self.lane_id {
            LaneId::NorthIn => self.position.y,
            LaneId::SouthIn => -self.position.y,
            LaneId::EastIn => -self.position.x,
            LaneId::WestIn => self.position.x,
        }
    }

    /// Move toward the current waypoint at VEHICLE_SPEED px/frame.
    ///
    /// At waypoint 0 (stop line) the vehicle waits until the light is green
    /// before advancing.  All other waypoints are traversed unconditionally.
    pub fn advance(&mut self, lights: &TrafficLightController) {
        if !self.active {
            return;
        }
        if self.waypoint_idx >= self.waypoints.len() {
            self.active = false;
            return;
        }

        let target = self.waypoints[self.waypoint_idx];
        let dx = target.x - self.position.x;
        let dy = target.y - self.position.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= VEHICLE_SPEED {
            // Snap to waypoint.
            self.position = target;
            if self.waypoint_idx == 0 {
                // Stop-line checkpoint: only advance on green.
                if lights.can_proceed(self.lane_id) {
                    self.waypoint_idx += 1;
                }
                // else: park here, re-checked next frame
            } else {
                self.waypoint_idx += 1;
            }
        } else {
            self.position.x += dx / dist * VEHICLE_SPEED;
            self.position.y += dy / dist * VEHICLE_SPEED;
        }

        // Remove once all waypoints are consumed or vehicle is off-screen.
        if self.waypoint_idx >= self.waypoints.len() || off_screen(self.position) {
            self.active = false;
        }
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let s = VEHICLE_LENGTH as u32;
        let x = self.position.x.round() as i32 - s as i32 / 2;
        let y = self.position.y.round() as i32 - s as i32 / 2;
        canvas.set_draw_color(self.route.color());
        canvas.fill_rect(Rect::new(x, y, s, s))?;
        canvas.set_draw_color(Color::RGB(15, 15, 15));
        canvas.draw_rect(Rect::new(x, y, s, s))?;
        Ok(())
    }
}

// ── VehicleManager ────────────────────────────────────────────────────────────

pub struct VehicleManager {
    vehicles: Vec<Vehicle>,
    next_id: u32,
}

impl VehicleManager {
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
            next_id: 0,
        }
    }

    // ── Spawn ─────────────────────────────────────────────────────────────────

    /// Attempt to spawn a vehicle in `lane_id`.  Silently rejected when the
    /// most-recently spawned vehicle in that lane is still within
    /// MIN_SPAWN_DISTANCE of the spawn point (anti-spam guard).
    pub fn try_spawn(&mut self, lane_id: LaneId, spawn_point: Point) {
        if !self.spawn_is_safe(lane_id, spawn_point) {
            return;
        }
        let route = Route::random();
        let waypoints = build_waypoints(lane_id, route);
        self.vehicles.push(Vehicle {
            id: self.next_id,
            lane_id,
            route,
            position: spawn_point,
            waypoints,
            waypoint_idx: 0,
            active: true,
        });
        self.next_id += 1;
    }

    /// Returns true when no existing vehicle in the lane is within
    /// MIN_SPAWN_DISTANCE of `spawn_point`.
    fn spawn_is_safe(&self, lane_id: LaneId, spawn_point: Point) -> bool {
        !self.vehicles.iter().filter(|v| v.active && v.lane_id == lane_id).any(|v| {
            let dx = v.position.x - spawn_point.x;
            let dy = v.position.y - spawn_point.y;
            (dx * dx + dy * dy).sqrt() < MIN_SPAWN_DISTANCE
        })
    }

    // ── Update ────────────────────────────────────────────────────────────────

    /// Advance all vehicles one frame:
    /// 1. Compute following-distance blocking for queued vehicles.
    /// 2. Move unblocked vehicles.
    /// 3. Purge inactive vehicles.
    /// 4. Update `queue_len` on each lane in the intersection.
    pub fn update(&mut self, intersection: &mut Intersection, lights: &TrafficLightController) {
        let n = self.vehicles.len();

        // Step 1 — following-distance blocking (queue vehicles only).
        let mut blocked = vec![false; n];
        for i in 0..n {
            let vi = &self.vehicles[i];
            if !vi.active || vi.past_stop_line() {
                continue;
            }
            let my_progress = vi.lane_progress();
            let my_lane = vi.lane_id;

            for j in 0..n {
                if i == j {
                    continue;
                }
                let vj = &self.vehicles[j];
                if !vj.active || vj.lane_id != my_lane || vj.past_stop_line() {
                    continue;
                }
                let lead_progress = vj.lane_progress();
                // vj is ahead of vi and too close.
                if lead_progress > my_progress
                    && lead_progress - my_progress < VEHICLE_LENGTH + SAFETY_GAP
                {
                    blocked[i] = true;
                    break;
                }
            }
        }

        // Step 2 — advance unblocked vehicles.
        for i in 0..n {
            if self.vehicles[i].active && !blocked[i] {
                self.vehicles[i].advance(lights);
            }
        }

        // Step 3 — remove inactive vehicles.
        self.vehicles.retain(|v| v.active);

        // Step 4 — sync queue lengths back into the intersection.
        for &id in &[LaneId::NorthIn, LaneId::SouthIn, LaneId::EastIn, LaneId::WestIn] {
            let q = self.vehicles.iter().filter(|v| v.lane_id == id && !v.past_stop_line()).count();
            if let Some(lane) = intersection.get_lane_mut(id) {
                lane.queue_len = q;
            }
        }
    }

    // ── Draw ──────────────────────────────────────────────────────────────────

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        for v in &self.vehicles {
            if v.active {
                v.draw(canvas)?;
            }
        }
        Ok(())
    }
}

impl Default for VehicleManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn off_screen(p: Point) -> bool {
    const MARGIN: f32 = 60.0;
    p.x < -MARGIN
        || p.x > WINDOW_WIDTH as f32 + MARGIN
        || p.y < -MARGIN
        || p.y > WINDOW_HEIGHT as f32 + MARGIN
}
