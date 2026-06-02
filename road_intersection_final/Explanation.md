# Road Intersection Simulation - Project Explanation

## Project Overview

The **Road Intersection Simulation** is a Rust-based traffic control system that simulates a 4-way intersection with traffic lights and autonomous vehicles. The project demonstrates:
- **Real-time graphics rendering** using SDL2
- **Collision avoidance algorithms**
- **Traffic flow optimization** with dynamic light control
- **Event-driven input handling**
- **State machine design patterns**

The simulation runs at 60 FPS on an 800×800 window and allows users to spawn vehicles using keyboard input to test the traffic management system.

---

## Architecture Overview

```
┌─────────────────────────────────────────┐
│           main.rs (Game Loop)           │
│  - Initializes SDL2 and systems         │
│  - Runs 60 FPS main event loop          │
│  - Coordinates all subsystems           │
└────────────┬────────────────────────────┘
             │
    ┌────────┼────────┬──────────┐
    │        │        │          │
    v        v        v          v
┌─────┐ ┌──────────┐ ┌────────┐ ┌──────────┐
│Input│ │ Traffic  │ │Vehicle │ │   Map    │
│Handler│ │ Lights  │ │Manager │ │Intersection
└─────┘ └──────────┘ └────────┘ └──────────┘
```

---

## Core Modules

### 1. **main.rs** - Game Loop & Orchestration

**Purpose**: The entry point that initializes SDL2, creates the game window, and runs the main event loop.

**Key Functions**:

#### `fn main()`
- Initializes SDL2 video context
- Creates 800×800 window titled "Road Intersection Simulation"
- Creates canvas for rendering with vsync enabled
- Instantiates subsystems:
  - `InputHandler`: Processes keyboard events
  - `TrafficLightController`: Manages traffic light states
  - `VehicleManager`: Manages vehicle spawning and updates
  - `Intersection`: Represents the static road structure
- Runs main loop at 60 FPS (16.67ms per frame)

**Main Loop Flow**:
```
frame_start = current_time
  → Poll input events
  → Handle spawn/quit events
  → Update vehicle positions and logic
  → Update traffic light states based on queue counts
  → Sync lane queues for visualization
  → Render all elements
  → Sleep to maintain 60 FPS
frame_end
```

#### `fn draw_frame()`
- Sets background color to green (grass)
- Draws intersection (roads and markings)
- Draws traffic lights
- Draws all vehicles with their route colors
- Draws legend panel in bottom-right
- Presents canvas to display

---

### 2. **constants.rs** - Layout & Configuration

**Purpose**: Centralized configuration values for intersection layout, vehicle behavior, and traffic control.

**Key Constants**:

#### Window & Intersection Layout
```rust
WINDOW_WIDTH: 800              // Total window width
WINDOW_HEIGHT: 800             // Total window height
LANE_WIDTH: 60                 // Width of each lane (2 per road)
ROAD_WIDTH: 120                // Total width of a road (2 lanes)
INTERSECTION_X/Y: 340          // Top-left corner of intersection
```

These values define the visual layout. The intersection is centered, with roads extending to all edges.

#### Vehicle Parameters
```rust
VEHICLE_LENGTH: 20             // Rendered size of vehicle (square)
SAFETY_GAP: 8                  // Minimum distance between vehicles
LANE_LENGTH: 340               // Distance from spawn to stop line
VEHICLE_SPEED: 2.0             // Pixels per frame movement
```

**Capacity Calculation**:
```
lane_capacity = floor(340 / (20 + 8)) = 12 vehicles per lane
```

#### Traffic Light Timing
```rust
BASE_GREEN_MS: 4,000           // Minimum green light duration (4 seconds)
ALL_RED_MS: 800                // Safety buffer between phases (0.8 seconds)
MAX_GREEN_MS: 12,000           // Maximum green when congested (12 seconds)
CONGESTION_THRESHOLD: 0.80     // 80% capacity triggers extended green
MIN_CONGESTED_QUEUE: 5         // Also trigger extended green at 5 vehicles
```

