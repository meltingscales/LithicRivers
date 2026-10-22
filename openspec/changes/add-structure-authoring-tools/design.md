# Design

## Context

- **`StructureDefinition`** loads `structures/<name>.lrstructure/` from embedded assets:
  - `shape_layers.txt`, layers separated by `~~~~~`
  - `data.json`: a legend of character to block or spawn name, plus generation settings (`gen_biomes`, `gen_chance`, `y_layer_gen_range`)
- **Buried structures** put their top layer at the placement z and stack downward.
- **The importer:** `utility_voxelbuilder_importer` converts VoxelBuilder JSON (colors mapped to characters) and runs before every build.

## Goals / Non-Goals

**Goals:**
- Structures small enough to write in a prompt.
- Mistakes caught by `cargo test`.
- Results visible as text and in the real renderer.
- No change to how structures load or place.

**Non-Goals:**
- Replacing VoxelBuilder (it stays for people who prefer it).
- Procedural dungeons (`add-procedural-dungeons`, later, can emit plans).
- A graphical editor.

## Decisions

### Plans
- **TOML,** because it's easy to write and to read in a diff.
- **Coordinates:** `x, y` from the top-left, and `layer` from the bottom, matching `shape_layers.txt`'s order.
- **Order:** primitives apply in file order, so a later one draws over an earlier one.
  - `room { layer, x, y, w, h, wall = "#", floor = " ", doors = [[x, y], ...] }`
  - `corridor { layer, from = [x, y], to = [x, y], width = 1, floor = " " }` (an L-shaped path)
  - `stairs { x, y, from_layer, to_layer }` (`>` on every layer in between)
  - `fill { layer(s), x, y, w, h, block = "." }`
  - `put { layer, x, y, block }`
- **Background:** unset tiles are `.` (existing worldgen) by default; `background` changes it.
- **Generated files:** the compiler writes `shape_layers.txt` and merges the legend into `data.json`. Both stay checked in, so loading never needs the compiler.

### One source per structure
The lint fails if a structure folder has a plan and is also in the VoxelBuilder importer's map. That way nothing silently overwrites the other.

### Preview and debug world
- **Text preview:** each layer printed with its index and z offset, the legend, the size, and warnings from the lint.
- **`world structure:<name>`:** the flat ore-test-style world with the structure placed at (0, 0, 0) and fog of war off.

## Risks / Trade-offs

- [Another format to maintain] → Plans compile to the existing format, so they're optional and loading doesn't change.
- [Primitives are too limited] → `put` and `fill` cover anything; primitives grow only when a structure needs one.
