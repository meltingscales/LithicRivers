# Proposal

## Why

Multiblock structures (the bunker, the SapienCorp facility, ships, temples) are made two ways today:
- **By hand** in `shape_layers.txt`: one ASCII grid per layer, separated by `~~~~~`, with a legend in `data.json` mapping characters to blocks and spawn points.
- **In VoxelBuilder** (a 3D editor in the browser): exported to JSON and converted by `utility_voxelbuilder_importer`, which every build reruns.

VoxelBuilder is made for people with a mouse. The user wants to sketch structures with an AI from prompts (IDEAS-INBOX.md). That's hard today:
- Big structures are hundreds of lines of grid.
- Nothing checks a structure before it's placed in a world, so a typo shows up as a broken room in play.
- An AI has no way to see the result without playing.
- Hand edits to the bunker and facility are overwritten by the importer on the next build. This already happened: the arm quest's locker tile in the facility was wiped by a build.
- The user no longer wants to open the VoxelBuilder editor at all.

## What Changes

- **Structure plans:** a `plan.toml` in a `.lrstructure` folder describes a structure with a few primitives instead of every tile:
  - **Shapes:** `room` (walls, floor, door positions), `corridor` (from, to, width), `stairs` (a column spanning layers), `fill` (a box of one block), and `put` (one block or spawn point).
  - **Settings:** the layer count, a size and a legend.
  - `just util-structure-build` compiles every plan into `shape_layers.txt`. A structure has either a plan or a hand-written `shape_layers.txt`, never both.
- **Retire the VoxelBuilder import:** remove `utility_voxelbuilder_importer`, its `just` recipes and the two `.voxelbuilder.json` sources (the bunker and the facility). Their checked-in `shape_layers.txt` become the hand-edited source, so builds stop overwriting them.
- **A preview an AI can read:** `just util-structure-preview <name>` prints each layer as text with the legend, sizes and any warnings. A headless `world structure:<name>` debug world places the structure at the origin, so `just client-blind` can show it in the real renderer.
- **A lint test that runs with `cargo test`:**
  - every structure's layers are rectangles of one size
  - every character is in the legend, and every legend entry names a real block or spawn point
  - stairs line up on consecutive layers
  - quest structures have the spawn points their code expects
- **`docs/STRUCTURES.md`:** the format, the plan primitives, conventions (walls, doors, how buried structures stack), and a prompt workflow for making a structure with an AI and checking it.

## Capabilities

### New Capabilities
- `structure-authoring`: Structure plans and their compiler, the text preview, the structure debug world, the lint test and the authoring guide.

### Modified Capabilities
- `debug-worlds`: adds the `structure:<name>` world type.

## Impact

- **Core:**
  - `structure_plan` (TOML parsing and compiling into layers), adding the `toml` crate as a new dependency.
  - `StructureDefinition` loading unchanged.
  - The lint as a test over every embedded structure.
- **Client:** the `structure:<name>` debug world, and the preview binary.
- **Tools:** `just util-structure-build` and `just util-structure-preview`; the build recipes run the plan compiler instead of the VoxelBuilder importer, which is removed with its JSON sources.
- **Docs:** `docs/STRUCTURES.md`.
