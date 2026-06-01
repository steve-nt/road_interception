use std::time::Duration;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::constants::*;
use crate::input::Direction;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LightState {
    Red,
    Green,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntryLane {
    North,
    South,
    East,
    West,
}

impl EntryLane {
    pub fn index(self) -> usize {
        match self {
            EntryLane::North => 0,
            EntryLane::South => 1,
            EntryLane::East => 2,
            EntryLane::West => 3,
        }
    }

    fn all() -> [EntryLane; 4] {
        [
            EntryLane::North,
            EntryLane::South,
            EntryLane::East,
            EntryLane::West,
        ]
    }
}

impl From<Direction> for EntryLane {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => EntryLane::North,
            Direction::South => EntryLane::South,
            Direction::East => EntryLane::East,
            Direction::West => EntryLane::West,
        }
    }
}

/// Queue counts indexed by entry lane: North, South, East, West.
pub type LaneQueueCounts = [u32; 4];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    NorthSouthGreen,
    AllRed,
    EastWestGreen,
}

struct TrafficLight {
    state: LightState,
    position: (i32, i32),
}

const POLE_WIDTH: u32 = 6;
const POLE_HEIGHT: u32 = 28;
const HOUSING_WIDTH: u32 = 14;
const HOUSING_HEIGHT: u32 = 22;
const LENS_SIZE: u32 = 10;

const POLE_COLOR: Color = Color::RGB(90, 90, 95);
const HOUSING_COLOR: Color = Color::RGB(25, 25, 28);
const RED_COLOR: Color = Color::RGB(220, 40, 40);
const GREEN_COLOR: Color = Color::RGB(40, 200, 60);
const LENS_OFF_COLOR: Color = Color::RGB(50, 50, 55);

#[derive(Clone, Copy, PartialEq, Eq)]
enum GreenAxis {
    NorthSouth,
    EastWest,
}

pub struct TrafficLightController {
    lights: [TrafficLight; 4],
    phase: Phase,
    phase_timer: Duration,
    next_green_axis: GreenAxis,
}

pub fn lane_capacity() -> u32 {
    (LANE_LENGTH / (VEHICLE_LENGTH + SAFETY_GAP)) as u32
}

fn is_lane_congested(count: u32) -> bool {
    let pct_threshold = (lane_capacity() as f32 * CONGESTION_THRESHOLD).ceil() as u32;
    count >= MIN_CONGESTED_QUEUE || count >= pct_threshold
}

impl TrafficLightController {
    pub fn new() -> Self {
        let lights = [
            TrafficLight {
                state: LightState::Red,
                position: (LANE_SOUTH_X, STOP_LINE_NORTH - 18),
            },
            TrafficLight {
                state: LightState::Red,
                position: (LANE_NORTH_X, STOP_LINE_SOUTH + 18),
            },
            TrafficLight {
                state: LightState::Red,
                position: (STOP_LINE_EAST + 18, LANE_WEST_Y),
            },
            TrafficLight {
                state: LightState::Red,
                position: (STOP_LINE_WEST - 18, LANE_EAST_Y),
            },
        ];

        let mut controller = Self {
            lights,
            phase: Phase::AllRed,
            phase_timer: Duration::ZERO,
            next_green_axis: GreenAxis::NorthSouth,
        };
        controller.apply_phase_lights();
        controller
    }

    pub fn is_green(&self, lane: EntryLane) -> bool {
        self.lights[lane.index()].state == LightState::Green
    }

    pub fn update(&mut self, delta: Duration, queue_counts: LaneQueueCounts) {
        self.phase_timer += delta;

        match self.phase {
            Phase::NorthSouthGreen => {
                if self.should_end_green([EntryLane::North, EntryLane::South], queue_counts) {
                    self.begin_all_red();
                }
            }
            Phase::EastWestGreen => {
                if self.should_end_green([EntryLane::East, EntryLane::West], queue_counts) {
                    self.begin_all_red();
                }
            }
            Phase::AllRed => {
                if self.phase_timer >= Duration::from_millis(ALL_RED_MS) {
                    self.advance_from_all_red();
                }
            }
        }
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        for light in &self.lights {
            draw_traffic_light_unit(canvas, light.position, light.state);
        }
    }

