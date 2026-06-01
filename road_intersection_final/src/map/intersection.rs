use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

use crate::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaneId {
    NorthIn,
    SouthIn,
    EastIn,
    WestIn,
}

pub struct Lane {
    pub id: LaneId,
    pub queue_len: usize,
}

pub struct Intersection {
    pub lanes: Vec<Lane>,
}

impl Intersection {
    pub fn new() -> Self {
        Self {
            lanes: vec![
                Lane {
                    id: LaneId::NorthIn,
                    queue_len: 0,
                },
                Lane {
                    id: LaneId::SouthIn,
                    queue_len: 0,
                },
                Lane {
                    id: LaneId::EastIn,
                    queue_len: 0,
                },
                Lane {
                    id: LaneId::WestIn,
                    queue_len: 0,
                },
            ],
        }
    }

    #[allow(dead_code)]
    pub fn get_lane(&self, id: LaneId) -> Option<&Lane> {
        self.lanes.iter().find(|lane| lane.id == id)
    }

    pub fn draw(&self, canvas: &mut WindowCanvas) {
        draw_surface(canvas);
        draw_center_lines(canvas);
        draw_stop_lines(canvas);
    }
}

impl Default for Intersection {
    fn default() -> Self {
        Self::new()
    }
}

/* NOT USED
pub fn lane_capacity(lane_length: i32, vehicle_length: i32, safety_gap: i32) -> usize {
    (lane_length / (vehicle_length + safety_gap)) as usize
}
*/

const ROAD_COLOR: Color = Color::RGB(60, 60, 60);
const LANE_LINE_COLOR: Color = Color::RGB(255, 220, 0);
const STOP_LINE_COLOR: Color = Color::RGB(255, 255, 255);

fn draw_surface(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(ROAD_COLOR);

    canvas
        .fill_rect(Rect::new(
            INTERSECTION_X,
            0,
            ROAD_WIDTH as u32,
            WINDOW_HEIGHT,
        ))
        .unwrap();

    canvas
        .fill_rect(Rect::new(
            0,
            INTERSECTION_Y,
            WINDOW_WIDTH,
            ROAD_WIDTH as u32,
        ))
        .unwrap();
}

fn draw_center_lines(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(LANE_LINE_COLOR);

    let dash = 20i32;
    let gap = 15i32;
    let step = dash + gap;
    let cx = INTERSECTION_X + LANE_WIDTH;
    let cy = INTERSECTION_Y + LANE_WIDTH;

    let mut y = 0;
    while y < INTERSECTION_Y {
        canvas
            .draw_line((cx, y), (cx, (y + dash).min(INTERSECTION_Y)))
            .unwrap();
        y += step;
    }

    y = STOP_LINE_SOUTH;
    while y < WINDOW_HEIGHT as i32 {
        canvas
            .draw_line((cx, y), (cx, (y + dash).min(WINDOW_HEIGHT as i32)))
            .unwrap();
        y += step;
    }

    let mut x = 0;
    while x < INTERSECTION_X {
        canvas
            .draw_line((x, cy), ((x + dash).min(INTERSECTION_X), cy))
            .unwrap();
        x += step;
    }

    x = STOP_LINE_EAST;
    while x < WINDOW_WIDTH as i32 {
        canvas
            .draw_line((x, cy), ((x + dash).min(WINDOW_WIDTH as i32), cy))
            .unwrap();
        x += step;
    }
}

fn draw_stop_lines(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(STOP_LINE_COLOR);

    canvas
        .draw_line(
            (INTERSECTION_X, STOP_LINE_NORTH),
            (INTERSECTION_X + LANE_WIDTH, STOP_LINE_NORTH),
        )
        .unwrap();

    canvas
        .draw_line(
            (INTERSECTION_X + LANE_WIDTH, STOP_LINE_SOUTH),
            (INTERSECTION_X + ROAD_WIDTH, STOP_LINE_SOUTH),
        )
        .unwrap();

    canvas
        .draw_line(
            (STOP_LINE_EAST, INTERSECTION_Y),
            (STOP_LINE_EAST, INTERSECTION_Y + LANE_WIDTH),
        )
        .unwrap();

    canvas
        .draw_line(
            (STOP_LINE_WEST, INTERSECTION_Y + LANE_WIDTH),
            (STOP_LINE_WEST, INTERSECTION_Y + ROAD_WIDTH),
        )
        .unwrap();
}