#### Spawn Points
```rust
SPAWN_NORTH: (340, 0)          // Top center
SPAWN_SOUTH: (460, 800)        // Bottom center
SPAWN_EAST: (800, 310)         // Right center
SPAWN_WEST: (0, 490)           // Left center
```

Each spawn point is offset to the appropriate lane.

---

### 3. **input/mod.rs** - Input Handling & Spawn Gating

**Purpose**: Processes keyboard input and prevents vehicle spawn spam by gating spawns per direction.

**Key Types**:

#### `enum Direction`
Represents which lane a vehicle enters from:
- `North`: Vehicle from top moving down
- `South`: Vehicle from bottom moving up
- `East`: Vehicle from right moving left
- `West`: Vehicle from left moving right

**Methods**:
- `index()`: Maps direction to array index (0-3)
- `from_lane()`: Converts lane ID to direction
- `random()`: Returns next direction in round-robin fashion using atomic counter

#### `enum GameEvent`
Represents events that affect the simulation:
- `SpawnVehicle(Direction)`: Create a new vehicle
- `Quit`: End simulation

#### `struct InputHandler`
Tracks which directions are currently blocked from spawning.

**Key Data**:
- `spawn_blocked: [bool; 4]`: One boolean per direction
  - `true`: Spawn is blocked (vehicle clearing spawn point)
  - `false`: Spawn is available

**Methods**:

##### `fn new() -> Self`
Creates InputHandler with all spawn directions available.

##### `fn poll(&mut self, event_pump: &mut EventPump) -> Vec<GameEvent>`
**Purpose**: Process SDL2 events and convert to GameEvents.

**Logic**:
1. Iterate through SDL2 events
2. Handle quit events (window close, ESC key)
3. Handle key-down events (not repeating):
   - Up arrow → `SpawnVehicle(Direction::South)`
   - Down arrow → `SpawnVehicle(Direction::North)`
   - Right arrow → `SpawnVehicle(Direction::West)`
   - Left arrow → `SpawnVehicle(Direction::East)`
   - R key → `SpawnVehicle(Direction::random())`
4. Check if spawn is blocked for this direction
5. If not blocked, add to events and mark direction as blocked
6. Return all generated events

**Key Feature**: The `repeat: false` condition ensures only the first key press is processed, not auto-repeat events.

##### `fn clear_spawn(&mut self, dir: Direction)`
Unlocks a direction for spawning. Called when a vehicle clears the spawn point.

---

### 4. **vehicle/mod.rs** - Vehicle Simulation

**Purpose**: Defines vehicle behavior, physics, collision avoidance, and state transitions.

**Key Types**:

#### `enum State`
Vehicles have 4 distinct states:

- **`Approaching`**: Vehicle moving toward intersection at constant velocity
  - Moves linearly toward stop line
  - Position updated: `x += vx * speed`, `y += vy * speed`
  - Transitions to `Waiting` if stop line reached and light is red
  - Transitions to entering intersection if light is green

- **`Waiting`**: Vehicle stopped at stop line, waiting for green light
  - Position unchanged
  - Waits for `can_proceed` signal from traffic light
  - Transitions to entering intersection when light turns green

- **`Turning { cx, cy, radius, theta, theta_end, ccw }`**: Vehicle following circular arc
  - Represents turning behavior (left or right)
  - Parameters:
    - `cx, cy`: Center of circular arc
    - `radius`: 30.0 for right turn, 90.0 for left turn
    - `theta`: Current angle on arc
    - `theta_end`: End angle
    - `ccw`: Counter-clockwise (right turn) or clockwise (left turn)
  - Velocity vector is tangent to circle: `vx = -sin(θ)`, `vy = cos(θ)`
  - Transitions to `Exiting` when theta reaches end

- **`Exiting`**: Vehicle leaving intersection
  - Moves linearly at constant velocity
  - Exits map in direction determined by route
  - Eventually moves off-screen and is removed

#### `struct Vehicle`
Represents a single vehicle in the simulation.

**Fields**:
```rust
pub x: f32, pub y: f32                 // Position
pub direction: Direction               // Spawn lane
pub route: Route                       // Destination (Straight/Left/Right)
pub color: Color                       // Visual color (route-dependent)
vx: f32, vy: f32                       // Velocity components
state: State                           // Current state machine state
spawn_cleared: bool                    // Has vehicle cleared spawn?
```

