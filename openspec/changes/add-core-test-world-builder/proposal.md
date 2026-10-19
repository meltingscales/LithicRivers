# Proposal

## Why

Core game logic (combat, entity AI, interaction, movement) is the riskiest code and has the least test coverage. Every existing test builds its setup by hand, and `Game::new` spawns sheep, a dead dog, NPCs and structures near the player, so tests built on it are noisy. A small fixture builder makes writing a focused test cheap, for humans and agents alike.

## What Changes

- Add a test fixture builder in `lithicrivers-core`: start from a seed with only the player, then place tiles, items, and entities (sheep, feral dogs, NPCs) at chosen positions, and build a `Game`.
- Add helpers to drive the built game: queue a player intent, run ticks, and query entities/positions/inventory.
- Expose the builder to `#[cfg(test)]` code in core and, behind a `test-support` cargo feature, to other crates' tests (the client's snapshot tests).
- Add initial tests using the builder for combat entry, feral dog behavior, and item pickup.

## Capabilities

### New Capabilities
- `core-test-fixtures`: Building minimal, deterministic game scenarios in tests and driving them tick by tick.

### Modified Capabilities
<!-- None: no existing specs. -->

## Impact

- `crates/core`: new test-support module, `test-support` feature in `Cargo.toml`; `Game` may gain a constructor that skips the default spawns.
- `crates/core/src/systems_tests.rs` and per-system test modules gain builder-based tests.
- No change to gameplay behavior or release builds.
