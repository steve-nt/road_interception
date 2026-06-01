pub mod route;

use std::f32::consts::PI;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::constants::*;
use crate::input::{Direction, InputHandler};
use crate::traffic_light::controller::{EntryLane, LaneQueueCounts, TrafficLightController};
use route::Route;

const SMALL_R: f32 = 30.0;
const LARGE_R: f32 = 90.0;
const LEFT_YIELD_MARGIN: f32 = (VEHICLE_LENGTH * 2) as f32;

#[derive(Clone, Copy)]
enum State {
    Approaching,
    Waiting,
    Turning {
        cx: f32,
        cy: f32,
        radius: f32,
        theta: f32,
        theta_end: f32,
        ccw: bool,
    },
    Exiting,
}

pub struct Vehicle {
    pub x: f32,
    pub y: f32,
    pub direction: Direction,
    pub route: Route,
    pub color: Color,
    vx: f32,
    vy: f32,
    state: State,
    spawn_cleared: bool,
}

impl Vehicle {
    fn new(dir: Direction, route: Route) -> Self {
        let (sx, sy) = spawn_pt(dir);
        let (vx, vy) = approach_vel(dir);
        Self {
            x: sx as f32,
            y: sy as f32,
            vx,
            vy,
            direction: dir,
            route,
            color: route.color(),
            state: State::Approaching,
            spawn_cleared: false,
        }
    }

    fn tick(&mut self, can_proceed: bool, max_step: f32) -> bool {
        let just_cleared = if !self.spawn_cleared && self.cleared_spawn() {
            self.spawn_cleared = true;
            true
        } else {
            false
        };

        match self.state {
            State::Approaching => {
                self.x += self.vx * max_step;
                self.y += self.vy * max_step;

                if self.past_stop_line() {
                    self.snap_to_stop_line();
                    if can_proceed && self.route != Route::Left {
                        self.enter_intersection();
                    } else {
                        self.state = State::Waiting;
                    }
                }
            }

            State::Waiting => {
                if can_proceed {
                    self.enter_intersection();
                }
            }

            State::Turning {
                cx,
                cy,
                radius,
                theta,
                theta_end,
                ccw,
            } => {
                let d = VEHICLE_SPEED / radius;
                let new_theta = if ccw { theta + d } else { theta - d };
                let done = if ccw {
                    new_theta >= theta_end
                } else {
                    new_theta <= theta_end
                };

                if done {
                    self.x = cx + radius * theta_end.cos();
                    self.y = cy + radius * theta_end.sin();
                    let (evx, evy) = exit_vel(self.direction, self.route);
                    self.vx = evx;
                    self.vy = evy;
                    self.state = State::Exiting;
                } else {
                    self.x = cx + radius * new_theta.cos();
                    self.y = cy + radius * new_theta.sin();
                    if ccw {
                        self.vx = -new_theta.sin();
                        self.vy = new_theta.cos();
                    } else {
                        self.vx = new_theta.sin();
                        self.vy = -new_theta.cos();
                    }
                    self.state = State::Turning {
                        cx,
                        cy,
                        radius,
                        theta: new_theta,
                        theta_end,
                        ccw,
                    };
                }
            }

            State::Exiting => {
                self.x += self.vx * VEHICLE_SPEED;
                self.y += self.vy * VEHICLE_SPEED;
            }
        }

        just_cleared
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        canvas.set_draw_color(self.color);
        let half = VEHICLE_LENGTH / 2;
        canvas
            .fill_rect(Rect::new(
                self.x as i32 - half,
                self.y as i32 - half,
                VEHICLE_LENGTH as u32,
                VEHICLE_LENGTH as u32,
            ))
            .unwrap();
    }

    fn is_off_screen(&self) -> bool {
        let m = (VEHICLE_LENGTH + 4) as f32;
        self.x < -m
            || self.x > WINDOW_WIDTH as f32 + m
            || self.y < -m
            || self.y > WINDOW_HEIGHT as f32 + m
    }

    fn cleared_spawn(&self) -> bool {
        let gap = (VEHICLE_LENGTH + SAFETY_GAP) as f32;
        match self.direction {
            Direction::North => self.y >= gap,
            Direction::South => self.y <= WINDOW_HEIGHT as f32 - gap,
            Direction::East => self.x <= WINDOW_WIDTH as f32 - gap,
            Direction::West => self.x >= gap,
        }
    }

    fn past_stop_line(&self) -> bool {
        match self.direction {
            Direction::North => self.y >= STOP_LINE_NORTH as f32,
            Direction::South => self.y <= STOP_LINE_SOUTH as f32,
            Direction::East => self.x <= STOP_LINE_EAST as f32,
            Direction::West => self.x >= STOP_LINE_WEST as f32,
        }
    }

