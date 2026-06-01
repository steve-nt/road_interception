# Team Tasks — Road Intersection Simulation

**Stack:** Rust + SDL2  
**Goal:** Working intersection sim — roads, adaptive traffic lights, vehicles with safe following distance, keyboard spawning.

---

## Constants ownership

Each person owns their constants in `src/constants.rs` when their task starts. Do not pre-fill values for later tasks.

### Person 1 — in `constants.rs` now

| Constant | Value | Notes |
|----------|-------|-------|
| `WINDOW_WIDTH` / `WINDOW_HEIGHT` | 800 × 800 | Window and canvas size |
| `ROAD_WIDTH` | 80 px | Road arm width for drawing |
| `LANE_LENGTH` | 400 px | Stop line → spawn point |

Capacity **formula** lives in `map/intersection.rs` (`lane_capacity()`). Person 3 supplies `vehicle_length` and `safety_gap` when implementing vehicles.

### Person 2 — add when starting traffic lights

| Constant | Notes |
|----------|-------|
| e.g. `BASE_GREEN_MS` | Minimum green phase duration |
| (others as needed) | Congestion thresholds, all-red buffer, etc. |

### Person 3 — add when starting vehicles

| Constant | Notes |
|----------|-------|
| `VEHICLE_LENGTH` | Car size; used in capacity + following distance |
| `SAFETY_GAP` | Gap between vehicles |
| `VEHICLE_SPEED` | Fixed velocity |
| `MIN_SPAWN_DISTANCE` | Anti-spam spacing (~`VEHICLE_LENGTH + SAFETY_GAP`) |

**Lane IDs (use everywhere):** `NorthIn`, `SouthIn`, `EastIn`, `WestIn` — one lane per direction entering the intersection.

**Route enum:** `Left | Straight | Right` — assigned randomly at spawn, never changed.

---

## Dependency graph

```
Person 1 (Map + render) ──┐
                          ├──► Integration (all three) ──► Audit-ready demo
Person 2 (Traffic lights) ┤
                          │
Person 3 (Vehicles + input)┘
```

**Integration order:** Person 1 first (window + lanes), then Person 3 (vehicles on lanes), then Person 2 (lights gate movement). Daily sync: lane API, vehicle positions, light states.

---

## Person 1 — Map, geometry & rendering

**Owner:** Person1  
**Branch suggestion:** `feature/map-render`

### Deliverables

- [x] SDL2 window, event loop skeleton, clear + present frame
- [x] Draw two crossing roads with **one lane per direction** (4 entry lanes)
- [x] Mark stop lines and intersection box (no-go zone for conflicting traffic)
- [x] Visual traffic-light placeholders at each entry (circles/boxes — Person 2 drives color)
- [x] `Lane` struct: id, spawn point, stop line, direction vector, vehicle queue slot
- [x] `capacity()` using `floor(lane_length / (vehicle_length + safety_gap))`
- [x] `src/map/mod.rs` + `src/map/intersection.rs` exports used by others

### Key files

| File | Responsibility |
|------|----------------|
| `src/constants.rs` | Window size, road width, lane length (Person 1 only) |
| `src/map/intersection.rs` | Lane definitions, coordinates, draw_roads() |
| `src/main.rs` | Init SDL2, call map render each frame |

### Acceptance criteria

- [x] Window opens; cross intersection visible with N/S/E/W entry lanes
- [x] Each lane exposes: spawn position, stop line, current queue length, max capacity
- [x] Other modules can call `intersection.get_lane(LaneId)` without knowing pixel math

### Interfaces for teammates

```rust
// Expose from map module — adjust names to match implementation
pub enum LaneId { NorthIn, SouthIn, EastIn, WestIn }
pub struct Lane {
    pub id: LaneId,
    pub queue_len: usize,
    // spawn_pos, stop_line, direction ...
}
pub fn capacity(&self, vehicle_length: f32, safety_gap: f32) -> usize;
pub fn lane_capacity(lane_length: f32, vehicle_length: f32, safety_gap: f32) -> usize;
```

---

## Person 2 — Traffic lights & intersection control

**Owner:** Person2
**Branch suggestion:** `feature/traffic-lights`

### Deliverables

