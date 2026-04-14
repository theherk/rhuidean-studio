# Rhuidean Studio

<p align="center">
  <img src="assets/rhuidean-studio.svg" alt="Rhuidean Studio" width="200">
</p>

A polyrhythmic orbital music visualizer. Concentric orbiting circles traverse rings at different angular velocities determined by frequency ratios from just intonation. When each circle crosses a reference line, it triggers a tone whose pitch corresponds to its orbital radius. Over time the system oscillates between chaos and order, eventually converging when all orbits realign simultaneously.

## Web

Try it live at [rhuidean.studio](https://rhuidean.studio).

## TUI

Install from [crates.io](https://crates.io/crates/rhuidean-studio):

```sh
cargo install rhuidean-studio
```

Run:

```sh
rhuidean-studio
```

### Controls

| Key | Action |
|-----|--------|
| Space | Play / Pause |
| Tab | Toggle controls panel |
| j / Down | Move cursor down |
| k / Up | Move cursor up |
| h / Left | Decrease / previous |
| l / Right | Increase / next |
| r | Reset orbits |
| r r | Reset to defaults (double-tap) |
| y | Yank share command |
| ? | Help |
| q / Esc | Quit |

## Concept

The interface presents a set of frequency ratio presets (e.g. 9/7, 13/11) drawn from musical interval theory. These ratios control the relationship between the angular velocities of the innermost and outermost orbits. Intermediate orbits are interpolated between these extremes using a configurable distribution mode.

Each orbit is a concentric track with a circle traveling along it. When a circle crosses the reference radius (the 12 o'clock line), it emits a tone. The pitch is determined by the orbit's distance from the center, mapped through a configurable tuning system.

> [!NOTE]
> See [Backstory](#backstory) near the bottom for some more non-technical information.

## Ratio Presets

The following frequency ratios are available as presets, grouped by interval size:

### Seconds

| Ratio | Cents | Interval Name              |
| ----- | ----- | -------------------------- |
| 9/8   | ~204  | Major second (Pythagorean) |
| 8/7   | ~231  | Septimal whole tone        |

### Thirds

| Ratio | Cents | Interval Name            |
| ----- | ----- | ------------------------ |
| 7/6   | ~267  | Septimal minor third     |
| 13/11 | ~289  | Tridecimal neutral third |
| 6/5   | ~316  | Just minor third         |
| 11/9  | ~347  | Undecimal neutral third  |
| 5/4   | ~386  | Just major third         |
| 9/7   | ~435  | Septimal major third     |

### Fourths

| Ratio | Cents | Interval Name         |
| ----- | ----- | --------------------- |
| 4/3   | ~498  | Perfect fourth        |
| 11/8  | ~551  | Undecimal superfourth |

### Tritones

| Ratio | Cents | Interval Name    |
| ----- | ----- | ---------------- |
| 7/5   | ~583  | Septimal tritone |

### Fifths

| Ratio | Cents | Interval Name             |
| ----- | ----- | ------------------------- |
| 3/2   | ~702  | Perfect fifth             |
| 11/7  | ~782  | Undecimal augmented fifth |

### Sixths

| Ratio | Cents | Interval Name            |
| ----- | ----- | ------------------------ |
| 8/5   | ~814  | Just minor sixth         |
| 13/8  | ~841  | Tridecimal neutral sixth |
| 5/3   | ~884  | Just major sixth         |

### Sevenths

| Ratio | Cents | Interval Name             |
| ----- | ----- | ------------------------- |
| 7/4   | ~969  | Harmonic seventh          |
| 11/6  | ~1049 | Undecimal neutral seventh |
| 15/8  | ~1088 | Just major seventh        |

### Octave

| Ratio | Cents | Interval Name |
| ----- | ----- | ------------- |
| 2/1   | 1200  | Octave        |

Custom ratios can also be entered manually. If a custom ratio matches a known interval, its name is displayed automatically.

## Configuration

### Number of Orbits

Configurable from 2 to 32 (default: 12). Can be grounded in tonation systems; for example, 12 maps naturally to chromatic tunings.

### Velocity Distribution

Controls how angular velocities are interpolated between the inner and outer orbits:

- **Linear**: Velocities evenly spaced between the ratio endpoints.
- **Geometric**: Exponential interpolation, `v(i) = p * (q/p)^t`. Produces logarithmically spaced speeds.
- **Inverse-Square**: Kepler-like relationship where velocity scales with the inverse square of the radius, calibrated so endpoints match the ratio.

### Tuning System

Maps orbit index to pitch frequency:

- **Overtone Series**: `f(n) = base * (n + 1)` (natural harmonic series).
- **Equal Temperament**: `f(n) = base * 2^(n/N)` (N-TET where N is the orbit count).
- **Just Intonation**: Pure frequency ratios derived from small whole numbers.
- **Pythagorean**: Stacked perfect fifths (3:2 ratio).

### Scale

Optionally assign each orbit to successive degrees of a musical scale, bypassing the tuning system. Available scales include diatonic modes (Ionian through Locrian), pentatonic (major/minor), blues, whole tone, harmonic minor, melodic minor, and chromatic.

### Waveform

Oscillator type for tone generation: Sine, Square, Triangle, or Sawtooth.

### Speed

Playback speed multiplier for slow-motion or fast-forward.

## Architecture

```
rhuidean-studio/
├── Cargo.toml            # workspace root
├── core/                 # shared simulation, tuning, and scale logic
│   └── src/
│       ├── lib.rs
│       ├── simulation.rs
│       ├── tuning.rs
│       └── scale.rs
├── web/                  # WASM web frontend
│   ├── src/
│   │   ├── lib.rs        # wasm entry point
│   │   ├── audio.rs      # Web Audio API wrapper
│   │   └── renderer.rs   # Canvas 2D drawing
│   └── www/
│       ├── index.html
│       ├── main.js
│       └── style.css
├── tui/                  # terminal UI
│   └── src/
│       ├── main.rs       # event loop
│       ├── app.rs        # application state
│       ├── audio.rs      # cpal audio engine
│       ├── renderer.rs   # terminal visualization
│       └── ui.rs         # controls panel
├── assets/
└── Makefile
```

### Stack

- **Core**: Pure Rust simulation, tuning systems, scale logic (no platform dependencies)
- **Web**: Rust/WASM via `wasm-pack`, `web-sys` for Canvas 2D and Web Audio API, vanilla JS controls
- **TUI**: `ratatui` + `crossterm` for terminal rendering, `cpal` for native audio synthesis

### Build

```sh
# Web
cargo install wasm-pack
wasm-pack build --target web web
python3 -m http.server 8090
# visit http://localhost:8090/web/www/

# TUI
cargo build --release -p rhuidean-studio
./target/release/rhuidean-studio
```

## Cycle Length

For a ratio p/q (reduced to lowest terms), the cycle completes when all orbits return to their starting positions. With linear interpolation, orbit `i` has velocity `v(i) = q + i * (p - q) / (N - 1)`. The total cycle period is determined by the LCM of all orbit periods. For integer harmonic mode, this resolves cleanly; for linear mode with rational velocities, it is `LCM(denominators) / GCD(numerators)` of the velocity fractions.

## Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features.

## Backstory

Many years ago, in the early internet days, I came across a website. I've sought it again for years and even reached out to many professional contacts to find it. I never did, so now I want to attempt to recreate it.

It was a web interface that consisted of a few options. If I remember correctly, they were options much like those available here. The outcome was a ring around which travelled smaller orbiting circles. These circles were evenly distributed from near the center to the circumference along a radius. When the animation began, they started to travel. The difference in angular velocity between the inner and outer orbits varied based on the set values as well. When each orbiting circle passed the original radius, it would make a sound whose note was in accordance with the distance from the center.

Over time it would vacillate through cycles of chaos and order depending on the input values. At some point the cycle would loop when they all passed the original radius at the precise same moment again, producing the most satisfying sound imaginable. I sought this for decades, but it really depressed me that I never found it again, so I've been tinkering with this for a while, and I'm finally satisfied. It lives again.
