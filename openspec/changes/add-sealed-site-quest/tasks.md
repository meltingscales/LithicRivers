# Tasks

## 1. Reactor tier

- [x] 1.1 Tiles `ReactorCasing`, `NuclearWasteDeposit`, `UraniumDeposit` (appended to `define_tiles!`), their drops, the uranium chunk and reactor casing scrap items, and `MiningDrop::hardness` scaling the mine action's cost; verify `cargo test` for each tile's drop, casing taking five times rock's mining time, rock unchanged, and the raw-sources test
- [x] 1.2 Three reactor ruin plans and their own seeded roll (1.5% of surface chunks outside cities, after the existing roll); verify the structure lint, a same-seed placement test, a test that existing structures don't move, and that every layout has uranium and far less uranium than waste
- [x] 1.3 The thermoelectric generator (one-slot inventory for waste or uranium, 30 kW or 60 kW forever, never consumed), its item, recipe, window through `OpenMachine` and save registration; verify `cargo test` for empty, waste and uranium output over many steps, a reload with a source loaded, the reachability test, and a showcase example

## 2. Thale's Annex

- [x] 2.1 The `thale-annex` plan (surface hut, 13-layer shaft with a stairs column, and the annex with its lava ford, outer reactor room, door pole, sealed blast door, lab with a `TreasureQuest3` chest, android and Annex Station), queued at `ANNEX_POS` in `Game::new` and for old saves with the quest line; verify `just util-structure-preview thale-annex`, and `cargo test` that every new world has it, the stairs run from the surface to the annex, the chest holds Thale's journal, the station is named "Annex Station" and can't power the door, and the annex holds enough for one generator

## 3. The quest

- [x] 3.1 `QuestType::SealedSite`, its four ordered steps, the offer and hand-in dialogue, Thale's journal with its lore text, `finish_sealed_site` and its reward, the annex marker, and `tp_quest annex`; verify `cargo test` for not offered before "Black Box", offered after, the steps in order (including early), the door opening only once a placed generator powers it, the hand-in taking the journal and giving the advanced arm and uranium, and `save_reload_anywhere_changes_nothing`

## 4. Client and art

- [x] 4.1 Sprites for the new tiles, items and the generator; verify the sprite gallery and font tests
- [x] 4.2 `just client-blind` snapshots: a reactor ruin, the generator window with a heat source, the annex door opening on a placed generator, and the full quest from the offer to the reveal; verify the new snapshots

## 5. Checks

- [x] 5.1 Update `docs/QUESTS.md` (quest 4), `docs/TECH-TREE.md` (thermoelectric before ore spouts' ingots), `AGENTS.md` (`tp_quest annex`, the generator) and remove `openspec/changes/add-ruined-reactors`; verify by reading the diff
- [x] 5.2 Verify `just code-clippy`, `cargo test --workspace`, `just util-bench` and PR checks pass
- [ ] 5.3 User check: the descent feels like a journey, building the generator to open the door is clear and satisfying, and the aperture reveal lands

## Workflow follow-up

- When merged with only 5.3 left, add a playtest queue entry for it.
