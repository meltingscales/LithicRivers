# Proposal

## Why

Once a base has belts, machines, poles and wires, plus creatures roaming around, the world view gets crowded. Players need to see one system at a time: where the power reaches and what's starved, where enemies are, and how items flow. Factorio-style view modes answer "why isn't this working?" at a glance.

## What Changes

- **View modes**, cycled with a keybind and shown in the MODES panel:
  - **Normal**: today's view (plus factory blocks and blinking wires).
  - **Power**: only the grid: poles with their supply areas, wires, generators with fuel status, and every consumer colored by power state (powered, partly powered, unpowered, idle).
  - **Enemies**: only hostile creatures, highlighted; everything else dimmed.
  - **Logistics**: only belts (with direction and carried items), chests and machines with how full their inventories are and whether they're working or stuck.
- In every non-normal mode, terrain and unrelated things are drawn dimmed so the highlighted system stands out; the game keeps running normally.
- Power lines blink every second in Normal and Power views alike.

## Capabilities

### New Capabilities
- `view-modes`: The set of view modes, switching between them, and what each one shows.

### Modified Capabilities
- `game-state-snapshot`: the headless state dump includes the current view mode.

## Impact

- Client: `ViewMode` in UI state, a render filter in the world view, legend in the MODES panel, keybinds `view.NEXT_VIEW_MODE` / `view.PREVIOUS_VIEW_MODE`.
- Saves: the current view mode is saved with the world in `ClientSave` (like zoom and build mode), so a loaded game reopens in the same view.
- Headless: state dump field, snapshot scripts per mode.
- Depends on `add-power-grid` (power view), `add-conveyor-belts` and `add-tree-automation` (logistics view); the enemies view only needs existing creatures, so it can ship first.
