# Road Intersection Simulation

Traffic control simulation for a four-way intersection. Vehicles spawn from four directions, follow a fixed route (left / straight / right), obey red/green lights, and maintain safe following distance. Traffic lights adapt to lane congestion.

Built with **Rust** and **SDL2** ([docs](https://docs.rs/sdl2/0.34.3/sdl2/)).

## Prerequisites

### macOS

```bash
brew install sdl2
```

If you still get `library 'SDL2' not found`, the repo includes `.cargo/config.toml` for Homebrew paths on macOS. As a fallback:

```bash
export LIBRARY_PATH="$(brew --prefix sdl2)/lib:$LIBRARY_PATH"
export CPATH="$(brew --prefix sdl2)/include:$CPATH"
cargo run
```

### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install libsdl2-dev
```

## Run

```bash
cargo run
```

Release build:

```bash
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| ↑ | Spawn vehicle from **south** (moving north) |
| ↓ | Spawn vehicle from **north** (moving south) |
| → | Spawn vehicle from **west** (moving east) |
| ← | Spawn vehicle from **east** (moving west) |
| `r` | Spawn from a **random** direction |
| `Esc` | Exit simulation |

Vehicles cannot be spammed; spawns respect safe distance from the last vehicle in that lane.

## Route colors (audit)

Shown in the **bottom-right legend** during the simulation and in this table:

| Route | Color | RGB |
|-------|-------|-----|
| Straight | Blue | (0, 80, 220) |
| Left | Yellow | (230, 190, 0) |
| Right | Orange | (220, 100, 0) |

Each car picks a random **turn** at spawn (left / straight / right through the intersection). Color always matches that choice.

## Project layout

```
src/
├── main.rs              # SDL2 init, main loop, rendering
├── lib.rs               # Module exports
├── constants.rs         # Shared simulation constants
├── map/                 # Roads, lanes, intersection geometry
├── traffic_light/       # Light state + adaptive control logic
├── vehicle/             # Vehicle model, movement, following distance
└── input/               # Keyboard spawning + anti-spam
```

## Team workflow

See [tasks.md](./tasks.md) for work split across three developers.

Integration branch: merge feature branches into `main` only when `cargo build` passes.

## Congestion formula

```
capacity = floor(lane_length / (vehicle_length + safety_gap))
```

When a lane queue reaches capacity, extend green time for that approach to avoid overflow.

## Bonus ideas

- Sprite-based vehicles and light animations ([limezu](https://limezu.itch.io/), [finalbossblues](https://finalbossblues.itch.io/), [spriters-resource](https://www.spriters-resource.com/))
- Pause / speed controls
- End-of-run statistics