    fn snap_to_stop_line(&mut self) {
        match self.direction {
            Direction::North => self.y = STOP_LINE_NORTH as f32,
            Direction::South => self.y = STOP_LINE_SOUTH as f32,
            Direction::East => self.x = STOP_LINE_EAST as f32,
            Direction::West => self.x = STOP_LINE_WEST as f32,
        }
    }

    fn enter_intersection(&mut self) {
        match self.route {
            Route::Straight => self.state = State::Exiting,
            Route::Right => {
                let (cx, cy, ts, te) = right_arc(self.direction);
                self.x = cx + SMALL_R * ts.cos();
                self.y = cy + SMALL_R * ts.sin();
                self.state = State::Turning {
                    cx,
                    cy,
                    radius: SMALL_R,
                    theta: ts,
                    theta_end: te,
                    ccw: true,
                };
            }
            Route::Left => {
                let (cx, cy, ts, te) = left_arc(self.direction);
                self.x = cx + LARGE_R * ts.cos();
                self.y = cy + LARGE_R * ts.sin();
                self.state = State::Turning {
                    cx,
                    cy,
                    radius: LARGE_R,
                    theta: ts,
                    theta_end: te,
                    ccw: false,
                };
            }
        }
    }

    fn lane_pos(&self) -> f32 {
        match self.direction {
            Direction::North | Direction::South => self.y,
            Direction::East | Direction::West => self.x,
        }
    }
}

fn in_box(v: &Vehicle) -> bool {
    let ix = INTERSECTION_X as f32;
    let iy = INTERSECTION_Y as f32;
    let ex = (INTERSECTION_X + ROAD_WIDTH) as f32;
    let ey = (INTERSECTION_Y + ROAD_WIDTH) as f32;
    v.x >= ix && v.x <= ex && v.y >= iy && v.y <= ey
}

fn perpendicular_occupying(dir: Direction, vehicles: &[Vehicle]) -> bool {
    let ns = matches!(dir, Direction::North | Direction::South);
    vehicles.iter().any(|v| {
        let v_ns = matches!(v.direction, Direction::North | Direction::South);
        if v_ns == ns {
            return false;
        }
        match v.state {
            State::Turning { .. } => true,
            State::Exiting => in_box(v),
            _ => false,
        }
    })
}

fn right_turn_exit_blocked(dir: Direction, vehicles: &[Vehicle]) -> bool {
    let opp = opposing_dir(dir);
    vehicles.iter().any(|v| {
        v.direction == opp
            && v.route == Route::Left
            && matches!(v.state, State::Turning { .. })
    })
}

fn opposing_blocks_left(dir: Direction, vehicles: &[Vehicle]) -> bool {
    let opp = opposing_dir(dir);
    let m = LEFT_YIELD_MARGIN;
    let ix = INTERSECTION_X as f32 - m;
    let iy = INTERSECTION_Y as f32 - m;
    let ex = (INTERSECTION_X + ROAD_WIDTH) as f32 + m;
    let ey = (INTERSECTION_Y + ROAD_WIDTH) as f32 + m;

    vehicles.iter().any(|v| {
        if v.direction != opp {
            return false;
        }
        match v.state {
            State::Turning { .. } => true,
            State::Exiting => v.x > ix && v.x < ex && v.y > iy && v.y < ey,
            State::Waiting => v.route == Route::Left && lower_left_priority(dir),
            _ => false,
        }
    })
}

fn lower_left_priority(dir: Direction) -> bool {
    matches!(dir, Direction::North | Direction::East)
}

fn opposing_dir(dir: Direction) -> Direction {
    match dir {
        Direction::North => Direction::South,
        Direction::South => Direction::North,
        Direction::East => Direction::West,
        Direction::West => Direction::East,
    }
}

fn spawn_pt(dir: Direction) -> (i32, i32) {
    match dir {
        Direction::North => SPAWN_NORTH,
        Direction::South => SPAWN_SOUTH,
        Direction::East => SPAWN_EAST,
        Direction::West => SPAWN_WEST,
    }
}

fn approach_vel(dir: Direction) -> (f32, f32) {
    match dir {
        Direction::North => (0.0, 1.0),
        Direction::South => (0.0, -1.0),
        Direction::East => (-1.0, 0.0),
        Direction::West => (1.0, 0.0),
    }
}

fn right_arc(dir: Direction) -> (f32, f32, f32, f32) {
    let ix = INTERSECTION_X as f32;
    let iy = INTERSECTION_Y as f32;
    let ex = (INTERSECTION_X + ROAD_WIDTH) as f32;
    let ey = (INTERSECTION_Y + ROAD_WIDTH) as f32;
    match dir {
        Direction::North => (ix, iy, 0.0, PI / 2.0),
        Direction::South => (ex, ey, PI, 3.0 * PI / 2.0),
        Direction::East => (ex, iy, PI / 2.0, PI),
        Direction::West => (ix, ey, 3.0 * PI / 2.0, 2.0 * PI),
    }
}

