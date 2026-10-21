# Tasks

## 1. World type

- [ ] 1.1 Add `WorldType` to `WorldSettings` (serde default Normal), `Game::new_with_settings`, and the type in `meta.json`; verify old saves and metas load as Normal and a save/reload test keeps the type
- [ ] 1.2 Add the World type row (only with Developer mode on) and the type label in the world list; verify headless tests for hidden/shown and "Lab (Void)", and a snapshot of the form with the row

## 2. Generation

- [ ] 2.1 Void worlds; verify tests for the platform, empty tiles beyond it and no creatures, and same-seed equality
- [ ] 2.2 Ore test worlds from the mining drop table; verify every drop-table tile has a row
- [ ] 2.3 Dog arena; verify exactly 100 extra dogs, all at least 8 tiles away, and same-seed positions
- [ ] 2.4 Factory showcase with the chest example and the coverage test; verify the coverage test passes and fails naming a test-only kind without an example, and the "examples run" test

## 3. Docs and checks

- [ ] 3.1 Document world types in Help, AGENTS.md (creating debug worlds in scripts) and `docs/TECH-TREE.md` (showcase rule for new machines); verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check: each world type looks right and is useful in the interactive client
