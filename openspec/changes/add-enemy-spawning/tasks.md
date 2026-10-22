# Tasks

## 0. Decisions

- [x] 0.1 User decision, recorded in design.md: the player walks through gates only, with nothing to open, and not through fences
- [ ] 0.2 User decision: whether spawners are craftable; record it in design.md

## 1. Core

- [ ] 1.1 The spawn rule table, the 10-second check, per-chunk cooldowns and caps (saved); verify `cargo test` for refilling, caps and determinism
- [ ] 1.2 The no-spawn rules (in view, near the player, near lights, inside fenced areas); verify tests for each
- [ ] 1.3 Fences (blocking creatures and the player) and gates (blocking creatures only), neither blocking sight or light; verify tests with a dog behind a fence and the player walking through a gate
- [ ] 1.4 Spawner blocks with their own cap and the spawner core item; verify tests, the reachability rule if craftable, and a showcase farm with the coverage test

## 2. Client

- [ ] 2.1 Sprites for spawners, fences and gates, and the look panel naming a spawner's creature and timer; verify the sprite gallery and a snapshot

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that areas feel alive without floods, and that a farm is fun to build
