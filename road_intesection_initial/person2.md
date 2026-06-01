# Person 2 — Traffic Lights & Intersection Control

## `src/constants.rs` (Person 2 section)

```rust
// ── Person 2: traffic-light timing ───────────────────────────────────────────

/// Minimum green-phase duration (accumulates at 16 ms/frame).
pub const BASE_GREEN_MS: u32 = 3_000;

/// Extra green time awarded once when the active approach is congested.
pub const CONGESTION_EXTENSION_MS: u32 = 2_000;

/// All-red inter-phase buffer.
pub const ALL_RED_MS: u32 = 500;
```

---

## `src/traffic_light/mod.rs`

```rust
pub mod controller;
```

---

## `src/traffic_light/controller.rs`

```rust
// Person 2 — adaptive traffic-light controller.
//
// Phase cycle: NorthSouth green → all-red buffer → EastWest green → all-red → …
// Congestion rule: if queue_len >= capacity for any lane in the active phase,
// the green is extended once by CONGESTION_EXTENSION_MS before switching.

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::constants::{
    ALL_RED_MS, BASE_GREEN_MS, CONGESTION_EXTENSION_MS, SAFETY_GAP, VEHICLE_LENGTH,
};
use crate::map::intersection::{Intersection, LaneId};

/// Traffic signal colour for a single lane entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightColor {
    Red,
    Green,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    NorthSouth,
    EastWest,
}

/// Internal controller state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CtrlState {
    Green,
    AllRed,
}

/// Drives all four entry lights with conflict-free phase switching and
/// queue-based green extension.
pub struct TrafficLightController {
    phase: Phase,
    state: CtrlState,
    timer_ms: u32,
    /// True once a congestion extension has been awarded for the current phase.
    extended: bool,
    lights: [(LaneId, LightColor); 4],
}

impl TrafficLightController {
    pub fn new() -> Self {
        let mut ctrl = Self {
            phase: Phase::NorthSouth,
            state: CtrlState::Green,
            timer_ms: 0,
            extended: false,
            lights: [
                (LaneId::NorthIn, LightColor::Red),
                (LaneId::SouthIn, LightColor::Red),
                (LaneId::EastIn,  LightColor::Red),
                (LaneId::WestIn,  LightColor::Red),
            ],
        };
        ctrl.apply_phase();
        ctrl
    }

    /// Returns true when the given lane may proceed past its stop line.
    pub fn can_proceed(&self, lane: LaneId) -> bool {
        self.lights
            .iter()
            .find(|(id, _)| *id == lane)
            .map(|(_, c)| *c == LightColor::Green)
            .unwrap_or(false)
    }

    /// Advance the controller by one frame (~16 ms).
    pub fn update(&mut self, intersection: &Intersection) {
        self.timer_ms = self.timer_ms.saturating_add(16);

        match self.state {
            CtrlState::Green => {
                if self.timer_ms < BASE_GREEN_MS {
                    return;
                }
                // Award extension once when the active approach is congested.
                if !self.extended && phase_congested(intersection, self.phase) {
                    self.extended = true;
                }
                let max_green = if self.extended {
                    BASE_GREEN_MS + CONGESTION_EXTENSION_MS
                } else {
                    BASE_GREEN_MS
                };
                if self.timer_ms >= max_green {
                    self.state = CtrlState::AllRed;
                    self.timer_ms = 0;
                    self.set_all_red();
                }
            }
            CtrlState::AllRed => {
                if self.timer_ms >= ALL_RED_MS {
                    self.phase = flip_phase(self.phase);
                    self.extended = false;
                    self.timer_ms = 0;
                    self.state = CtrlState::Green;
                    self.apply_phase();
                }
            }
        }
    }

    fn apply_phase(&mut self) {
        let (ns, ew) = match self.phase {
            Phase::NorthSouth => (LightColor::Green, LightColor::Red),
            Phase::EastWest   => (LightColor::Red,   LightColor::Green),
        };
        for (id, color) in &mut self.lights {
            *color = match id {
                LaneId::NorthIn | LaneId::SouthIn => ns,
                LaneId::EastIn  | LaneId::WestIn  => ew,
            };
        }
    }

    fn set_all_red(&mut self) {
        for (_, c) in &mut self.lights {
            *c = LightColor::Red;
        }
    }

    /// Render the signal colour on Person 1's placeholder housing.
    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        intersection: &Intersection,
    ) -> Result<(), String> {
        for (id, color) in self.lights {
            let (x, y) = intersection.light_position(id);
            canvas.set_draw_color(match color {
                LightColor::Red   => Color::RGB(200, 40,  40),
                LightColor::Green => Color::RGB(40,  200, 60),
            });
            canvas.fill_rect(Rect::new(x - 5, y - 5, 10, 10))?;
        }
        Ok(())
    }
}

impl Default for TrafficLightController {
    fn default() -> Self {
        Self::new()
    }
}

fn flip_phase(p: Phase) -> Phase {
    match p {
        Phase::NorthSouth => Phase::EastWest,
        Phase::EastWest   => Phase::NorthSouth,
    }
}

/// Returns true if any lane in the active phase has reached capacity.
/// Uses Person 3's VEHICLE_LENGTH + SAFETY_GAP to compute lane capacity.
fn phase_congested(intersection: &Intersection, phase: Phase) -> bool {
    let ids = match phase {
        Phase::NorthSouth => [LaneId::NorthIn, LaneId::SouthIn],
        Phase::EastWest   => [LaneId::EastIn,  LaneId::WestIn],
    };
    ids.iter().any(|&id| {
        intersection
            .get_lane(id)
            .map(|l| l.queue_len >= l.capacity(VEHICLE_LENGTH, SAFETY_GAP))
            .unwrap_or(false)
    })
}
```

---

## How it works

### Phase cycle

```
NorthSouth GREEN (3 s min)
       │
       ▼  [congested? +2 s extension, awarded once]
ALL-RED (500 ms)
       │
       ▼
EastWest GREEN (3 s min)
       │
       ▼  [congested? +2 s extension]
ALL-RED (500 ms)
       │
       └──► repeat
```

### Conflict matrix

N/S and E/W are never green at the same time — enforced by `apply_phase()` which always sets exactly one pair to green and the other to red.

### Congestion rule

`phase_congested()` calls `lane.capacity(VEHICLE_LENGTH, SAFETY_GAP)` (Person 1's formula: `floor(LANE_LENGTH / (VEHICLE_LENGTH + SAFETY_GAP))` = **14 vehicles**).  
If `queue_len >= 14` for either lane in the active phase, the green extends by 2 s — once per phase.

### Interface for Person 3

```rust
controller.can_proceed(lane_id) -> bool
```
Call this every frame before moving a vehicle past its stop line.
