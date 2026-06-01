use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::constants::{LANE_LENGTH, ROAD_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LaneId {
    NorthIn,
    SouthIn,
    EastIn,
    WestIn,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub start: Point,
    pub end: Point,
}

impl Segment {
    pub const fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}

pub struct Lane {
    pub id: LaneId,
    pub spawn_point: Point,
    pub stop_line: Segment,
    pub direction: Point,
    pub length: f32,
    pub queue_len: usize,
    light_position: Point,
}

impl Lane {
    /// capacity = floor(lane_length / (vehicle_length + safety_gap))
    pub fn capacity(&self, vehicle_length: f32, safety_gap: f32) -> usize {
        lane_capacity(self.length, vehicle_length, safety_gap)
    }

    pub fn light_position(&self) -> Point {
        self.light_position
    }
}

pub fn lane_capacity(lane_length: f32, vehicle_length: f32, safety_gap: f32) -> usize {
    (lane_length / (vehicle_length + safety_gap)).floor() as usize
}

pub struct Intersection {
    pub lanes: Vec<Lane>,
    center: Point,
    half_road: f32,
}

impl Intersection {
    pub fn new() -> Self {
        let center = Point::new(WINDOW_WIDTH as f32 / 2.0, WINDOW_HEIGHT as f32 / 2.0);
        let half_road = ROAD_WIDTH / 2.0;
        let lane_offset = ROAD_WIDTH / 4.0;
        let stop_half = ROAD_WIDTH / 4.0;

        let lanes = vec![
            build_lane(
                LaneId::NorthIn,
                Point::new(center.x + lane_offset, center.y - half_road - LANE_LENGTH),
                Segment::new(
                    Point::new(center.x + lane_offset - stop_half, center.y - half_road),
                    Point::new(center.x + lane_offset + stop_half, center.y - half_road),
                ),
                Point::new(0.0, 1.0),
                Point::new(center.x + lane_offset + 18.0, center.y - half_road - 14.0),
            ),
            build_lane(
                LaneId::SouthIn,
                Point::new(center.x - lane_offset, center.y + half_road + LANE_LENGTH),
                Segment::new(
                    Point::new(center.x - lane_offset - stop_half, center.y + half_road),
                    Point::new(center.x - lane_offset + stop_half, center.y + half_road),
                ),
                Point::new(0.0, -1.0),
                Point::new(center.x - lane_offset - 18.0, center.y + half_road + 14.0),
            ),
            build_lane(
                LaneId::EastIn,
                Point::new(center.x + half_road + LANE_LENGTH, center.y - lane_offset),
                Segment::new(
                    Point::new(center.x + half_road, center.y - lane_offset - stop_half),
                    Point::new(center.x + half_road, center.y - lane_offset + stop_half),
                ),
                Point::new(-1.0, 0.0),
                Point::new(center.x + half_road + 14.0, center.y - lane_offset - 18.0),
            ),
            build_lane(
                LaneId::WestIn,
                Point::new(center.x - half_road - LANE_LENGTH, center.y + lane_offset),
                Segment::new(
                    Point::new(center.x - half_road, center.y + lane_offset - stop_half),
                    Point::new(center.x - half_road, center.y + lane_offset + stop_half),
                ),
                Point::new(1.0, 0.0),
                Point::new(center.x - half_road - 14.0, center.y + lane_offset + 18.0),
            ),
        ];

        Self {
            lanes,
            center,
            half_road,
        }
    }

    pub fn get_lane(&self, id: LaneId) -> Option<&Lane> {
        self.lanes.iter().find(|lane| lane.id == id)
    }

    pub fn get_lane_mut(&mut self, id: LaneId) -> Option<&mut Lane> {
        self.lanes.iter_mut().find(|lane| lane.id == id)
    }

    /// Axis-aligned box covering the intersection conflict zone (no-go area).
    pub fn conflict_zone(&self) -> Rect {
        let half = self.half_road as i32;
        let cx = self.center.x as i32;
        let cy = self.center.y as i32;
        Rect::new(cx - half, cy - half, ROAD_WIDTH as u32, ROAD_WIDTH as u32)
    }

    pub fn light_position(&self, id: LaneId) -> (i32, i32) {
        self.get_lane(id)
            .map(|lane| {
                let p = lane.light_position();
                (p.x.round() as i32, p.y.round() as i32)
            })
            .unwrap_or((0, 0))
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        self.draw_roads(canvas)?;
        self.draw_conflict_zone(canvas)?;
        self.draw_lane_markings(canvas)?;
        self.draw_stop_lines(canvas)?;
        self.draw_light_placeholders(canvas)?;
        Ok(())
    }

    fn draw_roads(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(55, 55, 55));

        let cx = self.center.x as i32;
        let cy = self.center.y as i32;
        let half = self.half_road as i32;

        canvas.fill_rect(Rect::new(0, cy - half, WINDOW_WIDTH, ROAD_WIDTH as u32))?;
        canvas.fill_rect(Rect::new(cx - half, 0, ROAD_WIDTH as u32, WINDOW_HEIGHT))?;

        Ok(())
    }

    fn draw_conflict_zone(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(70, 70, 70));
        canvas.fill_rect(self.conflict_zone())?;

        canvas.set_draw_color(Color::RGB(180, 60, 60));
        canvas.draw_rect(self.conflict_zone())?;

        Ok(())
    }

    fn draw_lane_markings(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        let cx = self.center.x as i32;
        let cy = self.center.y as i32;
        let half = self.half_road as i32;
        let lane_offset = (ROAD_WIDTH / 4.0) as i32;

        canvas.set_draw_color(Color::RGB(220, 220, 80));

        // Separate opposing flows on each road arm.
        draw_dashed_line(
            canvas,
            Point::new((cx + lane_offset) as f32, 0.0),
            Point::new((cx + lane_offset) as f32, (cy - half) as f32),
            14,
            10,
        )?;
        draw_dashed_line(
            canvas,
            Point::new((cx - lane_offset) as f32, (cy + half) as f32),
            Point::new((cx - lane_offset) as f32, WINDOW_HEIGHT as f32),
            14,
            10,
        )?;
        draw_dashed_line(
            canvas,
            Point::new(0.0, (cy - lane_offset) as f32),
            Point::new((cx - half) as f32, (cy - lane_offset) as f32),
            14,
            10,
        )?;
        draw_dashed_line(
            canvas,
            Point::new((cx + half) as f32, (cy + lane_offset) as f32),
            Point::new(WINDOW_WIDTH as f32, (cy + lane_offset) as f32),
            14,
            10,
        )?;

        Ok(())
    }

    fn draw_stop_lines(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(240, 240, 240));
        for lane in &self.lanes {
            draw_thick_segment(canvas, lane.stop_line, 4)?;
        }
        Ok(())
    }

    fn draw_light_placeholders(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        for lane in &self.lanes {
            let p = lane.light_position();
            let x = p.x.round() as i32;
            let y = p.y.round() as i32;

            canvas.set_draw_color(Color::RGB(30, 30, 30));
            canvas.fill_rect(Rect::new(x - 8, y - 8, 16, 16))?;
            canvas.set_draw_color(Color::RGB(80, 80, 80));
            canvas.draw_rect(Rect::new(x - 8, y - 8, 16, 16))?;
        }
        Ok(())
    }
}

