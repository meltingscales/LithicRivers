# Tasks

## 0. Decisions

- [ ] 0.1 User decisions: the vat's cost and the factory tier of its parts; record them in design.md

## 1. Core

- [ ] 1.1 Body-part items and recipes, Functional at 100 integrity; verify the reachability test and installing one into a missing part
- [ ] 1.2 The resurrection vat (part slots, readiness, power) and a showcase example; verify `cargo test` for readiness with and without power and the showcase coverage test
- [ ] 1.3 Resurrecting into a chosen vat, and the junker fallback at the HQ; verify tests for both, and that the vat empties

## 2. Client

- [ ] 2.1 The vat window and the Resurrect screen; verify snapshots and a headless test choosing between two vats
- [ ] 2.2 The flyover on `app.clock`, skippable; verify snapshots of its frames and that skipping leaves the same state

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that preparing a vat feels worth it and the flyover looks good