- [ ] Red/green lights only at **each lane entry** (4 lights)
- [ ] Base cycle algorithm (e.g. pair N/S green, then E/W green)
- [ ] **Dynamic congestion rule:** if `queue_len >= capacity`, extend green for that approach
- [ ] Conflict matrix: never green two lanes whose paths would collide in the intersection
- [ ] `is_allowed(lane_id) -> bool` for Person 3 to query each frame
- [ ] Render light color (red/green) on Person 1's placeholders

### Key files

| File | Responsibility |
|------|----------------|
| `src/traffic_light/mod.rs` | Light state enum, controller struct |
| `src/traffic_light/controller.rs` | Timing, congestion adaptation, phase switching |

### Acceptance criteria

- No two conflicting movements have green simultaneously
- When a lane is at capacity, that approach gets priority (longer green or early switch)
- Person 3 can ask `controller.can_proceed(lane_id)` before moving a vehicle past stop line

### Algorithm sketch

1. Compute congestion ratio per lane: `queue_len / capacity`
2. Pick next green phase weighted toward most congested **compatible** group
3. Minimum green time + extension while `queue_len > 0` and below capacity target
4. All-red brief buffer optional (recommended for safety)

### Depends on

- Person 1: `LaneId`, lane queue length updates, light positions for drawing

---

## Person 3 — Vehicles, movement & keyboard input

**Owner:** Person3  
**Branch suggestion:** `feature/vehicles-input`

### Deliverables

- [ ] `Vehicle` struct: route (Left/Straight/Right), color by route, fixed speed, lane id
- [ ] Random route at spawn; route **immutable** for vehicle lifetime
- [ ] Movement along lane centerline; turn paths through intersection (waypoints or curves)
- [ ] Stop at red light (stop line); proceed on green
- [ ] **Following distance:** stop if gap to vehicle ahead < `VEHICLE_LENGTH + SAFETY_GAP`
- [ ] Remove vehicle after exiting intersection
- [ ] Keyboard: ↑ ↓ → ← spawn on corresponding side; `r` random side; `Esc` quit
- [ ] **Anti-spam:** reject spawn if last vehicle in lane too close to spawn point

### Key files

| File | Responsibility |
|------|----------------|
| `src/vehicle/mod.rs` | Vehicle struct, update(), draw() |
| `src/vehicle/route.rs` | Route enum, color mapping, path through intersection |
| `src/input/mod.rs` | Key handling, spawn cooldown per lane |

### Route colors (document for audit)

| Route | Color | RGB (example) |
|-------|-------|---------------|
| Left | _TBD_ | |
| Straight | _TBD_ | |
| Right | _TBD_ | |

### Acceptance criteria

- Arrow keys spawn only when safe distance allows
- Vehicles never overlap; no collisions in intersection
- Vehicles stop at red and at stopped lead vehicle
- `Esc` closes cleanly

### Depends on

- Person 1: lane geometry, spawn/stop positions
- Person 2: `can_proceed(lane_id)` at stop line

---

## Integration checklist (all three)

- [ ] `cargo build` and `cargo run` on each machine (macOS + Linux if applicable)
- [ ] Merge conflicts resolved in `main.rs` (keep one event loop)
- [ ] Queue length synced: Person 3 updates → Person 2 reads → Person 1 optional HUD
- [ ] README route-color table filled in
- [ ] Manual test script (5 min): spawn from all directions, fill one lane to capacity, verify green extension
- [ ] No `unwrap()` on user-facing paths without comment; prefer `expect("context")` in init only

---

## Suggested timeline

| Day | Focus |
|-----|--------|
| 1 | Agree constants; Person 1 window + static map; Person 2 + 3 stub modules |
| 2 | Person 3 single-lane movement; Person 2 basic N/S ↔ E/W cycle |
| 3 | Intersection turns + light gating; congestion adaptation |
| 4 | Input, anti-spam, polish, joint testing |
| 5 | Buffer: bonus sprites, stats, audit prep |

---

## Git conventions

- One feature branch per person; PR into `main`
- Commit message format: `[map]`, `[lights]`, `[vehicle]` prefix
- Do **not** commit `/target` or local SDL2 builds
- Push to: `https://platform.zone01.gr/git/kpetrout/road_intersection.git`

---

## Audit prep

Be ready to explain:

1. Your traffic light algorithm and how congestion changes timing
2. Route color legend
3. How following distance and spawn anti-spam work
4. How you prevent intersection collisions
