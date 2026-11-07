# Tasks

## 1. Lava

- [x] 1.1 Add `TileKind::Lava` and `TileKind::LavaCrust` (lava impassable, crust passable) with sprites; let sight pass over lava in `blocks_sight`; verify unit tests for walking into lava, mining lava and seeing across it, and `bundled_fonts_cover_every_drawn_character`
- [x] 1.2 Add `factory::place_plain_block` (lava becomes crust) and call it from the client's plain-block placement; verify a core test for bridging with stone and refusing a chest on lava, and a headless test that places stone on lava (`cheat set_tile ~1 ~ ~ lava`)

## 2. Worldgen

- [x] 2.1 Fill the deep river cores with lava and add crust fords from their own noise; verify `worldgen_tests` pins for the lava and crust shares, no lava above the deep, the ford spacing along a river, and the unchanged surface hash
- [x] 2.2 Add `TileKind::Riverglass` on the rims and `ItemKind::Riverglass` (appended) with its mining drop and sprites; verify the riverglass share pin, `every_generated_solid_tile_can_be_mined` and `raw_sources_are_what_the_world_drops`
- [x] 2.3 Add riverglass to the teleporter and advanced power cell recipes; verify the crafting tests

## 3. Snapshots and docs

- [x] 3.1 Update `docs/WORLDGEN-ARCHITECTURE.md`, AGENTS.md (scripting lava with `set_tile`), `docs/STEAM-MVP.md` and `docs/SPEC-GAPS.md`; re-record the snapshots this changes and say why in the PR; verify `just code-clippy` and `cargo test --workspace`
- [ ] 3.2 User check that the deep looks like a place of its own, fords are easy enough to find, and bridging with a block feels natural
