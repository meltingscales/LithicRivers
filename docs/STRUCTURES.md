# Structures

Multiblock structures (the bunker, the SapienCorp facility, ships, temples) are folders under `crates/client/assets/structures/`, named `<name>.lrstructure`. The game embeds them at build time.

## The format

Each folder holds:

- **`shape_layers.txt`:** one ASCII grid per layer, separated by lines of `~~~~~`. The first layer in the file is the bottom one (layer 0). Every row of every layer must be the same width, and every layer the same height.
- **`data.json`:** the legend and generation settings:
  ```json
  {
    "blocks": { "#": "rock", " ": "air", ".": "existing_worldgen", "D": "door" },
    "gen_biomes": "QUEST_ONLY",
    "gen_chance": 0.0,
    "y_layer_gen_range": [0, 10]
  }
  ```
  Legend values are tile keys from `crates/core/src/tile_registry.rs`. Some of them place things instead of a block:

  | Key | Places |
  |---|---|
  | `existing_worldgen` | Nothing: the world's own tile stays |
  | `enemy_spawn` | A feral dog |
  | `treasure_common`, `treasure_rare` | Loot on the floor |
  | `scrap_common`, `scrap_rare` | Scrap on the floor |
  | `treasure_quest_1` | The arm quest's parts locker (once per world) |
  | `special_player_spawn` | Where a new game puts the player (the bunker) |
- **`plan.toml`** (optional): a short description the build compiles into the two files above.

A structure has one source: its `plan.toml` if it has one, else its hand-written `shape_layers.txt`. Nothing else rewrites hand-written layers, so edit them directly. Never edit the layers of a planned structure: `cargo test` fails until they match the plan again.

### Placement

- **Normal structures** put layer 0 at the placement z, and the layers above go up.
- **Buried structures** (`bury_structure: true`, used for the bunker and the facility) put the **top** layer at the placement z, and the layers below go down. Their top layer is usually the entrance at the surface.
- `x, y` count from the structure's top-left corner, which lands on the placement x, y.

## Plans

A plan draws shapes in file order, so a later shape draws over an earlier one. Tiles that no shape sets get the `background` character (default `.`, the world's own tile).

```toml
layers = 2
width = 13
height = 5
background = "."

[legend]                # merged into data.json's blocks
"#" = "rock"
" " = "air"
"D" = "door"
">" = "stairs"
"." = "existing_worldgen"

[[shape]]
kind = "room"           # walls around a floor, doors in the walls
layer = 0
x = 0
y = 0
w = 5
h = 5
doors = [[4, 2]]        # wall = "#", floor = " ", door = "D" by default

[[shape]]
kind = "room"
layer = 0
x = 8
y = 0
w = 5
h = 5
doors = [[8, 2]]

[[shape]]
kind = "corridor"       # along x from `from`, then along y to `to`
layer = 0
from = [5, 2]
to = [7, 2]             # width = 1, floor = " " by default

[[shape]]
kind = "stairs"         # ">" on every layer from from_layer to to_layer
x = 2
y = 2
from_layer = 0
to_layer = 1
```

The other shapes:

- `kind = "fill"`: a box of one block: `layer` (or `layers = [low, high]`), `x`, `y`, `w`, `h`, `block`.
- `kind = "put"`: one block or spawn point: `layer`, `x`, `y`, `block`.

Complete examples: `small_temple.lrstructure/plan.toml` (one small room) and `ruined_outpost.lrstructure/plan.toml` (a ruined ground floor with stairs down to a cellar, made with the workflow below). Half the surface forest chunks that roll a ruin place the outpost, buried so its cellar sits a layer down; the last workflow step says how to place a new structure.

## Commands

- `just util-structure-build`: compile every `plan.toml`. The build recipes run it, and it only writes files that would change.
- `just util-structure-preview <name>`: print a structure layer by layer with its legend, size and anything the lint finds.
- In a headless script, `world structure:<name>` starts a flat world with the structure at the origin (layer 0 at z = 0) and fog of war off. In look mode (`action.LOOK_TOGGLE`), `movement.MOVE_UP` and `MOVE_DOWN` move the view between layers. `scripts/snapshots/structure-preview.txt` shows one.

## Checks

`cargo test` runs the lint (`crates/core/src/structure_lint.rs`) over every structure:

- every layer the same size, with rectangular rows
- every character in the legend, and every legend entry a real tile (the loader also checks this)
- every stairs tile with stairs directly above or below it
- every planned structure's layers and legend equal to what its plan compiles to

## Making a structure with an AI

1. Describe the structure in the prompt: its purpose, rough size, levels, what connects them, and what it should hold (enemies, loot, quest tiles). Point the AI at this file.
2. Ask for a `plan.toml` in a new `crates/client/assets/structures/<name>.lrstructure/` folder. Use `put` and `fill` for details the shapes don't cover.
3. Run `just util-structure-build`, then `just util-structure-preview <name>`, and read the layers and problems back. Iterate on the plan until the preview reads right and shows no problems.
4. Look at it in the real renderer with a script: `world structure:<name>`, `key ui.MENU_PREV`, `screen`, then `key action.LOOK_TOGGLE` and `key movement.MOVE_UP` to view the next layer up (the player can only climb on stairs, but look mode moves the view).
5. Run `cargo test`, then place the structure from code (a `PendingStructure`, as `Game::new` does for the facility) or add it to `structure::structures_list()` for random placement.
