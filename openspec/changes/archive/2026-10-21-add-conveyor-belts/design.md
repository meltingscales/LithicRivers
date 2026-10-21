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

## Implementation Notes

- **No separate handoff trait:** `factory::accepts` / `factory::insert` decide by component. An empty `BeltSlot` takes one item; a factory block with an `Inventory` (a chest) takes it if it can stack it or has a free slot. Later machines add their input components there.
- **Ordering:** `belt_order` walks each belt's chain downstream (by the position index) and emits belts sink-first, starting from belts sorted by position, so lines shift by one tile per move without depending on entity order. A loop is cut at the belt where the walk started; a loop with a gap still turns (tested).
- **Loading by hand:** instead of a directional drop key, Interact next to an empty belt offers "Put <item> on belt" for the item selected in the inventory. It goes through the Choose Interaction menu and the context help panel like every other interaction, and needs no new key.
- **Rendering:** a belt carrying an item draws the item's glyph; an empty belt draws `^ > v <` (`factory/belt_<facing>` sprites).
- **Breaking a belt** drops what it carried.
- **Recipe list order:** "Wooden Conveyor" sits before "Chest", so scripts that select the chest recipe move down one more.