    fn should_end_green(
        &self,
        lanes: [EntryLane; 2],
        queue_counts: LaneQueueCounts,
    ) -> bool {
        let base_green = Duration::from_millis(BASE_GREEN_MS);
        let max_green = Duration::from_millis(MAX_GREEN_MS);
        let congested = lanes
            .iter()
            .any(|&lane| is_lane_congested(queue_counts[lane.index()]));

        if self.phase_timer >= max_green {
            return true;
        }

        if self.phase_timer < base_green {
            return false;
        }

        !congested
    }

    fn begin_all_red(&mut self) {
        self.next_green_axis = match self.phase {
            Phase::NorthSouthGreen => GreenAxis::EastWest,
            Phase::EastWestGreen => GreenAxis::NorthSouth,
            Phase::AllRed => self.next_green_axis,
        };
        self.phase = Phase::AllRed;
        self.phase_timer = Duration::ZERO;
        self.apply_phase_lights();
    }

    fn advance_from_all_red(&mut self) {
        self.phase = match self.next_green_axis {
            GreenAxis::NorthSouth => Phase::NorthSouthGreen,
            GreenAxis::EastWest => Phase::EastWestGreen,
        };
        self.next_green_axis = match self.next_green_axis {
            GreenAxis::NorthSouth => GreenAxis::EastWest,
            GreenAxis::EastWest => GreenAxis::NorthSouth,
        };
        self.phase_timer = Duration::ZERO;
        self.apply_phase_lights();
    }

    fn apply_phase_lights(&mut self) {
        let green_lanes: &[EntryLane] = match self.phase {
            Phase::NorthSouthGreen => &[EntryLane::North, EntryLane::South],
            Phase::EastWestGreen => &[EntryLane::East, EntryLane::West],
            Phase::AllRed => &[],
        };

        for lane in EntryLane::all() {
            self.lights[lane.index()].state = if green_lanes.contains(&lane) {
                LightState::Green
            } else {
                LightState::Red
            };
        }
    }
}

impl Default for TrafficLightController {
    fn default() -> Self {
        Self::new()
    }
}

/// Pole + housing at the roadside, lens on top (red/green only).
fn draw_traffic_light_unit(canvas: &mut WindowCanvas, (cx, cy): (i32, i32), state: LightState) {
    let pole_x = cx - POLE_WIDTH as i32 / 2;
    let pole_y = cy;

    canvas.set_draw_color(POLE_COLOR);
    canvas
        .fill_rect(Rect::new(pole_x, pole_y, POLE_WIDTH, POLE_HEIGHT))
        .unwrap();

    let housing_x = cx - HOUSING_WIDTH as i32 / 2;
    let housing_y = pole_y - HOUSING_HEIGHT as i32;
    canvas.set_draw_color(HOUSING_COLOR);
    canvas
        .fill_rect(Rect::new(
            housing_x,
            housing_y,
            HOUSING_WIDTH,
            HOUSING_HEIGHT,
        ))
        .unwrap();

    let lens_x = cx - LENS_SIZE as i32 / 2;
    let lens_y = housing_y + 4;
    let active = match state {
        LightState::Red => RED_COLOR,
        LightState::Green => GREEN_COLOR,
    };
    let inactive = LENS_OFF_COLOR;

    // Red lens (top)
    canvas.set_draw_color(if state == LightState::Red {
        active
    } else {
        inactive
    });
    canvas
        .fill_rect(Rect::new(lens_x, lens_y, LENS_SIZE, LENS_SIZE))
        .unwrap();

    // Green lens (bottom)
    canvas.set_draw_color(if state == LightState::Green {
        active
    } else {
        inactive
    });
    canvas
        .fill_rect(Rect::new(
            lens_x,
            lens_y + LENS_SIZE as i32 + 2,
            LENS_SIZE,
            LENS_SIZE,
        ))
        .unwrap();
}