fn left_arc(dir: Direction) -> (f32, f32, f32, f32) {
    let ix = INTERSECTION_X as f32;
    let iy = INTERSECTION_Y as f32;
    let ex = (INTERSECTION_X + ROAD_WIDTH) as f32;
    let ey = (INTERSECTION_Y + ROAD_WIDTH) as f32;
    match dir {
        Direction::North => (ex, iy, PI, PI / 2.0),
        Direction::South => (ix, ey, 0.0, -PI / 2.0),
        Direction::East => (ex, ey, 3.0 * PI / 2.0, PI),
        Direction::West => (ix, iy, PI / 2.0, 0.0),
    }
}

fn exit_vel(dir: Direction, route: Route) -> (f32, f32) {
    match (dir, route) {
        (Direction::North, Route::Right) => (-1.0, 0.0),
        (Direction::North, Route::Left) => (1.0, 0.0),
        (Direction::South, Route::Right) => (1.0, 0.0),
        (Direction::South, Route::Left) => (-1.0, 0.0),
        (Direction::East, Route::Right) => (0.0, -1.0),
        (Direction::East, Route::Left) => (0.0, 1.0),
        (Direction::West, Route::Right) => (0.0, 1.0),
        (Direction::West, Route::Left) => (0.0, -1.0),
        (_, Route::Straight) => unreachable!(),
    }
}

pub struct VehicleManager {
    vehicles: Vec<Vehicle>,
}

impl VehicleManager {
    pub fn new() -> Self {
        Self {
            vehicles: Vec::new(),
        }
    }

    pub fn spawn(&mut self, dir: Direction) {
        self.vehicles.push(Vehicle::new(dir, Route::random()));
    }

    pub fn update(&mut self, traffic: &TrafficLightController, input: &mut InputHandler) {
        let n = self.vehicles.len();

        let params: Vec<(bool, f32)> = (0..n)
            .map(|i| {
                let v = &self.vehicles[i];
                let is_green = traffic.is_green(EntryLane::from(v.direction));

                let perp_blocked = perpendicular_occupying(v.direction, &self.vehicles);
                let left_blocked =
                    v.route == Route::Left && opposing_blocks_left(v.direction, &self.vehicles);
                let right_blocked = v.route == Route::Right
                    && right_turn_exit_blocked(v.direction, &self.vehicles);

                let can_proceed = is_green && !perp_blocked && !left_blocked && !right_blocked;
                let max_step = self.max_step(i);
                (can_proceed, max_step)
            })
            .collect();

        for (i, v) in self.vehicles.iter_mut().enumerate() {
            let (can_proceed, max_step) = params[i];
            if v.tick(can_proceed, max_step) {
                input.clear_spawn(v.direction);
            }
        }

        self.vehicles.retain(|v| !v.is_off_screen());
    }

    fn max_step(&self, idx: usize) -> f32 {
        let v = &self.vehicles[idx];

        if !matches!(v.state, State::Approaching) {
            return VEHICLE_SPEED;
        }

        let mut min_gap = f32::MAX;

        for (j, other) in self.vehicles.iter().enumerate() {
            if j == idx {
                continue;
            }
            if other.direction != v.direction {
                continue;
            }
            if !matches!(other.state, State::Approaching | State::Waiting) {
                continue;
            }

            let is_ahead = match v.direction {
                Direction::North => other.y > v.y,
                Direction::South => other.y < v.y,
                Direction::East => other.x < v.x,
                Direction::West => other.x > v.x,
            };

            if is_ahead {
                let gap = (other.lane_pos() - v.lane_pos()).abs() - VEHICLE_LENGTH as f32;
                if gap < min_gap {
                    min_gap = gap;
                }
            }
        }

        if min_gap == f32::MAX {
            VEHICLE_SPEED
        } else {
            (min_gap - SAFETY_GAP as f32).clamp(0.0, VEHICLE_SPEED)
        }
    }

    pub fn queue_counts(&self) -> LaneQueueCounts {
        let mut counts = [0u32; 4];
        for v in &self.vehicles {
            if matches!(v.state, State::Approaching | State::Waiting) {
                counts[v.direction.index()] += 1;
            }
        }
        counts
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        for v in &self.vehicles {
            v.draw(canvas);
        }
    }

    pub fn sync_lane_queues(&self, lanes: &mut [crate::map::intersection::Lane]) {
        let counts = self.queue_counts();
        for lane in lanes {
            lane.queue_len = counts[Direction::from_lane(lane.id).index()] as usize;
        }
    }
}

impl Default for VehicleManager {
    fn default() -> Self {
        Self::new()
    }
}
