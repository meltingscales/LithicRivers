# Design

## Context

Fog of war lives in `crates/core/src/systems/fog_of_war.rs`. `update_fog_of_war` marks tiles inside the player's light radius as visited each tick, and `FogLookup` is built once per frame so the renderer can ask each tile's `FogState` cheaply: it holds the player's position and light radius (dimmed by a damaged head) and a borrow of the player's `FogOfWar` memory. A tile is `Illuminated` when it's on the surface by day or when the player's light reaches it with line of sight (`has_line_of_sight`, a Bresenham ray stopped by impassable tiles). Factory blocks are entities with a `FactoryKind` and a `Position`, placed and broken by the shared factory code.

## Goals / Non-Goals

**Goals:** a placed block that keeps an area lit, with the same shadows as the player's light; no measurable cost per frame with dozens of lanterns on screen; deterministic and saved like every block.

**Non-Goals:** fuel or power, colored light, light across layers, creature behavior tied to light.

## Decisions

### A lantern is a factory block, not a tile

Lanterns need an entity anyway for Look, breaking and the hotbar, and factory blocks already have placement, breaking, saving and chunk-edge handling. A new `TileKind` would need its own light lookup over chunk data. *Alternative:* a `LightSource` component on any entity, which would also let creatures carry light later. Rejected for now: nothing else needs it, and the factory kind can grow into that if powered lamps arrive.

### Lit tiles are gathered once per `FogLookup`

`FogLookup::new` collects the lanterns on the player's layer within `LANTERN_GATHER_RANGE` (48 tiles: wider than any screen plus the lantern radius) and computes each one's lit tiles once, with `has_line_of_sight` from the lantern, into a `HashSet<(i64, i64)>`. `is_illuminated` then checks the player's light as today, or membership in that set. Each lantern costs about 113 ray casts per frame, independent of screen size. *Alternative:* test every tile against every lantern per tile, which grows with screen area times lanterns. *Alternative:* cache lit sets on the lantern and invalidate on tile changes; faster still, but the invalidation has to catch every mined or placed tile, and the simple version is well inside the budget. A counting test (like `fog_lookups_built`) guards the number of rays cast.

### `update_fog_of_war` marks lantern tiles visited

The tick marks the same lit set visited, so lit areas stay remembered (grey) after the player leaves, exactly like tiles the player lit. It reuses the gathering code with the player's position as the center.

### Numbers

Radius 6: smaller than a carried torch (8), so the torch stays worth crafting, and big enough to light a room. Recipe: 1 torch + 1 iron scrap, reachable in the first minutes.

## Risks / Trade-offs

- [A lantern seen from beyond 48 tiles isn't drawn lit] → nothing is drawn that far away; the range is a constant next to the view code if screens grow.
- [Lanterns reveal tiles the player hasn't walked to, including through gaps in a wall] → intended: the player placed the light. Light still stops at walls.
- [Many lanterns in one view] → the `util-bench` surface walk and a lantern-heavy bench scenario check the 16 ms budget.
