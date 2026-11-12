# Tasks

## 1. Ruined City biome

- [x] 1.1 `BiomeBand::RuinedCity` from its own noise layered over `biome_for`, forced within 96 tiles of the datacenter; ash, cracked road, rubble and concrete tiles (appended to `define_tiles!`), rubble's mining drop, and city chunks skipping trees, rare resources and wild crops; re-pin the surface hash on purpose; verify `cargo test` for cities existing and being rare, tiles outside cities unchanged, rubble's drop, the forced city, and determinism
- [x] 1.2 The `skyscraper` structure (`plan.toml`, four floors with stairs, collapsed floors, scrap, treasure and an android spawn), checked with `just util-structure-preview skyscraper`, and city chunks rolling it at 30%; verify the structure lint tests and a `cargo test` that some city chunks hold one

## 2. Datacenter and sealed door

- [x] 2.1 `FactoryKind::SealedDoor`: a 10 kW consumer that lets only the player through while powered; verify `cargo test` for sealed without power, open with it, closed again when power is lost, and creatures never passing
- [x] 2.2 The `datacenter` structure (entrance with android spawns, generator room with an unfueled generator and pole, server hall behind a sealed door with a `TreasureQuest2` chest, and a transit station named "Datacenter Station"), queued at `DATACENTER_POS` in `Game::new` and for old saves; verify `cargo test` that every new world has it at its place in a city, its chest holds the data shard, and `game_new_matches_golden_entities` (re-recorded on purpose)

## 3. The quest

- [x] 3.1 `QuestType::BlackBox`, the `PoweredAt` objective, the four ordered steps, the android's offer (after "A New Arm") and hand-in dialogue, the data shard item with its lore text, the seeded reward (a standard fusion core and a gun with ammo from `BLACK_BOX_GUNS`), the datacenter marker, and `tp_quest datacenter`; verify `cargo test` for not offered early, offered after the arm, each step completing in order (including early), the hand-in taking the shard and giving the reward, the same gun for the same seed, more than one gun across seeds, and `save_reload_anywhere_changes_nothing`

## 4. Client and art

- [x] 4.1 Sprites for the city tiles, the sealed door (open and sealed), the data shard and the skyscraper's new tiles; Look text for the door and generator room; verify the sprite gallery and font tests
- [x] 4.2 `just client-blind` snapshots: the city with a skyscraper, the datacenter's sealed door before and after fueling the generator, and the hand-in dialogue; verify the new snapshots

## 5. Checks

- [x] 5.1 Update `docs/QUESTS.md` with quest 3, and add `datacenter` to the `tp_quest` line in `AGENTS.md`; remove the nuked-city row from `docs/SPEC-GAPS.md`; verify by reading the diff
- [x] 5.2 Verify `just code-clippy`, `cargo test --workspace`, `just util-bench` and PR checks pass
- [ ] 5.3 User check: the city feels like the old world's ruins, the datacenter is findable and the power step is clear, and the reveal lands

## Workflow follow-up

- When merged with only 5.3 left, add a playtest queue entry for it.