**Methods**:

##### `fn new(dir: Direction, route: Route) -> Self`
Creates a vehicle at spawn position with appropriate velocity vector.

##### `fn tick(&mut self, can_proceed: bool, max_step: f32) -> bool`
**Purpose**: Update vehicle position and state. Return `true` if vehicle just cleared spawn.

**Parameters**:
- `can_proceed`: Is the traffic light green for this lane?
- `max_step`: Maximum distance to move this frame

**Logic** (state machine transitions):

1. **Check spawn clearance**: 
   - If vehicle just passed `cleared_spawn()` threshold, return `true` and set flag
   
2. **Update based on state**:
   - **Approaching**: 
     - Move toward intersection: `x += vx * max_step`
     - If past stop line:
       - Snap to exact stop line
       - If light is green and not turning left: Enter intersection
       - Else: Wait
   
   - **Waiting**: 
     - If light turns green: Enter intersection
   
   - **Turning**: 
     - Calculate angular velocity: `d = VEHICLE_SPEED / radius`
     - Update angle: `new_theta = ccw ? theta + d : theta - d`
     - Update position: `x = cx + r*cos(θ)`, `y = cy + r*sin(θ)`
     - Update velocity tangent: `vx = -sin(θ)`, `vy = cos(θ)`
     - If reached `theta_end`: Transition to Exiting with exit velocity
   
   - **Exiting**: 
     - Move linearly: `x += vx * VEHICLE_SPEED`

##### `fn draw(&self, canvas: &mut WindowCanvas)`
Renders vehicle as a 20×20 colored square at its position.

##### `fn is_off_screen() -> bool`
Returns true if vehicle has exited all boundaries (used for cleanup).

##### `fn cleared_spawn() -> bool`
Returns true if vehicle has moved `VEHICLE_LENGTH + SAFETY_GAP` from spawn point in its direction of travel.

##### `fn past_stop_line() -> bool`
Checks if vehicle has crossed the intersection stop line.

##### `fn enter_intersection(&mut self)`
Transitions vehicle to turning or exiting based on route:
- **Straight**: Immediately exit (pass through)
- **Right**: Set up right-turn circular arc
- **Left**: Set up left-turn circular arc

---

### 5. **vehicle/route.rs** - Route Definition

**Purpose**: Defines vehicle routing options and their visual representations.

#### `enum Route`
Represents vehicle destination:
- `Straight`: Continue through intersection
- `Left`: Turn left at intersection
- `Right`: Turn right at intersection

**Methods**:

##### `fn random() -> Self`
Uses atomic counter for deterministic round-robin distribution:
- Counter increments by 2 each call
- `counter % 3` maps to route (avoids bias)
- Ensures equal distribution: Straight, Left, Right, Straight, ...

##### `fn color(self) -> Color`
Returns visual color for each route (for audit verification):
- **Straight**: Blue `RGB(0, 80, 220)`
- **Left**: Yellow `RGB(230, 190, 0)`
- **Right**: Orange `RGB(220, 100, 0)`

---

### 6. **vehicle/mod.rs (continued)** - VehicleManager

**Purpose**: Manages the collection of vehicles, spawning, updates, and rendering.

#### `struct VehicleManager`
Holds the current list of active vehicles.

**Methods**:

##### `fn new() -> Self`
Creates manager with empty vehicle list.

##### `fn spawn(&mut self, dir: Direction)`
Creates new vehicle with random route and adds to list.

##### `fn update(&mut self, traffic_lights: &TrafficLightController, input_handler: &mut InputHandler)`
**Purpose**: Update all vehicles and manage spawn gating.

**Logic**:
1. For each vehicle:
   - Get traffic light state for this lane
   - Calculate max step based on collision avoidance
   - Call `vehicle.tick(can_proceed, max_step)`
   - If vehicle just cleared spawn: `input_handler.clear_spawn(dir)`
   
2. Remove off-screen vehicles: `retain(|v| !v.is_off_screen())`

