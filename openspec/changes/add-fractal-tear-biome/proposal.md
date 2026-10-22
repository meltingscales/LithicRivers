# Proposal

## Why

The deepest layers are the Lithic Rivers caverns, richer in deposits as they go down, but they look the same at every depth. The endgame needs a place that feels like the edge of the world: strange to look at, dangerous, and holding the materials the last factory tier is made of. The user's idea (IDEAS-INBOX.md): a "Fractal Tear", where reality has torn along self-similar patterns, inspired by Caves of Qud's Moon Stair.

## What Changes

- **The Fractal Tear, an endgame biome:**
  - Rare regions far from spawn in the deep band, and occasionally rifts reaching up into the shallows.
  - Laid out by fractal functions instead of noise:
    - **Mandelbrot and Julia sets** (including the Douady rabbit) shape the open caverns.
    - **Menger-sponge lattices** form walls full of square voids.
    - **Bifurcation curves** (the logistic map) trace branching seams of rare blocks.
- **New blocks:**
  - **Tear glass**, see-through and unbreakable, outlining the set's boundary.
  - **Petrogen shale**, the raw input of petrogen distillation.
  - **Uranic crust**, raw uranium ore for `add-ruined-reactors`' nuclear tier.
  - **Fractal residue**, a sludge that ore washing (`add-advanced-processing`) sifts for very rare materials.
  - Each has a mining drop, and the strange ones have a lore description.
- **Atmosphere:**
  - The biome's colors shift slowly with the game clock, as Moon Stair's light does.
  - Its ambient creatures are a later change; this one adds only the place.
- **Finding it:** the biome compass (`add-biome-compass`) can point to the nearest tear.

## Capabilities

### New Capabilities
- `fractal-tear-biome`: Where Fractal Tears generate, the fractal functions that shape them, and their blocks.

### Modified Capabilities
- `worldgen-depth`: the deep band gains rare Fractal Tear regions, placed through the feature stage.

## Impact

- **Core:**
  - A region test in the feature stage (`world.rs`): a seeded, rare, region-sized chance, at least 2,000 tiles from spawn.
  - Fractal tile functions: integer-friendly escape-time iteration, and Menger-sponge digit tests on coordinates.
  - New tiles and their drops.
- **Client:** sprites for the new blocks (CP437 shades and bright colors, per `docs/ART-DIRECTION.md`), and the color shift on `app.clock`.
- **Tests:**
  - the survey pins the tear's rarity and block mix
  - the fractal functions are deterministic
  - no tear near spawn
- **Depends on:** `overhaul-worldgen-depth`.
