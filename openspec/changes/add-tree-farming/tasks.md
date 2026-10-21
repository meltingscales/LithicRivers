# Tasks

## 1. Core

- [x] 1.1 Add `TileKind::Sapling`, the `Growth` component and planting rules (grass/dirt only, acorn consumed); verify `TestWorld` tests for planting on grass, dirt, rock and an occupied tile
- [x] 1.2 Add the growth factory system with seeded 3–5 minute growth; verify tests that a sapling becomes a tree exactly at `grows_at_tick`, that the same seed and position give the same tick, and that a sapling on a chunk edge grows normally
- [x] 1.3 Breaking a sapling returns the acorn; verify a test

## 2. Client and checks

- [x] 2.1 Sapling glyph and sprite; acorn placeable from the hotbar; verify a headless script that plants an acorn, passes time until it grows, chops the tree, and checks the drops
- [ ] 2.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that planting and growth feel right