**Collision Avoidance**:
- `perpendicular_occupying()`: Block if perpendicular traffic in intersection
- `right_turn_exit_blocked()`: Block if opposing vehicle turning left
- `opposing_blocks_left()`: Block if opposing traffic present
- Calculates safe movement distance to prevent collisions

##### `fn draw(&self, canvas: &mut WindowCanvas)`
Renders all vehicles with their route colors.

##### `fn queue_counts(&self) -> LaneQueueCounts`
**Purpose**: Count vehicles in each lane for traffic light control.

**Returns**: `[u32; 4]` array with counts:
- Index 0: North lane vehicles approaching
- Index 1: South lane vehicles approaching
- Index 2: East lane vehicles approaching
- Index 3: West lane vehicles approaching

Counts are based on position relative to stop lines and state (not in intersection yet).

---

### 7. **traffic_light/controller.rs** - Traffic Light Control

**Purpose**: Implements traffic light state machine and dynamic congestion-based control.

**Key Types**:

#### `enum LightState`
- `Red`: Vehicle must stop
- `Green`: Vehicle may proceed

#### `enum EntryLane`
Represents the 4 entry lanes:
- `North`, `South`, `East`, `West`

**Methods**:
- `index()`: Maps to array index (0-3)
- `all()`: Returns array of all lanes

#### `enum Phase`
Represents traffic light phases:
- `NorthSouthGreen`: North and South lights green, East/West red
- `AllRed`: All lights red (safety buffer)
- `EastWestGreen`: East and West lights green, North/South red

#### `struct TrafficLight`
Individual light at a lane entry:
```rust
state: LightState              // Red or Green
position: (i32, i32)           // Screen coordinates
```

#### `struct TrafficLightController`
Master controller managing all 4 lights and phase timing.

**Fields**:
```rust
lights: [TrafficLight; 4]      // One light per lane
phase: Phase                   // Current phase
phase_timer: Duration          // Time in current phase
next_green_axis: GreenAxis     // Which axis gets green next
```

**Methods**:

##### `fn new() -> Self`
Initializes all lights to red, phase to AllRed, ready to start.

##### `fn is_green(&self, lane: EntryLane) -> bool`
Returns whether traffic light is green for given lane.

##### `fn update(&mut self, delta: Duration, queue_counts: LaneQueueCounts)`
**Purpose**: Update phase based on timers and congestion.

**Logic**:
1. Add `delta` to `phase_timer`
2. Handle each phase:
   - **NorthSouthGreen**: 
     - If `should_end_green([North, South], counts)` returns true: Begin AllRed
   - **EastWestGreen**: 
     - If `should_end_green([East, West], counts)` returns true: Begin AllRed
   - **AllRed**: 
     - If `phase_timer >= ALL_RED_MS` (800ms): Advance from AllRed

##### `fn should_end_green(&self, lanes: [EntryLane; 2], queue_counts: LaneQueueCounts) -> bool`
**Purpose**: Determine if green phase should end for given lanes.

**Logic**:
1. Check if any lane is congested: `is_lane_congested(count)`
2. If `phase_timer >= MAX_GREEN_MS`: End (force switch)
3. If `phase_timer < BASE_GREEN_MS`: Keep green (minimum duration)
4. If any lane is congested: Keep green (extend to ease congestion)
5. Otherwise: End green (switch to other axis)

**Congestion Detection**:
```
is_congested = (count >= 5) || (count >= 80% * 12)
             = (count >= 5) || (count >= 9.6, rounded to 10)
```

##### `fn begin_all_red(&mut self)`
Transitions to AllRed phase:
- Determines next green axis (alternates)
- Sets phase to AllRed
- Resets timer to zero
- Applies all red lights

##### `fn advance_from_all_red(&mut self)`
Transitions from AllRed to green phase:
- Switches to `next_green_axis`
- Prepares opposite axis for next turn
- Resets timer
- Applies green lights

##### `fn apply_phase_lights(&mut self)`
Sets all 4 lights based on current phase:
- Green lanes: `LightState::Green`
- Red lanes: `LightState::Red`

##### `fn draw(&self, canvas: &mut WindowCanvas)`
Renders all 4 traffic lights at their positions.

---

### 8. **map/intersection.rs** - Intersection Layout

