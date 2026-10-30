# Tasks

## 1. Toolchains

- [ ] 1.1 Build and run a minimal Rust program on GBA with `agb` in mGBA, and on the DS with the most current Rust route (BlocksDS or devkitARM bindings) in melonDS; record the steps, versions and what didn't work in `docs/HANDHELD-PORTS.md`

## 2. The core on a tiny machine

- [ ] 2.1 List every obstacle to building `lithicrivers-core` as `no_std` + `alloc` (std APIs, dependencies, floating point), with a rough size for each fix; record the list in the writeup
- [ ] 2.2 Measure a world's heap use with a counting allocator at world start, after walking 200 steps, and in the factory showcase; record the numbers against 256 KB (GBA) and 4 MB (DS), and keep the test if it's useful as a guard
- [ ] 2.3 Estimate worldgen's cost per chunk with soft float or fixed point at 16.8 MHz and 67 MHz, from host timings scaled and, if a toolchain works, from a run on the emulator; record it against a 60 Hz frame and against walking speed

## 3. Fitting the game

- [ ] 3.1 Sketch how the world view, menus and dialogue would fit 30x20 cells (GBA) and 32x24 cells per screen (DS), and map the actions to the D-pad, A/B/L/R/Start/Select (and the DS touch screen); record the sketches in the writeup

## 4. Decision

- [ ] 4.1 Write the go, go-Lite or no-go for each platform with its evidence and, for a go, the follow-up changes to propose; verify by review with the user
