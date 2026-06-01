use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use crate::map::intersection::LaneId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn index(self) -> usize {
        match self {
            Direction::North => 0,
            Direction::South => 1,
            Direction::East => 2,
            Direction::West => 3,
        }
    }

    pub fn from_lane(lane: LaneId) -> Self {
        match lane {
            LaneId::NorthIn => Direction::North,
            LaneId::SouthIn => Direction::South,
            LaneId::EastIn => Direction::East,
            LaneId::WestIn => Direction::West,
        }
    }

    #[allow(dead_code)]
    pub fn to_lane(self) -> LaneId {
        match self {
            Direction::North => LaneId::NorthIn,
            Direction::South => LaneId::SouthIn,
            Direction::East => LaneId::EastIn,
            Direction::West => LaneId::WestIn,
        }
    }

    fn random() -> Self {
        static COUNTER: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 4;
        match n {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        }
    }
}

#[derive(Debug)]
pub enum GameEvent {
    SpawnVehicle(Direction),
    Quit,
}

/// Keyboard input with per-direction spawn gating (cleared when the vehicle
/// leaves the spawn point — see `VehicleManager::update`).
pub struct InputHandler {
    spawn_blocked: [bool; 4],
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            spawn_blocked: [false; 4],
        }
    }

    pub fn poll(&mut self, event_pump: &mut EventPump) -> Vec<GameEvent> {
        let mut events = Vec::new();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => events.push(GameEvent::Quit),

                Event::KeyDown {
                    keycode: Some(kc),
                    repeat: false,
                    ..
                } => {
                    let dir = match kc {
                        Keycode::Up => Some(Direction::South),
                        Keycode::Down => Some(Direction::North),
                        Keycode::Right => Some(Direction::West),
                        Keycode::Left => Some(Direction::East),
                        Keycode::R => Some(Direction::random()),
                        _ => None,
                    };

                    if let Some(d) = dir {
                        if !self.spawn_blocked[d.index()] {
                            self.spawn_blocked[d.index()] = true;
                            events.push(GameEvent::SpawnVehicle(d));
                        }
                    }
                }

                _ => {}
            }
        }

        events
    }

    pub fn clear_spawn(&mut self, dir: Direction) {
        self.spawn_blocked[dir.index()] = false;
    }
}