**Purpose**: Represents the static intersection geometry and renders roads.

**Key Types**:

#### `enum LaneId`
Identifies lanes:
- `NorthIn`, `SouthIn`, `EastIn`, `WestIn`

#### `struct Lane`
Data about a single approach lane:
```rust
id: LaneId                     // Which lane
queue_len: usize               // Number of vehicles in lane (for display)
```

#### `struct Intersection`
Holds all lanes and renders the road structure.

**Methods**:

##### `fn new() -> Self`
Creates intersection with 4 lanes, zero vehicles in each.

##### `fn draw(&self, canvas: &mut WindowCanvas)`
Renders the intersection:
1. `draw_surface()`: Fills road areas with gray
2. `draw_center_lines()`: Draws dashed yellow lane markings
3. `draw_stop_lines()`: Draws white stop lines at 4 entries

**Rendering Functions**:

- **`draw_surface()`**: 
  - Fills vertical road (ROAD_WIDTH × WINDOW_HEIGHT)
  - Fills horizontal road (WINDOW_WIDTH × ROAD_WIDTH)
  - Creates two intersecting roads

- **`draw_center_lines()`**: 
  - Dashed lines down center of each road
  - Yellow color indicates lane separation
  - Pattern: 20px dash, 15px gap

- **`draw_stop_lines()`**: 
  - White lines at each entry point to intersection
  - 60 pixels long (one lane width)
  - Marks where vehicles must stop at red light

---

### 9. **ui/legend.rs** - UI Legend Panel

**Purpose**: Displays route-to-color mapping for audit verification.

**Functions**:

#### `fn draw(canvas: &mut WindowCanvas)`
Renders legend panel in bottom-right corner (118×88 pixels):
1. Fills background (dark green)
2. Draws border (gray)
3. Draws "Routes" title
4. For each route (Left, Straight, Right):
   - Draws color swatch (12×12)
   - Draws label text with custom glyphs

#### `fn draw_text()`, `fn draw_text_clipped()`
Text rendering helper that wraps at max_x coordinate.

#### `fn draw_char()`
Renders single character using pixel-by-pixel glyph data.

#### `fn glyph(ch: char) -> Option<[u8; 7]>`
Returns 5×7 pixel representation of a character:
- Each byte is one row (bits represent columns)
- Supports: A-Z, lowercase variants, numbers as needed
- Enables text rendering without external font library

---

## Data Flow During Simulation

### Spawn Cycle
```
User presses Arrow Key
  ↓
InputHandler.poll() → detects keycode
  ↓
Checks spawn_blocked[direction]
  ↓
If not blocked: Sets spawn_blocked[direction] = true
  ↓
Creates GameEvent::SpawnVehicle(direction)
  ↓
main.rs receives event → VehicleManager.spawn(direction)
  ↓
VehicleManager creates Vehicle with random Route
  ↓
Vehicle.draw() shows colored square at spawn point
  ↓
Vehicle.tick() moves vehicle forward
  ↓
When Vehicle.cleared_spawn() = true → InputHandler.clear_spawn()
  ↓
Spawn is unblocked for this direction
```

### Vehicle Movement Cycle
```
VehicleManager.update()
  ↓
For each vehicle:
  → Calculate traffic light state for this lane
  → Check perpendicular/opposing vehicles for collisions
  → Determine safe max_step
  → Call vehicle.tick(can_proceed, max_step)
    → Update position/state based on current State
    → Transition states if needed
  → Return whether vehicle cleared spawn
  → If cleared: unlock direction in InputHandler
  ↓
Remove off-screen vehicles
  ↓
Return queue_counts [u32; 4]
```

### Traffic Light Cycle
```
TrafficLightController.update(delta, queue_counts)
  ↓
Increment phase_timer by delta
  ↓
Based on current phase:
  ├─ NorthSouthGreen:
  │   If should_end_green() → begin_all_red()
  ├─ EastWestGreen:
  │   If should_end_green() → begin_all_red()
  └─ AllRed:
      If timer ≥ 800ms → advance_from_all_red()
  ↓
should_end_green() checks:
  ├─ Any lane congested?
  ├─ Timer exceeds maximum?
  └─ Timer below minimum?
  ↓
Update lights: apply_phase_lights()
  ↓
VehicleManager queries is_green() for traffic decisions
```

