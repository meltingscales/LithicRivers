# Tasks

## 1. Checks first

- [ ] 1.1 The lint over every embedded structure (sizes, legend, real blocks, stairs alignment, one source each); verify `cargo test` passes on today's structures, or fix what it finds

## 2. Plans

- [ ] 2.1 `plan.toml` parsing and the compiler (room, corridor, stairs, fill, put, background, legend); verify `cargo test` with a two-room plan and its expected layers
- [ ] 2.2 `just util-structure-build`, run by the build recipes next to the VoxelBuilder importer; verify building twice changes nothing

## 3. Seeing them

- [ ] 3.1 `just util-structure-preview <name>`; verify its output on `small_temple`
- [ ] 3.2 The `structure:<name>` debug world; verify a snapshot of `small_temple` and the unknown-name message

## 4. Docs and checks

- [ ] 4.1 `docs/STRUCTURES.md` with the format, primitives, conventions and the AI prompt workflow, linked from AGENTS.md
- [ ] 4.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.3 User check: make a small structure with an AI from a prompt using the guide
