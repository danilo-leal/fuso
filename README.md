# Time Zone Converter

A native Rust desktop starter for comparing the same time-zone app built with
[GPUI Component](https://github.com/longbridge/gpui-component) and
[Base GPUI](https://github.com/LukeTandjung/base-gpui). Both use
[GPUI](https://www.gpui.rs/) underneath; Base GPUI is a headless component library,
not a different name for the underlying GPUI framework.

The project lives at `/Users/danilo/Documents/GitHub/timezone-converter`.
It is a local folder, not a Git repository, and nothing has been published or pushed.

For a guided tour, open [the interactive architecture guide](2026-10-12-architecture.html)
in a browser. It explains the shared core, each frontend's state ownership, and the
separate dependency graphs, with a timeline demonstration and a five-question quiz.

## Run either version

On this Mac, Rust 1.92.0 and full Xcode are already installed. The toolchain is
pinned in `rust-toolchain.toml`. On another Mac, install Rust through rustup and
full Xcode, finish Xcode's first-launch setup, and ensure `xcode-select -p` points
to Xcode. This starter has been built and launched on macOS; other platforms have
not been validated.

From Terminal:

```sh
cd ~/Documents/GitHub/timezone-converter

# Styled, ready-made widgets (the default app)
cargo run --locked -p timezone-component

# Headless widgets, with styling defined in this project
cargo run --locked --manifest-path apps/base/Cargo.toml
```

These launch two separate apps, not two modes inside one window. You can run them
in separate terminals for comparison. Close the window or press Control-C in its
terminal to exit. No dev server, API key, browser, or extra asset download is needed
at runtime. The initial dependency fetch/build does need internet access.

## Try the prototype

- The reference zone starts with your Mac's IANA time zone. If detection fails,
  the app explicitly reports that it is using UTC.
- The app starts with 3 initial cities (Sao Paulo, New York, London).
- Reference buttons allow selecting any currently tracked city or UTC.
  Selecting a reference changes the presentation, not the shared instant.
- Each city has its own 24-hour slider showing its local time. Dragging **any**
  slider adjusts all other city sliders in lockstep to reflect the shared instant.
- **Day & Night Visual Gradient**:
  - Each slider track displays a smooth 24-hour gradient indicating nighttime and daytime:
    deep night (00:00) → sunrise dawn (06:00) → bright midday sun (12:00) → sunset dusk (18:00) → deep night (24:00).
  - Time ticks (`🌙 00:00`, `🌅 06:00`, `☀️ 12:00`, `🌇 18:00`, `🌙 24:00`) mark key solar transition points.
  - A real-time day/night badge (`☀️ Day`, `🌙 Night`, `🌅 Dawn`, `🌇 Dusk`) appears alongside each city's time.
- **Add & Remove Cities**:
  - Remove any city with the **✕ Remove** button in its row (a minimum of 1 city is preserved).
  - Add cities via the **+ Add City** panel: search across 45+ popular world cities or all 600+ IANA time zones.
- All rows update together. Each displays its local date, time, abbreviation,
  and UTC offset. `+1 day` / `-1 day` compare local dates with the reference.
- **Now** reanchors the timeline to the current time and recenters the sliders.
  This is a planning timeline, not a continuously ticking clock.
- In the Base GPUI version, Tab can focus any slider thumb and arrow keys adjust
  it. The pinned GPUI Component sliders are pointer-driven.

Time conversion uses `chrono` and `chrono-tz` with IANA time-zone rules rather
than hardcoded offsets. Moving through daylight-saving transitions therefore
skips nonexistent wall-clock times and distinguishes repeated times by offset.
The bundled rules come from the locked `chrono-tz` version; they are not fetched
from a service.

## Project map

| Path                                 | What to explore                                                                                                       |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------- |
| `crates/time-core/src/lib.rs`        | `Timeline`, shared UTC instant, system zone detection, city list, formatting, and date/DST tests                      |
| `apps/component/src/main.rs`         | GPUI Component `Button`, `SliderState`, slider events, subscriptions, and required `Root` wrapper                     |
| `apps/base/src/main.rs`              | Base GPUI `ButtonRoot`, controlled `SliderRoot`, `SliderControl`, `SliderTrack`, `SliderIndicator`, and `SliderThumb` |
| `Cargo.toml`                         | Main workspace: shared core plus the Component frontend                                                               |
| `apps/base/Cargo.toml`               | Independent Base frontend dependency resolution                                                                       |
| `Cargo.lock`, `apps/base/Cargo.lock` | Reproducible dependency graphs for the two apps                                                                       |
| `.cargo/config.toml`                 | Build concurrency capped at four jobs                                                                                 |
| `rust-toolchain.toml`                | Rust version, formatter, and Clippy                                                                                   |

GPUI's `div()` layout, `Render`, entities, and `cx.notify()` are used directly in
both frontends. The Component version provides styled buttons and a stateful
slider. The Base version assembles headless parts and defines their colors,
geometry, and focus styling locally. Keeping each frontend in one source file
makes those differences easy to inspect without an extra UI abstraction.

To add another city, extend `CITIES` in `time-core` with its `chrono_tz` IANA zone.
Both frontends will include it automatically. To experiment with widget styling,
edit the corresponding frontend; to change conversion behavior, edit the core.

## Why two dependency graphs?

The checked libraries do not currently use the same GPUI source revision.
Even when two Rust crates both say `gpui 0.2.2`, Git and crates.io versions can
expose different APIs and incompatible entity/window types.

| Frontend               | Pinned dependency                                     |
| ---------------------- | ----------------------------------------------------- |
| GPUI Component         | Published `gpui-component = 0.5.1`, `gpui = 0.2.2`    |
| Base GPUI              | Git commit `a603edd0d1cb13d50fb900742ae9cfb78cb737e0` |
| Base's GPUI / platform | Zed commit `1d029c5ff5654fb1b1e8caf4462993c8ee13a133` |

These are verified compatible pairs, deliberately not floating latest versions.
The Base frontend is excluded from the top-level workspace and has its own
workspace and lockfile. Both apps depend on the same small core via a relative
path. No dependency uses or modifies your existing library checkouts.

If upgrading Base GPUI, update its GPUI and platform revisions to the exact
revision used by that Base GPUI commit, then regenerate its lockfile and test.
Do not independently update GPUI and expect the component APIs to stay compatible.

## Development checks

Run from the project root:

```sh
cargo fmt --all --check
cargo fmt --manifest-path apps/base/Cargo.toml --check

# Conversion tests and Component UI interaction test
cargo test --workspace --locked
# Base UI interaction test (not included in the root workspace)
cargo test --manifest-path apps/base/Cargo.toml --locked

cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --manifest-path apps/base/Cargo.toml --all-targets --locked -- -D warnings
```

UI tests simulate actual slider pointer interactions and assert that the shared
timeline changes, including the roughly 12-hour offsets at quarter/three-quarter
positions. Core tests cover date rollover, reference changes, slider limits, and
spring/fall DST transitions.

## Scope and build footprint

This is a runnable learning prototype: a curated city list, manual reference
selection, and one timeline. It does not yet include city search, editable city
lists, arbitrary date entry, persisted preferences, packaging, or networking.

Both GUI frameworks have substantial dependency graphs. They build separately
into `target/` and `apps/base/target/`. Debug symbols and incremental compilation
are disabled to reduce disk use, at the cost of less debugger detail and slower
rebuilds. No unrelated caches or projects were deleted during setup.
