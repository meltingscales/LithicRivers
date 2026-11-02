# Tasks

## 1. The lantern block

- [x] 1.1 Add `ItemKind::ScrapLantern` and `FactoryKind::ScrapLantern` (name, sprite key, `from_item`, not directional) and the recipe (1 torch + 1 iron scrap); verify `cargo test` for crafting, placing and breaking with `TestWorld`, and the recipe reachability test
- [x] 1.2 The sprite, Look text and Item Dictionary description, following `docs/ART-DIRECTION.md` with characters in the bundled font; verify `bundled_fonts_cover_every_drawn_character` and a snapshot script `scripts/snapshots/scrap-lantern.txt` that places a lantern underground and shows the lit room

## 2. Light

- [x] 2.1 Gather lanterns on the player's layer within 48 tiles into `FogLookup` with their lit tiles (line of sight from the lantern), and use them in `is_illuminated`; verify `cargo test` for the scenarios in the `light-blocks` spec (radius, shadows, other layers, surface at night) and a counting test for the rays cast per lookup
- [x] 2.2 Mark lantern-lit tiles visited in `update_fog_of_war`; verify a test that the tiles stay remembered after the player walks away, and that `save_reload_anywhere_changes_nothing` still passes
- [x] 2.3 Add a lantern-heavy scenario under `scripts/bench/` (20 lanterns in one view underground); verify `just util-bench` stays inside the budget

## 3. Docs and checks

- [x] 3.1 Add the lantern to Light Sources in `docs/MECHANICS.md` and to the factory notes in `AGENTS.md`
- [x] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check that lanterns make an underground base pleasant to see, and that 6 tiles feels right next to the torch's 8
