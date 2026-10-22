# Tasks

## 1. Core

- [x] 1.1 `Blueprint`, `capture` and rotation; verify `cargo test` for capturing belts, chests and filters (but not contents) and for rotating offsets and facings
- [x] 1.2 All-or-nothing `paste` paid from the inventory, with replace detection; verify tests for a short inventory, blocked tiles, replacing into the inventory, and an exact paste
- [x] 1.3 The library on `Resources`; verify the save/reload test keeps it

## 2. Client

- [x] 2.1 Blueprint mode, corner selection and the blinking preview; verify the `blueprint-paste-preview` snapshot, with blocked and replaced counts in the help panel (colors don't show in text)
- [x] 2.2 The replace confirmation and the Blueprints panel; verify a headless test that pastes over a chest after confirming

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that copying and pasting feels fast
