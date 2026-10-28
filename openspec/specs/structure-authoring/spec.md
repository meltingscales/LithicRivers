# structure-authoring Specification

## Purpose
Make multiblock structures easy to write from a prompt, check them automatically, and see them without playing.

## Requirements

### Requirement: Structure plans
A structure SHALL be describable by a plan of primitives (rooms, corridors, stairs, fills and single blocks) that compiles into the existing layer format, and a structure SHALL have only one source: a plan or hand-written layers. Builds SHALL NOT regenerate hand-written layers.

#### Scenario: Compile a plan
- **WHEN** a plan with two rooms joined by a corridor and a stairs column across 2 layers is compiled
- **THEN** `shape_layers.txt` holds 2 layers with both rooms, the corridor, and stairs at the same spot on both layers

#### Scenario: A hand edit survives the build
- **WHEN** a hand-written structure's layers are edited and the game is built
- **THEN** the edit is still there

### Requirement: Structures are checked
Every structure SHALL be checked by `cargo test`: equal rectangular layers, every character in the legend, every legend entry a real block or spawn point, and stairs lined up across consecutive layers.

#### Scenario: A typo
- **WHEN** a layer uses a character missing from the legend
- **THEN** the test fails naming the structure, the layer, the position and the character

### Requirement: Previewing a structure
A structure SHALL be viewable as text layer by layer, and in the real renderer through a debug world that places it at the origin.

#### Scenario: Preview in the renderer
- **WHEN** a script starts `world structure:small_temple`
- **THEN** the temple stands at (0, 0, 0) with fog of war off
