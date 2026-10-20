# Tasks

## 1. Tests that show today's gaps

- [x] 1.1 Add `save-reload` to the headless runner (in-memory save + load of the current format, no logging); verify parser and render round-trip tests cover it
- [x] 1.2 Add the identity test (each script with `save-reload` inserted after each step vs. unmodified) and the save-data scenario tests (damaged body, corpse with loot, hurt sheep, dog inventory, toggles and log, mid-combat, combat target); verify they fail on the current format and record which ones in the PR description

## 2. Serializable state

- [x] 2.1 Derive `Serialize`/`Deserialize` for `ActionQueue`, `QueuedAction`, `CombatAction`, `Move`, `PlayerState`, `PlayerIntent`, `TargetTracker`, `GameEvents`/`MessageLog` and any other component or resource type still missing it; verify `cargo build --workspace` and `just code-clippy` pass
- [x] 2.2 Enable `hecs` `row-serialize` and add the component registry (`SaveContext`) covering every component and marker type; verify the registry completeness test passes and fails, naming the type, when a test-only unregistered component is attached

## 3. New save format

- [x] 3.1 Implement version 5 save/load: entities via the registry with preserved IDs, `SavedResources`, and the opaque client section; keep the version 4 loader as `SaveDataV4`; verify core round-trip tests and a version 4 fixture file load test
- [x] 3.2 Client: write and restore tutorial progress, hotbar slots and view position in the client section, and reopen the combat UI when loading mid-combat; verify the tutorial, hotbar and mid-combat scenario tests
- [x] 3.3 Switch Menu-tab save/load and `save-reload` to the version 5 path; verify every task 1.2 test now passes, including the identity test across all scripts

## 4. Docs and checks

- [x] 4.1 Document `save-reload` in AGENTS.md and the `play-headless` skill, and note in AGENTS.md that new components must be registered for saving; verify `just code-clippy`, `cargo test --workspace` (twice, no pending snapshots) and the PR checks pass
- [ ] 4.2 User check: in the interactive client, save mid-fight with a damaged body and a corpse nearby, quit, relaunch, load, and confirm everything is as you left it
