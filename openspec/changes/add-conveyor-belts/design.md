# Design

## Context

`add-factory-foundation` provides factory blocks with facing, a position index, factory steps (5 per game second) and `Game::factory_tile` for chunk-safe neighbor access. Chests have `Storage`.

## Goals / Non-Goals

**Goals:** simple, deterministic item transport into chests and machines.

**Non-Goals:** two-lane belts, splitters, undergrounds, inserters/arms, belts moving the player or creatures, belt speed tiers.

## Decisions

### One item per belt, move every 2 steps
`BeltSlot { item: Option<ItemKind>, progress: u8 }`. An item advances when `progress` reaches 2 (half a game second at 5 steps/second). Deterministic ordering: process belts sink-first. Build the belt graph each step (each belt's successor is the tile it faces), then process in reverse topological order from belts that face a non-belt target, so a full line shifts by one in a single step without depending on entity iteration order. Cycles (belt loops) are processed in a fixed order starting from the lowest position.

### Item handoff interface
`fn accepts(&self, item) -> bool` / `fn insert(&mut self, item) -> bool` implemented for `Storage` (chest) and, in later changes, machine input buffers. Belts call it on the faced block through the position index.

### Loading
After moving, an empty belt takes one `DroppedItem` lying on its tile (one item from the stack). Dropping onto an adjacent belt reuses the inventory drop action with a direction.

### Recipe
Wooden Conveyor: 1 WoodenPlank, 2 Stick, 1 String → 2 Wooden Conveyor (tunable). Added to the recipe table; the reachability test from `add-crafting-tree` covers it.

### Rendering
Belt glyph `^ > v <` in brown; when carrying an item, the item's glyph is drawn on the tile with the belt's color as background.

## Risks / Trade-offs

- [Ordering bugs make lines stutter or duplicate items] → Sink-first processing plus tests for straight lines, corners, merges and loops; an item-count conservation assertion in tests.
- [Large belt networks slow each step] → Graph built per step is O(belts); optimize later if profiling shows it matters.
