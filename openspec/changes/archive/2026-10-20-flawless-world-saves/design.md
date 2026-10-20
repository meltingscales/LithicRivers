# Design

## Context

- `crates/core/src/save_load.rs`: `SaveData` (version 4) has per-type structs (`PlayerSave`, `SheepSave`, `FeralDogSave`, `DroppedItemSave`, `NPCSave`) plus world tiles, chunk/structure state, pending structures, quest markers, active quests, explored chunks and viewport. `apply_to_game` clears the ECS world and respawns entities, adding `Body::default()` for the player.
- Components in `components.rs` / `model/body.rs` mostly derive `Serialize`; `ActionQueue`, `QueuedAction`, `CombatAction`, `Move` (`moves.rs`) don't. `PlayerState`, `PlayerIntent`, `TargetTracker`, `GameEvents`/`MessageLog` aren't serialized.
- `hecs` is 0.10.5; its `row-serialize` feature serializes every entity through a user-supplied context that maps component types to stable names, and deserializes with `World::spawn_at`, so entity IDs are preserved.
- Client: `TutorialSystem` (current step, completed tutorials), `BuildPanelState` (hotbar slots), view position live in `App`. Combat UI is opened from tick results (`App::apply_tick_result`), not from core state.
- Tests: `save_load_json_roundtrip_after_actions` covers movement only. The headless runner and `TestWorld` exist for scenario setup.

## Goals / Non-Goals

**Goals:**
- Loading restores exactly what was saved, by construction rather than by a per-type list.
- A failing test, not a player bug report, when someone adds an unsaved component.
- One acceptance check that covers future behavior: script with `save-reload` == script without it.

**Non-Goals:**
- Save file size optimization, compression, or msgpack by default (JSON stays; msgpack helpers keep working).
- World folders and menus (that's `add-title-and-world-menu`).
- Saving transient UI (open modals, selections, cheat console text, splash state).

## Decisions

### Entities: hecs row serialization with a component registry
Enable `hecs` `row-serialize`. A `SaveContext` lists every saveable component with a stable string key (`"position"`, `"body"`, `"action_queue"`, ...) via one macro, so serialize and deserialize can't disagree. Entities are written with their IDs and restored with `spawn_at`, keeping cross-entity references valid.
Alternative: extend the per-type `*Save` structs. Rejected: it's exactly how the current gaps happened, and every new component would need edits in two places.

### Registry completeness test
A test builds worlds that exercise every spawn path (`Game::new`, `TestWorld` with every creature kind, a fight to death producing a corpse, a stunned and a delayed entity, items, NPCs), then walks every archetype's component `TypeId`s and fails, naming the type, if any isn't in the registry. Marker components (`Player`, `Sheep`, `FeralDog`, `GameEntity`, `BlocksMovement`, `Dead`, ...) are registered too.

### Resources: one `SavedResources` struct
Everything in `Resources` except `config` (rebuilt from the config file) and the scheduler: time, world state (tiles, seed), player state, events/message log, target tracker, pending structures, chunk/structure generation state, quest markers, active quests, explored chunks. `TargetTracker` stores entity IDs, which stay valid because entity IDs are preserved.

### Client play state
The client adds a `client` section: tutorial `current_step_index` and `completed_tutorials`, hotbar slot assignments, view position. On load, if core `combat_active` is set, the client opens the combat UI (same effect as `apply_tick_result` with `CombatTriggered`, without changing the tab rules). The core save API takes an opaque `serde_json::Value` for the client section so core doesn't depend on client types.

### Format versioning
`SAVE_VERSION` becomes 5. The loader reads `version` first: 5 uses the new path; 4 uses the existing `SaveData` code, renamed `SaveDataV4` and kept unchanged, then logs "Save upgraded from version 4". Anything else is an error naming the version.

### `save-reload` step and the identity test
`save-reload` serializes to an in-memory buffer (no files) and loads it into the same `App`, without logging anything, so output stays identical. The acceptance test takes every existing script (smoke, bug repros, every snapshot script) and, for each, inserts `save-reload` after every step in turn (or every Nth step for long scripts), asserting the output matches the unmodified run. This checks both "exact state" and "same future".

### Determinism hazards
`HashMap` iteration order doesn't matter for correctness, but anything whose order affects behavior (e.g. entity query order) must be the same after reload. `spawn_at` with original IDs keeps hecs archetype and query order stable in practice; the identity test catches any case where it doesn't.

## Risks / Trade-offs

- [Save files get larger (every component, full message log)] → Acceptable for now; JSON size is dominated by tiles. A log cap can come later if needed.
- [Entity ID reuse across saves (hecs generations)] → `spawn_at` restores exact IDs including generation; the identity test covers reuse after deaths.
- [A future component isn't serializable (e.g. holds a handle)] → The completeness test forces a decision: implement serde or mark it explicitly as rebuilt-on-load in the registry with a reason.
- [Behavior hidden in client-only state we didn't list] → The identity test runs every snapshot script; any client state that matters for output shows up as a mismatch.

## Implementation Notes

- **`Resources` derives serde itself** instead of a separate `SavedResources` struct, with `config` marked `#[serde(skip, default = "GameConfig::new")]`. A new resource field is then saved automatically, or fails to compile until it is serializable or explicitly skipped. `WorldState` saves its `ChaCha20Rng` (rand_chacha `serde1`), since combat rolls draw from it.
- **Unregistered components are named by entity, not type.** hecs exposes only `TypeId`s for an entity's components, and Rust can't turn a `TypeId` into a name. `unsaved_components` reports the entity, its registered components and how many types are unregistered, which points at the spawn site.
- **Ordered collections:** `FogOfWar.visited_tiles` is a `BTreeSet` and `TargetTracker` uses `BTreeMap`/`BTreeSet`; maps with tuple keys are written as sorted pairs (`serde_pairs`). Same state, same file.
- **Determinism fix found by the scenario test:** body-part damage picked its part with `rand::thread_rng()` over `HashMap` order. It now sorts the parts and draws from the seeded `world_state.rng`.
- **Version 4** code lives in `save_load/v4.rs` as `SaveDataV4`, only for loading; its save functions are test-only. MessagePack helpers write and read version 5 only.
- The core API takes and returns the client section as `serde_json::Value` (`LoadedSave`); a version 4 load returns its viewport instead.
