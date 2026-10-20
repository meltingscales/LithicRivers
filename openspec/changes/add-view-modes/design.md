# Design

## Context

- `render_game_view` (`ui/panels/world.rs`) draws tiles, entities and the player from the core `RenderView`, applying fog of war; the MODES panel shows torch and fog status.
- Power state (`add-power-grid`): networks with per-mille satisfaction; consumers' demand. Machines (`add-tree-automation`): `Machine` buffers and progress. Belts (`add-conveyor-belts`): `BeltSlot`.
- Wires blink from `app.clock` (power-grid design).

## Goals / Non-Goals

**Goals:** a read-only rendering filter per mode, with no gameplay effect.

**Non-Goals:** per-mode tooltips, hover inspection, combining modes, a minimap.

## Decisions

### View mode is client UI state
`ViewMode { Normal, Power, Enemies, Logistics }` in `UiState`. Keybinds `view.NEXT_VIEW_MODE` (default `]`) and `view.PREVIOUS_VIEW_MODE` (default `[`), both unused today. It isn't saved with the world (a display preference), and it's reported in the headless state dump.

### Render as a per-cell classification
The world view computes each cell's glyph and style as today, then asks the active mode to classify the cell: `Highlight(style)`, `Normal`, or `Dim`. Dim recolors to dark gray and keeps the glyph so the player stays oriented. This keeps each mode a small pure function over cell contents.

| Mode | Highlighted | Extra drawing |
|---|---|---|
| Power | poles, wires, generators, consumers | supply-area outline (dotted border); consumer color: green powered, yellow partial (<100%), red unpowered, gray idle; generator `!` when out of fuel |
| Enemies | hostile creatures (entities with `Combat`) in red | none |
| Logistics | belts, chests, machines | inventory fill digit `0`–`9` drawn as a suffix cell or via color ramp; machine status: green working, gray idle, red stuck |

### Factory blocks before their changes land
Modes only reference components that exist. Enemies view needs nothing new; Power and Logistics highlight nothing until their systems land, but cycling still works, so this change can be implemented at any point after `add-factory-foundation` (or even before, with Power/Logistics hidden until their components exist).

## Risks / Trade-offs

- [Color-only distinctions are hard for color-blind players] → Pair colors with glyph cues (`!` out of fuel, `x` stuck) as listed; revisit with an accessibility pass.
- [Keybind conflicts with `[` `]`] → None today; configurable.
