# Tasks

## 1. Checks first

- [x] 1.1 The lint over every embedded structure (sizes, legend, real blocks, stairs alignment, one source each); verify `cargo test` passes on today's structures, or fix what it finds

## 2. Plans

- [x] 2.1 `plan.toml` parsing and the compiler (room, corridor, stairs, fill, put, background, legend); verify `cargo test` with a two-room plan and its expected layers
- [x] 2.2 `just util-structure-build`, run by the build recipes; verify building twice changes nothing
- [x] 2.3 Retire the VoxelBuilder import: remove the importer, its recipes and `assets/voxelbuilder/`, keep the bunker's and facility's layers as hand-written sources, and update the docs that mention it; verify `just build-linux` leaves every `shape_layers.txt` unchanged

## 3. Seeing them

- [x] 3.1 `just util-structure-preview <name>`; verify its output on `small_temple`
- [x] 3.2 The `structure:<name>` debug world; verify a snapshot of `small_temple` and the unknown-name message

## 4. Docs and checks

- [x] 4.1 `docs/STRUCTURES.md` with the format, primitives, conventions and the AI prompt workflow, linked from AGENTS.md
- [x] 4.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.3 User check: make a small structure with an AI from a prompt using the guide