impl Default for Intersection {
    fn default() -> Self {
        Self::new()
    }
}

fn build_lane(
    id: LaneId,
    spawn_point: Point,
    stop_line: Segment,
    direction: Point,
    light_position: Point,
) -> Lane {
    Lane {
        id,
        spawn_point,
        stop_line,
        direction,
        length: LANE_LENGTH,
        queue_len: 0,
        light_position,
    }
}

fn draw_dashed_line(
    canvas: &mut Canvas<Window>,
    start: Point,
    end: Point,
    dash_len: i32,
    gap_len: i32,
) -> Result<(), String> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 0.0 {
        return Ok(());
    }

    let ux = dx / length;
    let uy = dy / length;
    let step = (dash_len + gap_len) as f32;
    let mut travelled = 0.0;

    while travelled < length {
        let seg_start = travelled;
        let seg_end = (travelled + dash_len as f32).min(length);
        let p1 = Point::new(start.x + ux * seg_start, start.y + uy * seg_start);
        let p2 = Point::new(start.x + ux * seg_end, start.y + uy * seg_end);
        canvas.draw_line(
            (p1.x.round() as i32, p1.y.round() as i32),
            (p2.x.round() as i32, p2.y.round() as i32),
        )?;
        travelled += step;
    }

    Ok(())
}

fn draw_thick_segment(
    canvas: &mut Canvas<Window>,
    segment: Segment,
    thickness: i32,
) -> Result<(), String> {
    let dx = segment.end.x - segment.start.x;
    let dy = segment.end.y - segment.start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 0.0 {
        return Ok(());
    }

    let nx = -dy / length;
    let ny = dx / length;
    let half = thickness as f32 / 2.0;

    for offset in [-half, half] {
        let p1 = Point::new(
            segment.start.x + nx * offset,
            segment.start.y + ny * offset,
        );
        let p2 = Point::new(segment.end.x + nx * offset, segment.end.y + ny * offset);
        canvas.draw_line(
            (p1.x.round() as i32, p1.y.round() as i32),
            (p2.x.round() as i32, p2.y.round() as i32),
        )?;
    }

    Ok(())
}
