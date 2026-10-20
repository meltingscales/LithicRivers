# Design

## Context

- **World page layout** (`ui/layout.rs`): outside combat, the game view sits on the left. A 24-column sidebar on the right holds the modes panel (3 lines) and then either the inventory list or, in look mode, the look panel. In combat, the combat panel takes the left half.
- **Build modes** are in `BuildPanelState.mode`: `Movement`, `Break` or `Place`. The hotbar shows in place mode. Look mode is `panels.look.mode`.
- **Nearby interactions:** `app/input/world_action.rs` builds `available_actions` (pick up item, loot corpse, talk to NPC, doors) when interact is pressed. That code lives inside the input handler.
- **Keys:** names come from `config_manager.get_printable_key_for_keybind`. `h` is unbound today.
- **Config:** settings persist through `ConfigManager::save`, and headless runs skip saving (`App::persist_config`, added by the title-menu change). Missing keys in old config files are filled from the defaults.

## Goals / Non-Goals

**Goals:**
- A short, always-correct list of the keys for the current situation.
- The panel and the interact key agree, because they share one function.

**Non-Goals:**
- Replacing the Help tab or the tutorial.
- Help text in other tabs, or in combat (the combat panel has its own key hints).
- Mouse support or scrolling in the panel.

## Decisions

### Extract the nearby-interactions search
Move the search that builds `available_actions` into a function, `nearby_interactions(app) -> Vec<InteractionType>`. The interact handler and the panel both call it.
Alternative: a separate, simpler search for the panel. Rejected: the two would drift; for example, the empty-corpse fix would apply to one and not the other.

### Panel content by situation, most specific first
1. **Look mode:** cursor keys and the key that exits look mode.
2. **Break or place mode:** direction keys, the mode-cycle key, and the hotbar in place mode.
3. **Movement mode:** move (shown as a 3×3 grid of the bound keys), mine, look, build mode, plus up to three nearby interactions, such as "f: loot corpse".

Lines are truncated to the sidebar width.

### Layout
- **Normal sidebar:** modes (3) / inventory (min 5) / help (fixed height, about 10 lines; less if the terminal is short).
- **Look mode:** the help panel goes below the look panel instead.
- **Hidden:** the help area collapses to a single "h: help" line at the bottom of the sidebar.

### Persistence
The visibility is read from `ui.SHOW_CONTEXT_HELP` at startup. Toggling updates the in-memory config and saves it when `persist_config` is set, the same path Options uses for volumes. It is a player preference, not world state, so it is not part of the save.

## Risks / Trade-offs

- [The sidebar is narrow (24 columns)] → Keep labels short ("f: loot corpse"); show the move keys as a grid rather than words.
- [World snapshots change] → Expected; the PR lists each changed snapshot. Non-World tabs and combat must not change.
- [Short terminals] → The help panel shrinks first, so the inventory keeps at least 5 lines.
