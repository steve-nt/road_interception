// Person 3 — keyboard input and vehicle spawning.
//
// Key bindings:
//   ↑  Spawn on SouthIn  (car comes from south, enters heading north)
//   ↓  Spawn on NorthIn  (car comes from north, enters heading south)
//   →  Spawn on WestIn   (car comes from west,  enters heading east)
//   ←  Spawn on EastIn   (car comes from east,  enters heading west)
//   R  Spawn on a random entry lane
//   Esc  Quit (handled in main.rs)

use sdl2::keyboard::Keycode;

use crate::map::intersection::{Intersection, LaneId};
use crate::vehicle::VehicleManager;

pub fn handle_key(key: Keycode, vehicles: &mut VehicleManager, intersection: &Intersection) {
    let lane_id = match key {
        Keycode::Up => Some(LaneId::SouthIn),
        Keycode::Down => Some(LaneId::NorthIn),
        Keycode::Right => Some(LaneId::WestIn),
        Keycode::Left => Some(LaneId::EastIn),
        Keycode::R => Some(random_lane()),
        _ => None,
    };

    if let Some(id) = lane_id {
        try_spawn(vehicles, intersection, id);
    }
}

fn try_spawn(vehicles: &mut VehicleManager, intersection: &Intersection, lane_id: LaneId) {
    // Copy spawn_point (it is a plain f32 struct, so Copy) before releasing
    // the borrow on intersection, then let VehicleManager handle anti-spam.
    if let Some(spawn_point) = intersection.get_lane(lane_id).map(|l| l.spawn_point) {
        vehicles.try_spawn(lane_id, spawn_point);
    }
}

fn random_lane() -> LaneId {
    use LaneId::{EastIn, NorthIn, SouthIn, WestIn};
    match rand::random::<u8>() % 4 {
        0 => NorthIn,
        1 => SouthIn,
        2 => EastIn,
        _ => WestIn,
    }
}
