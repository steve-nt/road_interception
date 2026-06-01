mod constants;
mod input;
mod map;
mod traffic_light;
mod ui;
mod vehicle;

use constants::{WINDOW_HEIGHT, WINDOW_WIDTH};
use input::{GameEvent, InputHandler};
use std::time::{Duration, Instant};
use traffic_light::controller::TrafficLightController;
use vehicle::VehicleManager;

const TARGET_FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TARGET_FPS);

fn main() {
    let sdl_context = sdl2::init().expect("SDL2 init failed");
    let video = sdl_context.video().expect("SDL2 video failed");

    let window = video
        .window("Road Intersection Simulation", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .expect("window creation failed");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("canvas creation failed");

    let mut event_pump = sdl_context.event_pump().expect("event pump failed");
    let mut input_handler = InputHandler::new();
    let mut traffic_lights = TrafficLightController::new();
    let mut vehicle_manager = VehicleManager::new();
    let mut intersection = map::Intersection::new();

    'running: loop {
        let frame_start = Instant::now();

        for event in input_handler.poll(&mut event_pump) {
            match event {
                GameEvent::Quit => break 'running,
                GameEvent::SpawnVehicle(dir) => vehicle_manager.spawn(dir),
            }
        }

        vehicle_manager.update(&traffic_lights, &mut input_handler);
        traffic_lights.update(FRAME_DURATION, vehicle_manager.queue_counts());
        vehicle_manager.sync_lane_queues(&mut intersection.lanes);

        draw_frame(&mut canvas, &intersection, &traffic_lights, &vehicle_manager);

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    std::process::exit(0);
}

fn draw_frame(
    canvas: &mut sdl2::render::WindowCanvas,
    intersection: &map::Intersection,
    traffic: &TrafficLightController,
    vehicles: &VehicleManager,
) {
    use sdl2::pixels::Color;

    canvas.set_draw_color(Color::RGB(34, 139, 34));
    canvas.clear();

    intersection.draw(canvas);
    traffic.draw(canvas);
    vehicles.draw(canvas);
    ui::legend::draw(canvas);

    canvas.present();
}
