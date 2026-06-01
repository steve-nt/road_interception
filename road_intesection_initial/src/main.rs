mod constants;
mod input;
mod map;
mod traffic_light;
mod vehicle;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use map::intersection::Intersection;
use traffic_light::controller::TrafficLightController;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Road Intersection Simulation",
            constants::WINDOW_WIDTH,
            constants::WINDOW_HEIGHT,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut intersection = Intersection::new();
    let mut lights = TrafficLightController::new();
    let mut vehicles = vehicle::VehicleManager::new();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(key), .. } => {
                    if key == Keycode::Escape {
                        break 'running;
                    }
                    input::handle_key(key, &mut vehicles, &intersection);
                }
                _ => {}
            }
        }

        lights.update(&intersection);
        vehicles.update(&mut intersection, &lights);

        draw_frame(&mut canvas, &intersection, &lights, &vehicles)?;
    }

    Ok(())
}

fn draw_frame(
    canvas: &mut Canvas<Window>,
    intersection: &Intersection,
    lights: &TrafficLightController,
    vehicles: &vehicle::VehicleManager,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(40, 120, 40));
    canvas.clear();

    intersection.draw(canvas)?;
    lights.draw(canvas, intersection)?;
    vehicles.draw(canvas)?;

    canvas.present();
    Ok(())
}
