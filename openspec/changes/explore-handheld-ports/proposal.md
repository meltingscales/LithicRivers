# Proposal

## Why

The user would like LithicRivers on the Nintendo DS and the Game Boy Advance, as homebrew; selling on Nintendo hardware needs a licensed developer account. Both are far smaller machines than anything the game runs on today:
- **GBA:** 256 KB of main RAM plus 32 KB of fast RAM, a 16.8 MHz ARM7 with no floating-point unit, and a 240x160 screen.
- **DS:** 4 MB of RAM, a 67 MHz ARM9, and two 256x192 screens.

Neither can run wgpu, winit, ratatui, rodio or the standard library. A port means rewriting everything outside the core and slimming the core itself. Before anyone writes port specs, we need numbers. The user chose a feasibility spike, not port specs.

## What Changes

- **No game changes.** Measurements and throwaway experiments only, on a spike branch, ending in a written go/no-go per platform in `docs/HANDHELD-PORTS.md`.
- **What gets measured:**
  - **`no_std`:** what stands between `lithicrivers-core` and building as `no_std` + `alloc`. Known today:
    - `std::fs` in config, structures and saves;
    - `serde_json`, `toml`, `rust-embed`, `crossterm`, `tracing-subscriber`, `anyhow`;
    - `noise` (f64 Perlin);
    - `HashMap` with its random hasher.
  - **Memory:** a world's real heap use: the 27 loaded 64x64 chunks, entities and components, resources, explored chunks and the factory index. It's measured with a counting allocator in a test at world start, after walking, and in the factory showcase.
  - **Speed without a floating-point unit:** worldgen's cost per chunk with soft float (or fixed point) on ARM7/ARM9 speeds. It's estimated from host timings and checked on an emulator if a toolchain builds.
  - **Toolchains:** whether a trivial Rust program builds and runs on each: `agb` for GBA (`thumbv4t-none-eabi`), and BlocksDS or devkitARM bindings for the DS (`armv5te-none-eabi`), on mGBA and melonDS.
  - **Screens and controls:** how the game's views would fit 30x20 cells (GBA) or 32x24 cells per screen (DS), and what the D-pad and buttons map to.
- **The writeup** gives each platform a go, a "go with a reduced 'Lite' scope" (what to cut: chunk radius, factory size, creature counts), or a no-go, with the evidence. If it's a go, it names the follow-up changes to propose.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. This is research; it sets `skip_specs`.

## Impact

- A spike branch with experiments that don't merge, apart from the writeup and any measurement test worth keeping. A counting-allocator memory test could stay as a guard on world memory.
- `docs/HANDHELD-PORTS.md`, and an entry in `docs/SPEC-GAPS.md` or follow-up changes depending on the outcome.