### Collision Avoidance Logic
```
Vehicle at stop line, about to enter intersection
  ↓
Query traffic lights: Is light green?
  ├─ No → Wait (State::Waiting)
  └─ Yes → Check collision rules:
      ├─ perpendicular_occupying(): Any NS/EW vehicle in box?
      ├─ right_turn_exit_blocked(): Opposite vehicle turning left?
      ├─ opposing_blocks_left(): If I turn left, is opposite in way?
      └─ If all clear → Enter intersection
```

---

## Key Algorithms

### 1. Circular Arc Navigation
When turning left or right, vehicles follow a perfect circle:
```
Position: x = cx + r*cos(θ), y = cy + r*sin(θ)
Velocity: v = r * dθ/dt  →  dθ = speed / r
Angle: θ(t+1) = θ(t) ± dθ
```
- Right turn: Small radius (30), counter-clockwise
- Left turn: Large radius (90), clockwise
- Transition to exit when θ reaches θ_end

### 2. Dynamic Traffic Light Timing
```
If phase ≥ MAX_GREEN:   End (force switch)
If phase < MIN_GREEN:   Continue (minimum duration)
If any lane congested:  Continue (manage queue)
Else:                   End (smooth traffic)
```

### 3. Congestion Detection
```
capacity = floor(lane_length / (vehicle_length + gap))
         = floor(340 / 28) = 12

is_congested = (count ≥ MIN_QUEUE) || (count ≥ 80% * 12)
             = (count ≥ 5) || (count ≥ 10)
```

### 4. Left Turn Yielding (Vehicle Priority)
```
N/S to N/S: No yield needed
E/W to E/W: No yield needed
N/S to E/W: E/W has priority if:
  - Different axis
  - Left turning
  - In intersection or approaching
```

---

## Event Loop Timing

### Target: 60 FPS = 16.67ms per frame

```
Frame Timer
│
├─ Frame Start: ~0ms
│
├─ Input Polling: ~1-2ms
│   └─ SDL2.poll_events()
│
├─ Vehicle Updates: ~3-5ms
│   └─ 20 vehicles × movement + collision checks
│
├─ Traffic Light Update: <1ms
│   └─ Phase transition logic
│
├─ Lane Sync: <1ms
│   └─ Copy vehicle counts
│
├─ Rendering: ~8-10ms
│   ├─ Clear screen
│   ├─ Draw roads
│   ├─ Draw lights
│   ├─ Draw vehicles
│   └─ Present to display
│
├─ Frame End: ~16.67ms
│
└─ Sleep: ~1-3ms (to maintain 60 FPS)
```

---

## Extensibility

### Adding New Features

1. **New Lane Type**: Add `EntryLane` variant, update light array
2. **Pedestrians**: New module, add collision checks
3. **Speed Variation**: Add `speed: f32` field to Vehicle
4. **Accidents**: Track collision events, create stopped vehicles
5. **Statistics**: Add counters to VehicleManager
6. **Replay System**: Record all events, replay from log
7. **Network Sync**: Send vehicle updates over network
8. **AI Spawning**: Add `Direction::automated()` to InputHandler

### Performance Improvements

1. **Spatial Partitioning**: Divide intersection into grid cells
2. **Object Pooling**: Reuse Vehicle structs
3. **GPU Rendering**: Move to compute shaders
4. **Multi-threading**: Process vehicles in parallel
5. **SIMD**: Vectorize position calculations

---

## Summary

The Road Intersection Simulation is a well-architected, modular project demonstrating:

- **Clean separation of concerns** (Input, Traffic, Vehicles, Map, UI)
- **State machine design** for vehicles and traffic phases
- **Real-time graphics** with efficient SDL2 rendering
- **Collision avoidance** through multi-layered checking
- **Adaptive traffic control** based on congestion
- **Frame-rate independent** timing with delta updates
- **Deterministic behavior** for reproducible testing

The codebase is easily extensible for additional features while maintaining clean interfaces between modules.

