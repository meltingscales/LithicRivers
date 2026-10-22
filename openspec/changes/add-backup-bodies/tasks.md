# Tasks

## 0. Decisions

- [x] 0.1 User decisions, recorded in design.md: a crude starter vat, a mid-game standard vat, and an end-game advanced vat for complex bodies

## 1. Core

- [ ] 1.1 Body-part items in three grades and their recipes, waking at their grade's state; verify the reachability test and installing one into a missing part
- [ ] 1.2 The three vat tiers (part slots per layout, readiness, power for standard and advanced) and a showcase example of each; verify `cargo test` for readiness with and without power, the advanced vat's multi-arm layouts, and the showcase coverage test
- [ ] 1.3 Resurrecting into a chosen vat, and the junker fallback at the HQ; verify tests for both, and that the vat empties

## 2. Client

- [ ] 2.1 The vat window and the Resurrect screen; verify snapshots and a headless test choosing between two vats
- [ ] 2.2 The flyover on `app.clock`, skippable; verify snapshots of its frames and that skipping leaves the same state

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that preparing a vat feels worth it and the flyover looks good
